#![feature(test)]
#![deny(warnings)]

extern crate futures;
extern crate hyper;
extern crate pretty_env_logger;
extern crate test;
extern crate tokio;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;

use futures::{stream, Future, Stream};
use futures::sync::oneshot;

use hyper::{Body, Response, Server};
use hyper::service::service_fn_ok;

macro_rules! bench_server {
    ($b:ident, $header:expr, $body:expr) => ({
        let _ = pretty_env_logger::try_init();
        let (_until_tx, until_rx) = oneshot::channel::<()>();
        let addr = {
            let (addr_tx, addr_rx) = mpsc::channel();
            ::std::thread::spawn(move || {
                let addr = "127.0.0.1:0".parse().unwrap();
                let srv = Server::bind(&addr)
                    .serve(|| {
                        let header = $header;
                        let body = $body;
                        service_fn_ok(move |_| {
                            Response::builder()
                                .header(header.0, header.1)
                                .header("content-type", "text/plain")
                                .body(body())
                                .unwrap()
                        })
                    });
                addr_tx.send(srv.local_addr()).unwrap();
                let fut = srv
                    .map_err(|e| panic!("server error: {}", e))
                    .select(until_rx.then(|_| Ok(())))
                    .then(|_| Ok(()));
                let mut rt = tokio::runtime::current_thread::Runtime::new().unwrap();
                rt.spawn(fut);
                rt.run().unwrap();
            });

            addr_rx.recv().unwrap()
        };

        let total_bytes = {
            let mut tcp = TcpStream::connect(addr).unwrap();
            tcp.write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n").unwrap();
            let mut buf = Vec::new();
            tcp.read_to_end(&mut buf).unwrap()
        };

        let mut tcp = TcpStream::connect(addr).unwrap();
        tcp.set_read_timeout(Some(::std::time::Duration::from_secs(3))).unwrap();
        let mut buf = [0u8; 8192];

        $b.bytes = 35 + total_bytes as u64;
        $b.iter(|| {
            tcp.write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n").unwrap();
            let mut sum = 0;
            while sum < total_bytes {
                sum += tcp.read(&mut buf).unwrap();
            }
            assert_eq!(sum, total_bytes);
        });
    })
}

fn body(b: &'static [u8]) -> hyper::Body {
    b.into()
}

#[bench]
fn throughput_fixedsize_small_payload(b: &mut test::Bencher) {
    bench_server!(b, ("content-length", "13"), || body(b"Hello, World!"))
}

#[bench]
fn throughput_fixedsize_large_payload(b: &mut test::Bencher) {
    bench_server!(b, ("content-length", "1000000"), ||  body(&[b'x'; 1_000_000]))
}

#[bench]
fn throughput_fixedsize_many_chunks(b: &mut test::Bencher) {
    bench_server!(b, ("content-length", "1000000"), || {
        static S: &'static [&'static [u8]] = &[&[b'x'; 1_000] as &[u8]; 1_000] as _;
        Body::wrap_stream(stream::iter_ok::<_, String>(S.iter()).map(|&s| s))
    })
}

#[bench]
fn throughput_chunked_small_payload(b: &mut test::Bencher) {
    bench_server!(b, ("transfer-encoding", "chunked"), || body(b"Hello, World!"))
}

#[bench]
fn throughput_chunked_large_payload(b: &mut test::Bencher) {
    bench_server!(b, ("transfer-encoding", "chunked"), ||  body(&[b'x'; 1_000_000]))
}

#[bench]
fn throughput_chunked_many_chunks(b: &mut test::Bencher) {
    bench_server!(b, ("transfer-encoding", "chunked"), || {
        static S: &'static [&'static [u8]] = &[&[b'x'; 1_000] as &[u8]; 1_000] as _;
        Body::wrap_stream(stream::iter_ok::<_, String>(S.iter()).map(|&s| s))
    })
}

#[bench]
fn raw_tcp_throughput_small_payload(b: &mut test::Bencher) {
    let (tx, rx) = mpsc::channel();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    ::std::thread::spawn(move || {
        let mut sock = listener.accept().unwrap().0;

        let mut buf = [0u8; 8192];
        while rx.try_recv().is_err() {
            sock.read(&mut buf).unwrap();
            sock.write_all(b"\
                HTTP/1.1 200 OK\r\n\
                Content-Length: 13\r\n\
                Content-Type: text/plain; charset=utf-8\r\n\
                Date: Fri, 12 May 2017 18:21:45 GMT\r\n\
                \r\n\
                Hello, World!\
            ").unwrap();
        }
    });

    let mut tcp = TcpStream::connect(addr).unwrap();
    let mut buf = [0u8; 4096];

    b.bytes = 130 + 35;
    b.iter(|| {
        tcp.write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n").unwrap();
        let n = tcp.read(&mut buf).unwrap();
        assert_eq!(n, 130);
    });
    tx.send(()).unwrap();
}

#[bench]
fn raw_tcp_throughput_large_payload(b: &mut test::Bencher) {
    let (tx, rx) = mpsc::channel();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    let srv_head = b"\
        HTTP/1.1 200 OK\r\n\
        Content-Length: 1000000\r\n\
        Content-Type: text/plain; charset=utf-8\r\n\
        Date: Fri, 12 May 2017 18:21:45 GMT\r\n\
        \r\n\
    ";
    ::std::thread::spawn(move || {
        let mut sock = listener.accept().unwrap().0;

        let mut buf = [0u8; 8192];
        while rx.try_recv().is_err() {
            let r = sock.read(&mut buf).unwrap();
            if r == 0 {
                break;
            }
            sock.write_all(srv_head).unwrap();
            sock.write_all(&[b'x'; 1_000_000]).unwrap();
        }
    });

    let mut tcp = TcpStream::connect(addr).unwrap();
    let mut buf = [0u8; 8192];

    let expect_read = srv_head.len() + 1_000_000;
    b.bytes = expect_read as u64 + 35;

    b.iter(|| {
        tcp.write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n").unwrap();
        let mut sum = 0;
        while sum < expect_read {
            sum += tcp.read(&mut buf).unwrap();
        }
        assert_eq!(sum, expect_read);
    });
    tx.send(()).unwrap();
}


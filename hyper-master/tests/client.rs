#![deny(warnings)]
extern crate bytes;
extern crate hyper;
extern crate futures;
extern crate futures_timer;
extern crate net2;
extern crate tokio;
extern crate tokio_io;
extern crate tokio_tcp;
extern crate pretty_env_logger;

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::thread;
use std::time::Duration;

use hyper::{Body, Client, Method, Request, StatusCode};

use futures::{Future, Stream};
use futures::sync::oneshot;
use tokio::runtime::current_thread::Runtime;
use tokio_tcp::{ConnectFuture, TcpListener as TkTcpListener, TcpStream};

fn s(buf: &[u8]) -> &str {
    ::std::str::from_utf8(buf).expect("from_utf8")
}

fn tcp_connect(addr: &SocketAddr) -> ConnectFuture {
    TcpStream::connect(addr)
}

macro_rules! test {
    (
        name: $name:ident,
        server:
            expected: $server_expected:expr,
            reply: $server_reply:expr,
        client:
            request: {$(
                $c_req_prop:ident: $c_req_val: tt,
            )*},

            response:
                status: $client_status:ident,
                headers: { $($response_header_name:expr => $response_header_val:expr,)* },
                body: $response_body:expr,
    ) => (
        test! {
            name: $name,
            server:
                expected: $server_expected,
                reply: $server_reply,
            client:
                set_host: true,
                request: {$(
                    $c_req_prop: $c_req_val,
                )*},

                response:
                    status: $client_status,
                    headers: { $($response_header_name => $response_header_val,)* },
                    body: $response_body,
        }
    );
    (
        name: $name:ident,
        server:
            expected: $server_expected:expr,
            reply: $server_reply:expr,
        client:
            set_host: $set_host:expr,
            request: {$(
                $c_req_prop:ident: $c_req_val:tt,
            )*},

            response:
                status: $client_status:ident,
                headers: { $($response_header_name:expr => $response_header_val:expr,)* },
                body: $response_body:expr,
    ) => (
        test! {
            name: $name,
            server:
                expected: $server_expected,
                reply: $server_reply,
            client:
                set_host: $set_host,
                title_case_headers: false,
                request: {$(
                    $c_req_prop: $c_req_val,
                )*},

                response:
                    status: $client_status,
                    headers: { $($response_header_name => $response_header_val,)* },
                    body: $response_body,
        }
    );
    (
        name: $name:ident,
        server:
            expected: $server_expected:expr,
            reply: $server_reply:expr,
        client:
            set_host: $set_host:expr,
            title_case_headers: $title_case_headers:expr,
            request: {$(
                $c_req_prop:ident: $c_req_val:tt,
            )*},

            response:
                status: $client_status:ident,
                headers: { $($response_header_name:expr => $response_header_val:expr,)* },
                body: $response_body:expr,
    ) => (
        #[test]
        fn $name() {
            let _ = pretty_env_logger::try_init();
            let mut rt = Runtime::new().expect("runtime new");

            let res = test! {
                INNER;
                name: $name,
                runtime: &mut rt,
                server:
                    expected: $server_expected,
                    reply: $server_reply,
                client:
                    set_host: $set_host,
                    title_case_headers: $title_case_headers,
                    request: {$(
                        $c_req_prop: $c_req_val,
                    )*},
            }.expect("test");


            assert_eq!(res.status(), StatusCode::$client_status);
            $(
                assert_eq!(
                    res
                        .headers()
                        .get($response_header_name)
                        .expect(concat!("response header '", stringify!($response_header_name), "'")),
                    $response_header_val,
                    "response header '{}'",
                    stringify!($response_header_name),
                );
            )*

            let body = rt.block_on(res
                .into_body()
                .concat2())
                .expect("body concat wait");

            let expected_res_body = Option::<&[u8]>::from($response_body)
                .unwrap_or_default();
            assert_eq!(body.as_ref(), expected_res_body);
        }
    );
    (
        name: $name:ident,
        server:
            expected: $server_expected:expr,
            reply: $server_reply:expr,
        client:
            request: {$(
                $c_req_prop:ident: $c_req_val:tt,
            )*},

            error: $err:expr,
    ) => (
        #[test]
        fn $name() {
            let _ = pretty_env_logger::try_init();
            let mut rt = Runtime::new().expect("runtime new");

            let err: ::hyper::Error = test! {
                INNER;
                name: $name,
                runtime: &mut rt,
                server:
                    expected: $server_expected,
                    reply: $server_reply,
                client:
                    set_host: true,
                    title_case_headers: false,
                    request: {$(
                        $c_req_prop: $c_req_val,
                    )*},
            }.unwrap_err();

            fn infer_closure<F: FnOnce(&::hyper::Error) -> bool>(f: F) -> F { f }

            let closure = infer_closure($err);
            if !closure(&err) {
                panic!("expected error, unexpected variant: {:?}", err);
            }
        }
    );

    (
        INNER;
        name: $name:ident,
        runtime: $runtime:expr,
        server:
            expected: $server_expected:expr,
            reply: $server_reply:expr,
        client:
            set_host: $set_host:expr,
            title_case_headers: $title_case_headers:expr,
            request: {$(
                $c_req_prop:ident: $c_req_val:tt,
            )*},
    ) => ({
        let server = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = server.local_addr().expect("local_addr");
        let rt = $runtime;

        let connector = ::hyper::client::HttpConnector::new(1);
        let client = Client::builder()
            .set_host($set_host)
            .http1_title_case_headers($title_case_headers)
            .build(connector);

        #[allow(unused_assignments, unused_mut)]
        let mut body = Body::empty();
        let mut req_builder = Request::builder();
        $(
            test!(@client_request; req_builder, body, addr, $c_req_prop: $c_req_val);
        )*
        let req = req_builder
            .body(body)
            .expect("request builder");

        let res = client.request(req);

        let (tx, rx) = oneshot::channel();

        let thread = thread::Builder::new()
            .name(format!("tcp-server<{}>", stringify!($name)));
        thread.spawn(move || {
            let mut inc = server.accept().expect("accept").0;
            inc.set_read_timeout(Some(Duration::from_secs(5))).expect("set_read_timeout");
            inc.set_write_timeout(Some(Duration::from_secs(5))).expect("set_write_timeout");
            let expected = format!($server_expected, addr=addr);
            let mut buf = [0; 4096];
            let mut n = 0;
            while n < buf.len() && n < expected.len() {
                n += match inc.read(&mut buf[n..]) {
                    Ok(n) => n,
                    Err(e) => panic!("failed to read request, partially read = {:?}, error: {}", s(&buf[..n]), e),
                };
            }
            assert_eq!(s(&buf[..n]), expected);

            inc.write_all($server_reply.as_ref()).expect("write_all");
            let _ = tx.send(());
        }).expect("thread spawn");

        let rx = rx.expect("thread panicked");

        rt.block_on(res.join(rx).map(|r| r.0)).map(move |mut resp| {
            // Always check that HttpConnector has set the "extra" info...
            let extra = resp
                .extensions_mut()
                .remove::<::hyper::client::connect::HttpInfo>()
                .expect("HttpConnector should set HttpInfo");

            assert_eq!(extra.remote_addr(), addr, "HttpInfo should have server addr");

            resp
        })
    });

    (
        @client_request;
        $req_builder:ident,
        $body:ident,
        $addr:ident,
        $c_req_prop:ident: $c_req_val:tt
    ) => ({
        __client_req_prop!($req_builder, $body, $addr, $c_req_prop: $c_req_val)
    });
}

macro_rules! __client_req_prop {
    ($req_builder:ident, $body:ident, $addr:ident, headers: $map:tt) => ({
        __client_req_header!($req_builder, $map)
    });

    ($req_builder:ident, $body:ident, $addr:ident, method: $method:ident) => ({
        $req_builder.method(Method::$method);
    });

    ($req_builder:ident, $body:ident, $addr:ident, version: $version:ident) => ({
        $req_builder.version(hyper::Version::$version);
    });

    ($req_builder:ident, $body:ident, $addr:ident, url: $url:expr) => ({
        $req_builder.uri(format!($url, addr=$addr));
    });

    ($req_builder:ident, $body:ident, $addr:ident, body: $body_e:expr) => ({
        $body = $body_e.into();
    });
}

macro_rules! __client_req_header {
    ($req_builder:ident, { $($name:expr => $val:expr,)* }) => {
        $(
        $req_builder.header($name, $val);
        )*
    }
}

static REPLY_OK: &'static str = "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";

test! {
    name: client_get,

    server:
        expected: "GET / HTTP/1.1\r\nhost: {addr}\r\n\r\n",
        reply: REPLY_OK,

    client:
        request: {
            method: GET,
            url: "http://{addr}/",
        },
        response:
            status: OK,
            headers: {
                "Content-Length" => "0",
            },
            body: None,
}

test! {
    name: client_get_query,

    server:
        expected: "GET /foo?key=val HTTP/1.1\r\nhost: {addr}\r\n\r\n",
        reply: REPLY_OK,

    client:
        request: {
            method: GET,
            url: "http://{addr}/foo?key=val#dont_send_me",
        },
        response:
            status: OK,
            headers: {
                "Content-Length" => "0",
            },
            body: None,
}

test! {
    name: client_get_implicitly_empty,

    server:
        expected: "GET / HTTP/1.1\r\nhost: {addr}\r\n\r\n",
        reply: REPLY_OK,

    client:
        request: {
            method: GET,
            url: "http://{addr}/",
            body: "", // not Body::empty
        },
        response:
            status: OK,
            headers: {
                "Content-Length" => "0",
            },
            body: None,
}

test! {
    name: client_post_sized,

    server:
        expected: "\
            POST /length HTTP/1.1\r\n\
            content-length: 7\r\n\
            host: {addr}\r\n\
            \r\n\
            foo bar\
            ",
        reply: REPLY_OK,

    client:
        request: {
            method: POST,
            url: "http://{addr}/length",
            headers: {
                "Content-Length" => "7",
            },
            body: "foo bar",
        },
        response:
            status: OK,
            headers: {},
            body: None,
}

test! {
    name: client_post_chunked,

    server:
        expected: "\
            POST /chunks HTTP/1.1\r\n\
            transfer-encoding: chunked\r\n\
            host: {addr}\r\n\
            \r\n\
            B\r\n\
            foo bar baz\r\n\
            0\r\n\r\n\
            ",
        reply: REPLY_OK,

    client:
        request: {
            method: POST,
            url: "http://{addr}/chunks",
            headers: {
                "Transfer-Encoding" => "chunked",
            },
            body: "foo bar baz",
        },
        response:
            status: OK,
            headers: {},
            body: None,
}

test! {
    name: client_post_empty,

    server:
        expected: "\
            POST /empty HTTP/1.1\r\n\
            content-length: 0\r\n\
            host: {addr}\r\n\
            \r\n\
            ",
        reply: REPLY_OK,

    client:
        request: {
            method: POST,
            url: "http://{addr}/empty",
            headers: {
                "Content-Length" => "0",
            },
        },
        response:
            status: OK,
            headers: {},
            body: None,
}

test! {
    name: client_head_ignores_body,

    server:
        expected: "\
            HEAD /head HTTP/1.1\r\n\
            host: {addr}\r\n\
            \r\n\
            ",
        reply: "\
            HTTP/1.1 200 OK\r\n\
            content-Length: 11\r\n\
            \r\n\
            Hello World\
            ",

    client:
        request: {
            method: HEAD,
            url: "http://{addr}/head",
        },
        response:
            status: OK,
            headers: {},
            body: None,
}

test! {
    name: client_pipeline_responses_extra,

    server:
        expected: "\
            GET /pipe HTTP/1.1\r\n\
            host: {addr}\r\n\
            \r\n\
            ",
        reply: "\
            HTTP/1.1 200 OK\r\n\
            Content-Length: 0\r\n\
            \r\n\
            HTTP/1.1 200 OK\r\n\
            Content-Length: 0\r\n\
            \r\n\
            ",

    client:
        request: {
            method: GET,
            url: "http://{addr}/pipe",
        },
        response:
            status: OK,
            headers: {},
            body: None,
}

test! {
    name: client_requires_absolute_uri,

    server:
        expected: "won't get here {addr}",
        reply: "won't reply",

    client:
        request: {
            method: GET,
            url: "/relative-{addr}",
        },
        error: |err| err.to_string() == "client requires absolute-form URIs",
}

test! {
    name: client_error_unexpected_eof,

    server:
        expected: "\
            GET /err HTTP/1.1\r\n\
            host: {addr}\r\n\
            \r\n\
            ",
        reply: "\
            HTTP/1.1 200 OK\r\n\
            ", // unexpected eof before double CRLF

    client:
        request: {
            method: GET,
            url: "http://{addr}/err",
        },
        error: |err| err.is_incomplete_message(),
}

test! {
    name: client_error_parse_version,

    server:
        expected: "\
            GET /err HTTP/1.1\r\n\
            host: {addr}\r\n\
            \r\n\
            ",
        reply: "\
            HEAT/1.1 200 OK\r\n\
            \r\n\
            ",

    client:
        request: {
            method: GET,
            url: "http://{addr}/err",
        },
        // should get a Parse(Version) error
        error: |err| err.is_parse(),

}

test! {
    name: client_100_continue,

    server:
        expected: "\
            POST /continue HTTP/1.1\r\n\
            content-length: 7\r\n\
            host: {addr}\r\n\
            \r\n\
            foo bar\
            ",
        reply: "\
            HTTP/1.1 100 Continue\r\n\
            \r\n\
            HTTP/1.1 200 OK\r\n\
            Content-Length: 0\r\n\
            \r\n\
            ",

    client:
        request: {
            method: POST,
            url: "http://{addr}/continue",
            headers: {
                "Content-Length" => "7",
            },
            body: "foo bar",
        },
        response:
            status: OK,
            headers: {},
            body: None,
}

test! {
    name: client_connect_method,

    server:
        expected: "\
            CONNECT {addr} HTTP/1.1\r\n\
            host: {addr}\r\n\
            \r\n\
            ",
        reply: "\
            HTTP/1.1 200 OK\r\n\
            \r\n\
            ",

    client:
        request: {
            method: CONNECT,
            url: "{addr}",
        },
        response:
            status: OK,
            headers: {},
            body: None,

}

test! {
    name: client_connect_method_with_absolute_uri,

    server:
        expected: "\
            CONNECT {addr} HTTP/1.1\r\n\
            host: {addr}\r\n\
            \r\n\
            ",
        reply: "\
            HTTP/1.1 200 OK\r\n\
            \r\n\
            ",

    client:
        request: {
            method: CONNECT,
            url: "http://{addr}",
        },
        response:
            status: OK,
            headers: {},
            body: None,

}

test! {
    name: client_set_host_false,

    server:
        // {addr} is here because format! requires it to exist in the string
        expected: "\
            GET /no-host/{addr} HTTP/1.1\r\n\
            \r\n\
            ",
        reply: "\
            HTTP/1.1 200 OK\r\n\
            Content-Length: 0\r\n\
            \r\n\
            ",

    client:
        set_host: false,
        request: {
            method: GET,
            url: "http://{addr}/no-host/{addr}",
        },
        response:
            status: OK,
            headers: {},
            body: None,
}

test! {
    name: client_set_http1_title_case_headers,

    server:
        expected: "\
            GET / HTTP/1.1\r\n\
            X-Test-Header: test\r\n\
            Host: {addr}\r\n\
            \r\n\
            ",
        reply: "\
            HTTP/1.1 200 OK\r\n\
            Content-Length: 0\r\n\
            \r\n\
            ",

    client:
        set_host: true,
        title_case_headers: true,
        request: {
            method: GET,
            url: "http://{addr}/",
            headers: {
                "X-Test-Header" => "test",
            },
        },
        response:
            status: OK,
            headers: {},
            body: None,
}

test! {
    name: client_h1_rejects_http2,

    server:
        expected: "won't get here {addr}",
        reply: "won't reply",

    client:
        request: {
            method: GET,
            url: "http://{addr}/",
            version: HTTP_2,
        },
        error: |err| err.to_string() == "request has unsupported HTTP version",
}

test! {
    name: client_always_rejects_http09,

    server:
        expected: "won't get here {addr}",
        reply: "won't reply",

    client:
        request: {
            method: GET,
            url: "http://{addr}/",
            version: HTTP_09,
        },
        error: |err| err.to_string() == "request has unsupported HTTP version",
}

mod dispatch_impl {
    use super::*;
    use std::io::{self, Read, Write};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;
    use std::time::Duration;

    use futures::{self, Future};
    use futures::sync::{mpsc, oneshot};
    use futures_timer::Delay;
    use tokio_tcp::TcpStream;
    use tokio::runtime::current_thread::Runtime;
    use tokio_io::{AsyncRead, AsyncWrite};

    use hyper::client::connect::{Connect, Connected, Destination, HttpConnector};
    use hyper::Client;
    use hyper;



    #[test]
    fn drop_body_before_eof_closes_connection() {
        // https://github.com/hyperium/hyper/issues/1353
        let _ = pretty_env_logger::try_init();

        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = server.local_addr().unwrap();
        let mut rt = Runtime::new().unwrap();
        let (closes_tx, closes) = mpsc::channel(10);
        let client = Client::builder()
            .build(DebugConnector::with_http_and_closes(HttpConnector::new(1), closes_tx));

        let (tx1, rx1) = oneshot::channel();

        thread::spawn(move || {
            let mut sock = server.accept().unwrap().0;
            sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            sock.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut buf = [0; 4096];
            sock.read(&mut buf).expect("read 1");
            let body = vec![b'x'; 1024 * 128];
            write!(sock, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", body.len()).expect("write head");
            let _ = sock.write_all(&body);
            let _ = tx1.send(());
        });

        let req = Request::builder()
            .uri(&*format!("http://{}/a", addr))
            .body(Body::empty())
            .unwrap();
        let res = client.request(req).and_then(move |res| {
            assert_eq!(res.status(), hyper::StatusCode::OK);
            Delay::new(Duration::from_secs(1))
                .expect("timeout")
        });
        let rx = rx1.expect("thread panicked");
        rt.block_on(res.join(rx).map(|r| r.0)).unwrap();

        rt.block_on(closes.into_future()).unwrap().0.expect("closes");
    }

    #[test]
    fn dropped_client_closes_connection() {
        // https://github.com/hyperium/hyper/issues/1353
        let _ = pretty_env_logger::try_init();

        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = server.local_addr().unwrap();
        let mut rt = Runtime::new().unwrap();
        let (closes_tx, closes) = mpsc::channel(10);

        let (tx1, rx1) = oneshot::channel();

        thread::spawn(move || {
            let mut sock = server.accept().unwrap().0;
            sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            sock.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut buf = [0; 4096];
            sock.read(&mut buf).expect("read 1");
            let body =[b'x'; 64];
            write!(sock, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", body.len()).expect("write head");
            let _ = sock.write_all(&body);
            let _ = tx1.send(());
        });

        let res = {
            let client = Client::builder()
                .build(DebugConnector::with_http_and_closes(HttpConnector::new(1), closes_tx));

            let req = Request::builder()
                .uri(&*format!("http://{}/a", addr))
                .body(Body::empty())
                .unwrap();
            client.request(req).and_then(move |res| {
                assert_eq!(res.status(), hyper::StatusCode::OK);
                res.into_body().concat2()
            }).and_then(|_| {
                Delay::new(Duration::from_secs(1))
                    .expect("timeout")
            })
        };
        // client is dropped
        let rx = rx1.expect("thread panicked");
        rt.block_on(res.join(rx).map(|r| r.0)).unwrap();

        rt.block_on(closes.into_future()).unwrap().0.expect("closes");
    }


    #[test]
    fn drop_client_closes_idle_connections() {
        let _ = pretty_env_logger::try_init();

        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = server.local_addr().unwrap();
        let mut rt = Runtime::new().unwrap();
        let (closes_tx, mut closes) = mpsc::channel(10);

        let (tx1, rx1) = oneshot::channel();
        let (_client_drop_tx, client_drop_rx) = oneshot::channel::<()>();

        thread::spawn(move || {
            let mut sock = server.accept().unwrap().0;
            sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            sock.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut buf = [0; 4096];
            sock.read(&mut buf).expect("read 1");
            let body =[b'x'; 64];
            write!(sock, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", body.len()).expect("write head");
            let _ = sock.write_all(&body);
            let _ = tx1.send(());

            // prevent this thread from closing until end of test, so the connection
            // stays open and idle until Client is dropped
            let _ = client_drop_rx.wait();
        });

        let client = Client::builder()
            .build(DebugConnector::with_http_and_closes(HttpConnector::new(1), closes_tx));

        let req = Request::builder()
            .uri(&*format!("http://{}/a", addr))
            .body(Body::empty())
            .unwrap();
        let res = client.request(req).and_then(move |res| {
            assert_eq!(res.status(), hyper::StatusCode::OK);
            res.into_body().concat2()
        });
        let rx = rx1.expect("thread panicked");
        rt.block_on(res.join(rx).map(|r| r.0)).unwrap();

        // not closed yet, just idle
        {
            rt.block_on(futures::future::poll_fn(|| {
                assert!(closes.poll()?.is_not_ready());
                Ok::<_, ()>(().into())
            })).unwrap();
        }
        drop(client);

        let t = Delay::new(Duration::from_millis(100))
            .map(|_| panic!("time out"));
        let close = closes.into_future()
            .map(|(opt, _)| {
                opt.expect("closes");
            })
            .map_err(|_| panic!("closes dropped"));
        let _ = rt.block_on(t.select(close));
    }


    #[test]
    fn drop_response_future_closes_in_progress_connection() {
        let _ = pretty_env_logger::try_init();

        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = server.local_addr().unwrap();
        let mut rt = Runtime::new().unwrap();
        let (closes_tx, closes) = mpsc::channel(10);

        let (tx1, rx1) = oneshot::channel();
        let (_client_drop_tx, client_drop_rx) = oneshot::channel::<()>();

        thread::spawn(move || {
            let mut sock = server.accept().unwrap().0;
            sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            sock.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut buf = [0; 4096];
            sock.read(&mut buf).expect("read 1");
            // we never write a response head
            // simulates a slow server operation
            let _ = tx1.send(());

            // prevent this thread from closing until end of test, so the connection
            // stays open and idle until Client is dropped
            let _ = client_drop_rx.wait();
        });

        let res = {
            let client = Client::builder()
                .build(DebugConnector::with_http_and_closes(HttpConnector::new(1), closes_tx));

            let req = Request::builder()
                .uri(&*format!("http://{}/a", addr))
                .body(Body::empty())
                .unwrap();
            client.request(req)
        };

        rt.block_on(res.select2(rx1)).unwrap();
        // res now dropped
        let t = Delay::new(Duration::from_millis(100))
            .map(|_| panic!("time out"));
        let close = closes.into_future()
            .map(|(opt, _)| {
                opt.expect("closes");
            })
            .map_err(|_| panic!("closes dropped"));
        let _ = rt.block_on(t.select(close));
    }

    #[test]
    fn drop_response_body_closes_in_progress_connection() {
        let _ = pretty_env_logger::try_init();

        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = server.local_addr().unwrap();
        let mut rt = Runtime::new().unwrap();
        let (closes_tx, closes) = mpsc::channel(10);

        let (tx1, rx1) = oneshot::channel();
        let (_client_drop_tx, client_drop_rx) = oneshot::channel::<()>();

        thread::spawn(move || {
            let mut sock = server.accept().unwrap().0;
            sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            sock.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut buf = [0; 4096];
            sock.read(&mut buf).expect("read 1");
            write!(sock, "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n").expect("write head");
            let _ = tx1.send(());

            // prevent this thread from closing until end of test, so the connection
            // stays open and idle until Client is dropped
            let _ = client_drop_rx.wait();
        });

        let res = {
            let client = Client::builder()
                .build(DebugConnector::with_http_and_closes(HttpConnector::new(1), closes_tx));

            let req = Request::builder()
                .uri(&*format!("http://{}/a", addr))
                .body(Body::empty())
                .unwrap();
            // notably, havent read body yet
            client.request(req)
        };

        let rx = rx1.expect("thread panicked");
        rt.block_on(res.join(rx).map(|r| r.0)).unwrap();

        let t = Delay::new(Duration::from_millis(100))
            .map(|_| panic!("time out"));
        let close = closes.into_future()
            .map(|(opt, _)| {
                opt.expect("closes");
            })
            .map_err(|_| panic!("closes dropped"));
        let _ = rt.block_on(t.select(close));
    }

    #[test]
    fn no_keep_alive_closes_connection() {
        // https://github.com/hyperium/hyper/issues/1383
        let _ = pretty_env_logger::try_init();

        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = server.local_addr().unwrap();
        let mut rt = Runtime::new().unwrap();
        let (closes_tx, closes) = mpsc::channel(10);

        let (tx1, rx1) = oneshot::channel();
        let (_tx2, rx2) = oneshot::channel::<()>();

        thread::spawn(move || {
            let mut sock = server.accept().unwrap().0;
            sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            sock.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut buf = [0; 4096];
            sock.read(&mut buf).expect("read 1");
            sock.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n").unwrap();
            let _ = tx1.send(());
            let _ = rx2.wait();
        });

        let client = Client::builder()
            .keep_alive(false)
            .build(DebugConnector::with_http_and_closes(HttpConnector::new(1), closes_tx));

        let req = Request::builder()
            .uri(&*format!("http://{}/a", addr))
            .body(Body::empty())
            .unwrap();
        let res = client.request(req).and_then(move |res| {
            assert_eq!(res.status(), hyper::StatusCode::OK);
            res.into_body().concat2()
        });
        let rx = rx1.expect("thread panicked");
        rt.block_on(res.join(rx).map(|r| r.0)).unwrap();

        let t = Delay::new(Duration::from_millis(100))
            .map(|_| panic!("time out"));
        let close = closes.into_future()
            .map(|(opt, _)| {
                opt.expect("closes");
            })
            .map_err(|_| panic!("closes dropped"));
        let _ = rt.block_on(t.select(close));
    }

    #[test]
    fn socket_disconnect_closes_idle_conn() {
        // notably when keep-alive is enabled
        let _ = pretty_env_logger::try_init();

        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = server.local_addr().unwrap();
        let mut rt = Runtime::new().unwrap();
        let (closes_tx, closes) = mpsc::channel(10);

        let (tx1, rx1) = oneshot::channel();

        thread::spawn(move || {
            let mut sock = server.accept().unwrap().0;
            sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            sock.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut buf = [0; 4096];
            sock.read(&mut buf).expect("read 1");
            sock.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n").unwrap();
            let _ = tx1.send(());
        });

        let client = Client::builder()
            .build(DebugConnector::with_http_and_closes(HttpConnector::new(1), closes_tx));

        let req = Request::builder()
            .uri(&*format!("http://{}/a", addr))
            .body(Body::empty())
            .unwrap();
        let res = client.request(req).and_then(move |res| {
            assert_eq!(res.status(), hyper::StatusCode::OK);
            res.into_body().concat2()
        });
        let rx = rx1.expect("thread panicked");
        rt.block_on(res.join(rx).map(|r| r.0)).unwrap();

        let t = Delay::new(Duration::from_millis(100))
            .map(|_| panic!("time out"));
        let close = closes.into_future()
            .map(|(opt, _)| {
                opt.expect("closes");
            })
            .map_err(|_| panic!("closes dropped"));
        let _ = rt.block_on(t.select(close));
    }

    #[test]
    fn connect_call_is_lazy() {
        // We especially don't want connects() triggered if there's
        // idle connections that the Checkout would have found
        let _ = pretty_env_logger::try_init();

        let _rt = Runtime::new().unwrap();
        let connector = DebugConnector::new();
        let connects = connector.connects.clone();

        let client = Client::builder()
            .build(connector);

        assert_eq!(connects.load(Ordering::Relaxed), 0);
        let req = Request::builder()
            .uri("http://hyper.local/a")
            .body(Body::empty())
            .unwrap();
        let _fut = client.request(req);
        // internal Connect::connect should have been lazy, and not
        // triggered an actual connect yet.
        assert_eq!(connects.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn client_keep_alive_0() {
        let _ = pretty_env_logger::try_init();
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = server.local_addr().unwrap();
        let mut rt = Runtime::new().unwrap();
        let connector = DebugConnector::new();
        let connects = connector.connects.clone();

        let client = Client::builder()
            .build(connector);

        let (tx1, rx1) = oneshot::channel();
        let (tx2, rx2) = oneshot::channel();
        thread::spawn(move || {
            let mut sock = server.accept().unwrap().0;
            //drop(server);
            sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            sock.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut buf = [0; 4096];
            sock.read(&mut buf).expect("read 1");
            sock.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n").expect("write 1");
            let _ = tx1.send(());

            let n2 = sock.read(&mut buf).expect("read 2");
            assert_ne!(n2, 0);
            let second_get = "GET /b HTTP/1.1\r\n";
            assert_eq!(s(&buf[..second_get.len()]), second_get);
            sock.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n").expect("write 2");
            let _ = tx2.send(());
        });


        assert_eq!(connects.load(Ordering::SeqCst), 0);

        let rx = rx1.expect("thread panicked");
        let req = Request::builder()
            .uri(&*format!("http://{}/a", addr))
            .body(Body::empty())
            .unwrap();
        let res = client.request(req);
        rt.block_on(res.join(rx).map(|r| r.0)).unwrap();

        assert_eq!(connects.load(Ordering::SeqCst), 1);

        // sleep real quick to let the threadpool put connection in ready
        // state and back into client pool
        thread::sleep(Duration::from_millis(50));

        let rx = rx2.expect("thread panicked");
        let req = Request::builder()
            .uri(&*format!("http://{}/b", addr))
            .body(Body::empty())
            .unwrap();
        let res = client.request(req);
        rt.block_on(res.join(rx).map(|r| r.0)).unwrap();

        assert_eq!(connects.load(Ordering::SeqCst), 1, "second request should still only have 1 connect");
        drop(client);
    }

    #[test]
    fn client_keep_alive_extra_body() {
        let _ = pretty_env_logger::try_init();
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = server.local_addr().unwrap();
        let mut rt = Runtime::new().unwrap();

        let connector = DebugConnector::new();
        let connects = connector.connects.clone();

        let client = Client::builder()
            .build(connector);

        let (tx1, rx1) = oneshot::channel();
        let (tx2, rx2) = oneshot::channel();
        thread::spawn(move || {
            let mut sock = server.accept().unwrap().0;
            sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            sock.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut buf = [0; 4096];
            sock.read(&mut buf).expect("read 1");
            sock.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello").expect("write 1");
            // the body "hello", while ignored because its a HEAD request, should mean the connection
            // cannot be put back in the pool
            let _ = tx1.send(());

            let mut sock2 = server.accept().unwrap().0;
            let n2 = sock2.read(&mut buf).expect("read 2");
            assert_ne!(n2, 0);
            let second_get = "GET /b HTTP/1.1\r\n";
            assert_eq!(s(&buf[..second_get.len()]), second_get);
            sock2.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n").expect("write 2");
            let _ = tx2.send(());
        });


        assert_eq!(connects.load(Ordering::Relaxed), 0);

        let rx = rx1.expect("thread panicked");
        let req = Request::builder()
            .method("HEAD")
            .uri(&*format!("http://{}/a", addr))
            .body(Body::empty())
            .unwrap();
        let res = client.request(req);
        rt.block_on(res.join(rx).map(|r| r.0)).unwrap();

        assert_eq!(connects.load(Ordering::Relaxed), 1);

        let rx = rx2.expect("thread panicked");
        let req = Request::builder()
            .uri(&*format!("http://{}/b", addr))
            .body(Body::empty())
            .unwrap();
        let res = client.request(req);
        rt.block_on(res.join(rx).map(|r| r.0)).unwrap();

        assert_eq!(connects.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn client_keep_alive_when_response_before_request_body_ends() {
        use futures_timer::Delay;
        let _ = pretty_env_logger::try_init();
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = server.local_addr().unwrap();
        let mut rt = Runtime::new().unwrap();

        let connector = DebugConnector::new();
        let connects = connector.connects.clone();

        let client = Client::builder()
            .build(connector);

        let (tx1, rx1) = oneshot::channel();
        let (tx2, rx2) = oneshot::channel();
        let (tx3, rx3) = oneshot::channel();
        thread::spawn(move || {
            let mut sock = server.accept().unwrap().0;
            sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            sock.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut buf = [0; 4096];
            sock.read(&mut buf).expect("read 1");
            sock.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n").expect("write 1");
            // after writing the response, THEN stream the body
            let _ = tx1.send(());

            sock.read(&mut buf).expect("read 2");
            let _ = tx2.send(());

            let n2 = sock.read(&mut buf).expect("read 3");
            assert_ne!(n2, 0);
            let second_get = "GET /b HTTP/1.1\r\n";
            assert_eq!(s(&buf[..second_get.len()]), second_get);
            sock.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n").expect("write 2");
            let _ = tx3.send(());
        });


        assert_eq!(connects.load(Ordering::Relaxed), 0);

        let delayed_body = rx1
            .map_err(|_| -> hyper::Error { panic!("rx1") })
            .and_then(|_| Delay::new(Duration::from_millis(200)).map_err(|_| panic!("delay")))
            .into_stream()
            .map(|_| "hello a");

        let rx = rx2.expect("thread panicked");
        let req = Request::builder()
            .method("POST")
            .uri(&*format!("http://{}/a", addr))
            .body(Body::wrap_stream(delayed_body))
            .unwrap();
        let client2 = client.clone();

        // req 1
        let fut = client.request(req)
            .join(rx)
            .and_then(|_| Delay::new(Duration::from_millis(200)).expect("delay"))
            // req 2
            .and_then(move |()| {
                let rx = rx3.expect("thread panicked");
                let req = Request::builder()
                    .uri(&*format!("http://{}/b", addr))
                    .body(Body::empty())
                    .unwrap();
                client2
                    .request(req)
                    .join(rx)
                    .map(|_| ())
            });

        rt.block_on(fut).unwrap();

        assert_eq!(connects.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn connect_proxy_sends_absolute_uri() {
        let _ = pretty_env_logger::try_init();
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = server.local_addr().unwrap();
        let mut rt = Runtime::new().unwrap();
        let connector = DebugConnector::new()
            .proxy();

        let client = Client::builder()
            .build(connector);

        let (tx1, rx1) = oneshot::channel();
        thread::spawn(move || {
            let mut sock = server.accept().unwrap().0;
            //drop(server);
            sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            sock.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut buf = [0; 4096];
            let n = sock.read(&mut buf).expect("read 1");
            let expected = format!("GET http://{addr}/foo/bar HTTP/1.1\r\nhost: {addr}\r\n\r\n", addr=addr);
            assert_eq!(s(&buf[..n]), expected);

            sock.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n").expect("write 1");
            let _ = tx1.send(());
        });


        let rx = rx1.expect("thread panicked");
        let req = Request::builder()
            .uri(&*format!("http://{}/foo/bar", addr))
            .body(Body::empty())
            .unwrap();
        let res = client.request(req);
        rt.block_on(res.join(rx).map(|r| r.0)).unwrap();
    }

    #[test]
    fn connect_proxy_http_connect_sends_authority_form() {
        let _ = pretty_env_logger::try_init();
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = server.local_addr().unwrap();
        let mut rt = Runtime::new().unwrap();
        let connector = DebugConnector::new()
            .proxy();

        let client = Client::builder()
            .build(connector);

        let (tx1, rx1) = oneshot::channel();
        thread::spawn(move || {
            let mut sock = server.accept().unwrap().0;
            //drop(server);
            sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            sock.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut buf = [0; 4096];
            let n = sock.read(&mut buf).expect("read 1");
            let expected = format!("CONNECT {addr} HTTP/1.1\r\nhost: {addr}\r\n\r\n", addr=addr);
            assert_eq!(s(&buf[..n]), expected);

            sock.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n").expect("write 1");
            let _ = tx1.send(());
        });


        let rx = rx1.expect("thread panicked");
        let req = Request::builder()
            .method("CONNECT")
            .uri(&*format!("http://{}/useless/path", addr))
            .body(Body::empty())
            .unwrap();
        let res = client.request(req);
        rt.block_on(res.join(rx).map(|r| r.0)).unwrap();
    }

    #[test]
    fn client_upgrade() {
        use tokio_io::io::{read_to_end, write_all};
        let _ = pretty_env_logger::try_init();
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = server.local_addr().unwrap();
        let mut rt = Runtime::new().unwrap();

        let connector = DebugConnector::new();

        let client = Client::builder()
            .build(connector);

        let (tx1, rx1) = oneshot::channel();
        thread::spawn(move || {
            let mut sock = server.accept().unwrap().0;
            sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            sock.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut buf = [0; 4096];
            sock.read(&mut buf).expect("read 1");
            sock.write_all(b"\
                HTTP/1.1 101 Switching Protocols\r\n\
                Upgrade: foobar\r\n\
                \r\n\
                foobar=ready\
            ").unwrap();
            let _ = tx1.send(());

            let n = sock.read(&mut buf).expect("read 2");
            assert_eq!(&buf[..n], b"foo=bar");
            sock.write_all(b"bar=foo").expect("write 2");
        });

        let rx = rx1.expect("thread panicked");

        let req = Request::builder()
            .method("GET")
            .uri(&*format!("http://{}/up", addr))
            .body(Body::empty())
            .unwrap();

        let res = client.request(req);
        let res = rt.block_on(res.join(rx).map(|r| r.0)).unwrap();

        assert_eq!(res.status(), 101);
        let upgraded = rt.block_on(res
            .into_body()
            .on_upgrade())
            .expect("on_upgrade");

        let parts = upgraded.downcast::<DebugStream>().unwrap();
        assert_eq!(s(&parts.read_buf), "foobar=ready");

        let io = parts.io;
        let io  = rt.block_on(write_all(io, b"foo=bar")).unwrap().0;
        let vec = rt.block_on(read_to_end(io, vec![])).unwrap().1;
        assert_eq!(vec, b"bar=foo");
    }

    #[test]
    fn alpn_h2() {
        use hyper::Response;
        use hyper::server::conn::Http;
        use hyper::service::service_fn_ok;

        let _ = pretty_env_logger::try_init();
        let mut rt = Runtime::new().unwrap();
        let listener = TkTcpListener::bind(&"127.0.0.1:0".parse().unwrap()).unwrap();
        let addr = listener.local_addr().unwrap();
        let mut connector = DebugConnector::new();
        connector.alpn_h2 = true;
        let connects = connector.connects.clone();

        let client = Client::builder()
            .build::<_, ::hyper::Body>(connector);

        let srv = listener.incoming()
            .into_future()
            .map_err(|_| unreachable!())
            .and_then(|(item, _incoming)| {
                let socket = item.unwrap();
                Http::new()
                    .http2_only(true)
                    .serve_connection(socket, service_fn_ok(|req| {
                        assert_eq!(req.headers().get("host"), None);
                        Response::new(Body::empty())
                    }))
            })
            .map_err(|e| panic!("server error: {}", e));
        rt.spawn(srv);


        assert_eq!(connects.load(Ordering::SeqCst), 0);

        let url = format!("http://{}/a", addr).parse::<::hyper::Uri>().unwrap();
        let res1 = client.get(url.clone());
        let res2 = client.get(url.clone());
        let res3 = client.get(url.clone());
        rt.block_on(res1.join(res2).join(res3)).unwrap();

        // Since the client doesn't know it can ALPN at first, it will have
        // started 3 connections. But, the server above will only handle 1,
        // so the unwrapped responses futures show it still worked.
        assert_eq!(connects.load(Ordering::SeqCst), 3);

        let res4 = client.get(url.clone());
        rt.block_on(res4).unwrap();

        assert_eq!(connects.load(Ordering::SeqCst), 3, "after ALPN, no more connects");
        drop(client);
    }


    struct DebugConnector {
        http: HttpConnector,
        closes: mpsc::Sender<()>,
        connects: Arc<AtomicUsize>,
        is_proxy: bool,
        alpn_h2: bool,
    }

    impl DebugConnector {
        fn new() -> DebugConnector {
            let http = HttpConnector::new(1);
            let (tx, _) = mpsc::channel(10);
            DebugConnector::with_http_and_closes(http, tx)
        }

        fn with_http_and_closes(http: HttpConnector, closes: mpsc::Sender<()>) -> DebugConnector {
            DebugConnector {
                http: http,
                closes: closes,
                connects: Arc::new(AtomicUsize::new(0)),
                is_proxy: false,
                alpn_h2: false,
            }
        }

        fn proxy(mut self) -> Self {
            self.is_proxy = true;
            self
        }
    }

    impl Connect for DebugConnector {
        type Transport = DebugStream;
        type Error = io::Error;
        type Future = Box<dyn Future<Item = (DebugStream, Connected), Error = io::Error> + Send>;

        fn connect(&self, dst: Destination) -> Self::Future {
            self.connects.fetch_add(1, Ordering::SeqCst);
            let closes = self.closes.clone();
            let is_proxy = self.is_proxy;
            let is_alpn_h2 = self.alpn_h2;
            Box::new(self.http.connect(dst).map(move |(s, mut c)| {
                if is_alpn_h2 {
                    c = c.negotiated_h2();
                }
                (DebugStream(s, closes), c.proxy(is_proxy))
            }))
        }
    }

    struct DebugStream(TcpStream, mpsc::Sender<()>);

    impl Drop for DebugStream {
        fn drop(&mut self) {
            let _ = self.1.try_send(());
        }
    }

    impl Write for DebugStream {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.0.flush()
        }
    }

    impl AsyncWrite for DebugStream {
        fn shutdown(&mut self) -> futures::Poll<(), io::Error> {
            AsyncWrite::shutdown(&mut self.0)
        }

        fn write_buf<B: ::bytes::Buf>(&mut self, buf: &mut B) -> futures::Poll<usize, io::Error> {
            self.0.write_buf(buf)
        }
    }

    impl Read for DebugStream {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.0.read(buf)
        }
    }

    impl AsyncRead for DebugStream {}
}

mod conn {
    use std::io::{self, Read, Write};
    use std::net::TcpListener;
    use std::thread;
    use std::time::Duration;

    use futures::{Async, Future, Poll, Stream};
    use futures::future::poll_fn;
    use futures::sync::oneshot;
    use futures_timer::Delay;
    use tokio::runtime::current_thread::Runtime;
    use tokio_tcp::TcpStream;
    use tokio_io::{AsyncRead, AsyncWrite};

    use hyper::{self, Request, Body, Method};
    use hyper::client::conn;

    use super::{s, tcp_connect, FutureHyperExt};

    #[test]
    fn get() {
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = server.local_addr().unwrap();
        let mut rt = Runtime::new().unwrap();

        let (tx1, rx1) = oneshot::channel();

        thread::spawn(move || {
            let mut sock = server.accept().unwrap().0;
            sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            sock.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut buf = [0; 4096];
            let n = sock.read(&mut buf).expect("read 1");

            // Notably:
            // - Just a path, since just a path was set
            // - No host, since no host was set
            let expected = "GET /a HTTP/1.1\r\n\r\n";
            assert_eq!(s(&buf[..n]), expected);

            sock.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n").unwrap();
            let _ = tx1.send(());
        });

        let tcp = rt.block_on(tcp_connect(&addr)).unwrap();

        let (mut client, conn) = rt.block_on(conn::handshake(tcp)).unwrap();

        rt.spawn(conn.map(|_| ()).map_err(|e| panic!("conn error: {}", e)));

        let req = Request::builder()
            .uri("/a")
            .body(Default::default())
            .unwrap();
        let res = client.send_request(req).and_then(move |res| {
            assert_eq!(res.status(), hyper::StatusCode::OK);
            res.into_body().concat2()
        });
        let rx = rx1.expect("thread panicked");

        let timeout = Delay::new(Duration::from_millis(200));
        let rx = rx.and_then(move |_| timeout.expect("timeout"));
        rt.block_on(res.join(rx).map(|r| r.0)).unwrap();
    }

    #[test]
    fn incoming_content_length() {
        use hyper::body::Payload;

        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = server.local_addr().unwrap();
        let mut rt = Runtime::new().unwrap();

        let (tx1, rx1) = oneshot::channel();

        thread::spawn(move || {
            let mut sock = server.accept().unwrap().0;
            sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            sock.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut buf = [0; 4096];
            let n = sock.read(&mut buf).expect("read 1");

            let expected = "GET / HTTP/1.1\r\n\r\n";
            assert_eq!(s(&buf[..n]), expected);

            sock.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello").unwrap();
            let _ = tx1.send(());
        });

        let tcp = rt.block_on(tcp_connect(&addr)).unwrap();

        let (mut client, conn) = rt.block_on(conn::handshake(tcp)).unwrap();

        rt.spawn(conn.map(|_| ()).map_err(|e| panic!("conn error: {}", e)));

        let req = Request::builder()
            .uri("/")
            .body(Default::default())
            .unwrap();
        let res = client.send_request(req).and_then(move |mut res| {
            assert_eq!(res.status(), hyper::StatusCode::OK);
            assert_eq!(res.body().content_length(), Some(5));
            assert!(!res.body().is_end_stream());
            loop {
                let chunk = res.body_mut().poll_data().unwrap();
                match chunk {
                    Async::Ready(Some(chunk)) => {
                        assert_eq!(chunk.len(), 5);
                        break;
                    }
                    _ => continue
                }
            }
            res.into_body().concat2()
        });
        let rx = rx1.expect("thread panicked");

        let timeout = Delay::new(Duration::from_millis(200));
        let rx = rx.and_then(move |_| timeout.expect("timeout"));
        rt.block_on(res.join(rx).map(|r| r.0)).unwrap();
    }

    #[test]
    fn aborted_body_isnt_completed() {
        let _ = ::pretty_env_logger::try_init();
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = server.local_addr().unwrap();
        let mut rt = Runtime::new().unwrap();

        let (tx, rx) = oneshot::channel();
        let server = thread::spawn(move || {
            let mut sock = server.accept().unwrap().0;
            sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            sock.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
            let expected = "POST / HTTP/1.1\r\ntransfer-encoding: chunked\r\n\r\n5\r\nhello\r\n";
            let mut buf = vec![0; expected.len()];
            sock.read_exact(&mut buf).expect("read 1");
            assert_eq!(s(&buf), expected);

            let _ = tx.send(());

            assert_eq!(sock.read(&mut buf).expect("read 2"), 0);
        });

        let tcp = rt.block_on(tcp_connect(&addr)).unwrap();

        let (mut client, conn) = rt.block_on(conn::handshake(tcp)).unwrap();

        rt.spawn(conn.map(|_| ()).map_err(|e| panic!("conn error: {}", e)));

        let (mut sender, body) = Body::channel();
        let sender = thread::spawn(move || {
            sender.send_data("hello".into()).ok().unwrap();
            rx.wait().unwrap();
            sender.abort();
        });

        let req = Request::builder()
            .method(Method::POST)
            .uri("/")
            .body(body)
            .unwrap();
        let res = client.send_request(req);
        rt.block_on(res).unwrap_err();

        server.join().expect("server thread panicked");
        sender.join().expect("sender thread panicked");
    }

    #[test]
    fn uri_absolute_form() {
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = server.local_addr().unwrap();
        let mut rt = Runtime::new().unwrap();

        let (tx1, rx1) = oneshot::channel();

        thread::spawn(move || {
            let mut sock = server.accept().unwrap().0;
            sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            sock.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut buf = [0; 4096];
            let n = sock.read(&mut buf).expect("read 1");

            // Notably:
            // - Still no Host header, since it wasn't set
            let expected = "GET http://hyper.local/a HTTP/1.1\r\n\r\n";
            assert_eq!(s(&buf[..n]), expected);

            sock.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n").unwrap();
            let _ = tx1.send(());
        });

        let tcp = rt.block_on(tcp_connect(&addr)).unwrap();

        let (mut client, conn) = rt.block_on(conn::handshake(tcp)).unwrap();

        rt.spawn(conn.map(|_| ()).map_err(|e| panic!("conn error: {}", e)));

        let req = Request::builder()
            .uri("http://hyper.local/a")
            .body(Default::default())
            .unwrap();

        let res = client.send_request(req).and_then(move |res| {
            assert_eq!(res.status(), hyper::StatusCode::OK);
            res.into_body().concat2()
        });
        let rx = rx1.expect("thread panicked");

        let timeout = Delay::new(Duration::from_millis(200));
        let rx = rx.and_then(move |_| timeout.expect("timeout"));
        rt.block_on(res.join(rx).map(|r| r.0)).unwrap();
    }

    #[test]
    fn http1_conn_coerces_http2_request() {
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = server.local_addr().unwrap();
        let mut rt = Runtime::new().unwrap();

        let (tx1, rx1) = oneshot::channel();

        thread::spawn(move || {
            let mut sock = server.accept().unwrap().0;
            sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            sock.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut buf = [0; 4096];
            let n = sock.read(&mut buf).expect("read 1");

            // Not HTTP/2, nor panicked
            let expected = "GET /a HTTP/1.1\r\n\r\n";
            assert_eq!(s(&buf[..n]), expected);

            sock.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n").unwrap();
            let _ = tx1.send(());
        });

        let tcp = rt.block_on(tcp_connect(&addr)).unwrap();

        let (mut client, conn) = rt.block_on(conn::handshake(tcp)).unwrap();

        rt.spawn(conn.map(|_| ()).map_err(|e| panic!("conn error: {}", e)));

        let req = Request::builder()
            .uri("/a")
            .version(hyper::Version::HTTP_2)
            .body(Default::default())
            .unwrap();

        let res = client.send_request(req).and_then(move |res| {
            assert_eq!(res.status(), hyper::StatusCode::OK);
            res.into_body().concat2()
        });
        let rx = rx1.expect("thread panicked");

        let timeout = Delay::new(Duration::from_millis(200));
        let rx = rx.and_then(move |_| timeout.expect("timeout"));
        rt.block_on(res.join(rx).map(|r| r.0)).unwrap();
    }

    #[test]
    fn pipeline() {
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = server.local_addr().unwrap();
        let mut rt = Runtime::new().unwrap();

        let (tx1, rx1) = oneshot::channel();

        thread::spawn(move || {
            let mut sock = server.accept().unwrap().0;
            sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            sock.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut buf = [0; 4096];
            sock.read(&mut buf).expect("read 1");
            sock.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n").unwrap();

            let _ = tx1.send(());
        });

        let tcp = rt.block_on(tcp_connect(&addr)).unwrap();

        let (mut client, conn) = rt.block_on(conn::handshake(tcp)).unwrap();

        rt.spawn(conn.map(|_| ()).map_err(|e| panic!("conn error: {}", e)));

        let req = Request::builder()
            .uri("/a")
            .body(Default::default())
            .unwrap();
        let res1 = client.send_request(req).and_then(move |res| {
            assert_eq!(res.status(), hyper::StatusCode::OK);
            res.into_body().concat2()
        });

        // pipelined request will hit NotReady, and thus should return an Error::Cancel
        let req = Request::builder()
            .uri("/b")
            .body(Default::default())
            .unwrap();
        let res2 = client.send_request(req)
            .then(|result| {
                let err = result.expect_err("res2");
                assert!(err.is_canceled(), "err not canceled, {:?}", err);
                Ok(())
            });

        let rx = rx1.expect("thread panicked");

        let timeout = Delay::new(Duration::from_millis(200));
        let rx = rx.and_then(move |_| timeout.expect("timeout"));
        rt.block_on(res1.join(res2).join(rx).map(|r| r.0)).unwrap();
    }

    #[test]
    fn upgrade() {
        use tokio_io::io::{read_to_end, write_all};
        let _ = ::pretty_env_logger::try_init();

        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = server.local_addr().unwrap();
        let mut rt = Runtime::new().unwrap();

        let (tx1, rx1) = oneshot::channel();

        thread::spawn(move || {
            let mut sock = server.accept().unwrap().0;
            sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            sock.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut buf = [0; 4096];
            sock.read(&mut buf).expect("read 1");
            sock.write_all(b"\
                HTTP/1.1 101 Switching Protocols\r\n\
                Upgrade: foobar\r\n\
                \r\n\
                foobar=ready\
            ").unwrap();
            let _ = tx1.send(());

            let n = sock.read(&mut buf).expect("read 2");
            assert_eq!(&buf[..n], b"foo=bar");
            sock.write_all(b"bar=foo").expect("write 2");
        });

        let tcp = rt.block_on(tcp_connect(&addr)).unwrap();

        let io = DebugStream {
            tcp: tcp,
            shutdown_called: false,
        };

        let (mut client, mut conn) = rt.block_on(conn::handshake(io)).unwrap();

        {
            let until_upgrade = poll_fn(|| {
                conn.poll_without_shutdown()
            });

            let req = Request::builder()
                .uri("/a")
                .body(Default::default())
                .unwrap();
            let res = client.send_request(req).and_then(move |res| {
                assert_eq!(res.status(), hyper::StatusCode::SWITCHING_PROTOCOLS);
                assert_eq!(res.headers()["Upgrade"], "foobar");
                res.into_body().concat2()
            });

            let rx = rx1.expect("thread panicked");

            let timeout = Delay::new(Duration::from_millis(200));
            let rx = rx.and_then(move |_| timeout.expect("timeout"));
            rt.block_on(until_upgrade.join(res).join(rx).map(|r| r.0)).unwrap();

            // should not be ready now
            rt.block_on(poll_fn(|| {
                assert!(client.poll_ready().unwrap().is_not_ready());
                Ok::<_, ()>(Async::Ready(()))
            })).unwrap();
        }

        let parts = conn.into_parts();
        let io = parts.io;
        let buf = parts.read_buf;

        assert_eq!(buf, b"foobar=ready"[..]);
        assert!(!io.shutdown_called, "upgrade shouldn't shutdown AsyncWrite");
        assert!(client.poll_ready().is_err());

        let io = rt.block_on(write_all(io, b"foo=bar")).unwrap().0;
        let vec = rt.block_on(read_to_end(io, vec![])).unwrap().1;
        assert_eq!(vec, b"bar=foo");
    }

    #[test]
    fn connect_method() {
        use tokio_io::io::{read_to_end, write_all};
        let _ = ::pretty_env_logger::try_init();

        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = server.local_addr().unwrap();
        let mut rt = Runtime::new().unwrap();

        let (tx1, rx1) = oneshot::channel();

        thread::spawn(move || {
            let mut sock = server.accept().unwrap().0;
            sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            sock.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut buf = [0; 4096];
            sock.read(&mut buf).expect("read 1");
            sock.write_all(b"\
                HTTP/1.1 200 OK\r\n\
                \r\n\
                foobar=ready\
            ").unwrap();
            let _ = tx1.send(());

            let n = sock.read(&mut buf).expect("read 2");
            assert_eq!(&buf[..n], b"foo=bar", "sock read 2 bytes");
            sock.write_all(b"bar=foo").expect("write 2");
        });

        let tcp = rt.block_on(tcp_connect(&addr)).unwrap();

        let io = DebugStream {
            tcp: tcp,
            shutdown_called: false,
        };

        let (mut client, mut conn) = rt.block_on(conn::handshake(io)).unwrap();

        {
            let until_tunneled = poll_fn(|| {
                conn.poll_without_shutdown()
            });

            let req = Request::builder()
                .method("CONNECT")
                .uri(addr.to_string())
                .body(Default::default())
                .unwrap();
            let res = client.send_request(req)
                .and_then(move |res| {
                    assert_eq!(res.status(), hyper::StatusCode::OK);
                    res.into_body().concat2()
                })
                .map(|body| {
                    assert_eq!(body.as_ref(), b"");
                });

            let rx = rx1.expect("thread panicked");

            let timeout = Delay::new(Duration::from_millis(200));
            let rx = rx.and_then(move |_| timeout.expect("timeout"));
            rt.block_on(until_tunneled.join(res).join(rx).map(|r| r.0)).unwrap();

            // should not be ready now
            rt.block_on(poll_fn(|| {
                assert!(client.poll_ready().unwrap().is_not_ready());
                Ok::<_, ()>(Async::Ready(()))
            })).unwrap();
        }

        let parts = conn.into_parts();
        let io = parts.io;
        let buf = parts.read_buf;

        assert_eq!(buf, b"foobar=ready"[..]);
        assert!(!io.shutdown_called, "tunnel shouldn't shutdown AsyncWrite");
        assert!(client.poll_ready().is_err());

        let io = rt.block_on(write_all(io, b"foo=bar")).unwrap().0;
        let vec = rt.block_on(read_to_end(io, vec![])).unwrap().1;
        assert_eq!(vec, b"bar=foo");
    }


    #[test]
    fn http2_detect_conn_eof() {
        use futures::future;
        use hyper::{Response, Server};
        use hyper::service::service_fn_ok;
        use tokio::timer::Delay;

        let _ = pretty_env_logger::try_init();

        let mut rt = Runtime::new().unwrap();

        let server = Server::bind(&([127, 0, 0, 1], 0).into())
            .http2_only(true)
            .serve(|| service_fn_ok(|_req| {
                Response::new(Body::empty())
            }));
        let addr = server.local_addr();
        let (shdn_tx, shdn_rx) = oneshot::channel();
        rt.spawn(server.with_graceful_shutdown(shdn_rx).map_err(|e| panic!("server error: {:?}", e)));

        let io = rt.block_on(tcp_connect(&addr)).expect("tcp connect");
        let (mut client, conn) = rt.block_on(
            conn::Builder::new().http2_only(true).handshake::<_, Body>(io)
        ).expect("http handshake");
        rt.spawn(conn.map_err(|e| panic!("client conn error: {:?}", e)));


        // Sanity check that client is ready
        rt.block_on(future::poll_fn(|| client.poll_ready())).expect("client poll ready sanity");

        let req = Request::builder()
            .uri(format!("http://{}/", addr))
            .body(Body::empty())
            .expect("request builder");

        rt.block_on(client.send_request(req)).expect("req1 send");

        // Sanity check that client is STILL ready
        rt.block_on(future::poll_fn(|| client.poll_ready())).expect("client poll ready after");

        // Trigger the server shutdown...
        let _ = shdn_tx.send(());

        // Allow time for graceful shutdown roundtrips...
        rt.block_on(Delay::new(::std::time::Instant::now() + Duration::from_millis(100)).map_err(|e| panic!("delay error: {:?}", e))).expect("delay");

        // After graceful shutdown roundtrips, the client should be closed...
        rt.block_on(future::poll_fn(|| client.poll_ready())).expect_err("client should be closed");
    }

    struct DebugStream {
        tcp: TcpStream,
        shutdown_called: bool,
    }

    impl Write for DebugStream {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.tcp.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.tcp.flush()
        }
    }

    impl AsyncWrite for DebugStream {
        fn shutdown(&mut self) -> Poll<(), io::Error> {
            self.shutdown_called = true;
            AsyncWrite::shutdown(&mut self.tcp)
        }
    }

    impl Read for DebugStream {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.tcp.read(buf)
        }
    }

    impl AsyncRead for DebugStream {}
}

trait FutureHyperExt: Future {
    fn expect<E>(self, msg: &'static str) -> Box<dyn Future<Item=Self::Item, Error=E>>;
}

impl<F> FutureHyperExt for F
where
    F: Future + 'static,
    F::Error: ::std::fmt::Debug,
{
    fn expect<E>(self, msg: &'static str) -> Box<dyn Future<Item=Self::Item, Error=E>> {
        Box::new(self.map_err(move |e| panic!("expect: {}; error={:?}", msg, e)))
    }
}

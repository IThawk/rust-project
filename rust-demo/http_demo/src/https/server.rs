//! Simple HTTPS echo service based on hyper-rustls
//!
//! First parameter is the mandatory port to use.
//! Certificate and private key are hardcoded to sample files.
// #![deny(warnings)]

extern crate futures;
extern crate hyper;
extern crate rustls;
extern crate tokio;
extern crate tokio_rustls;
extern crate tokio_tcp;
extern crate tokio_core;

use self::futures::future;
use self::futures::Stream;
use self::hyper::rt::Future;
use self::hyper::service::service_fn;
use self::hyper::{Body, Method, Request, Response, Server, StatusCode};
use rustls::internal::pemfile;
use std::{env, fs, io, sync};
use tokio_rustls::TlsAcceptor;
use tokio_rustls::server::TlsStream;
use std::result::Result::Err;
use std::string::String;
use std::option::Option::Some;
use std::result::Result::Ok;
use std::option::Option::None;
use std::vec::Vec;
use std::marker::Send;

pub fn https_server_main() {
    // Serve an echo service over HTTPS, with proper error handling.
    if let Err(e) = run_server() {
        eprintln!("FAILED: {}", e);
        std::process::exit(1);
    }
}

fn error(err: String) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err)
}

fn run_server() -> io::Result<()> {
    // First parameter is port number (optional, defaults to 1337)
    let port = match env::args().nth(1) {
        Some(ref p) => p.to_owned(),
        None => "1337".to_owned(),
    };
    let addr = format!("127.0.0.1:{}", port)
        .parse()
        .map_err(|e| error(format!("{}", e)))?;

    // Build TLS configuration.
    let tls_cfg = {
        // Load public certificate.
        let certs = load_certs("config/sample.pem")?;
        // Load private key.
        let key = load_private_key("config/sample.rsa")?;
        // Do not use client certificate authentication.
        let mut cfg = rustls::ServerConfig::new(rustls::NoClientAuth::new());
        // Select a certificate to use.
        cfg.set_single_cert(certs, key)
            .map_err(|e| error(format!("{}", e)))?;
        sync::Arc::new(cfg)
    };

    // Create a TCP listener via tokio.
    let tcp = tokio_tcp::TcpListener::bind(&addr)?;
    let tls_acceptor = TlsAcceptor::from(tls_cfg);
    // Prepare a long-running future stream to accept and serve cients.
    let tls = tcp
        .incoming()
        .and_then(move |s| tls_acceptor.accept(s))
        .then(|r| match r {
            Ok(x) => {
                println!("hava tsl {:?}",x);
                Ok::<_, io::Error>(Some(x))
            },
            Err(_e) => {
                println!("[!] Voluntary server halt due to client-connection error...");
                // Errors could be handled here, instead of server aborting. 这个地方使用Ok 就不会导致服务stop
                 Ok(None)
//                Err(_e)
            }
        })
        .filter_map(|x| {
            println!(".................{:?}",x);
            x
        });
    // Build a hyper server, which serves our custom echo service.
//    let fut = Server::builder(tls).serve(|| service_fn(echo));

    // Run the future, keep going until an error occurs.
    println!("Starting to serve on https://{}.", addr);
    let mut ss = false;
    let fut = Server::builder(tls.and_then(move |sss|{
        println!("ffffffffffffffffffffffffffffffffffff{:?}",sss);
        ss = true;
        println!("{}",ss);
        Ok(sss)
        }))
        .serve( move || {
            let a = ss;
            service_fn(move |req|{
                if a{
                    println!("true.....{:?}",req);
                    echo_err(req)
                }else{
                    println!("false.............{:?}",req);
                    echo(req)
                }


            })

        }
            );

    let mut core = tokio_core::reactor::Core::new().unwrap();
    if let Err(err) = core.run(fut) {
        println!("FAILEDdddddddddddddd: {}", err);
        std::process::exit(1)
    }
    Ok(())
}

// Future result: either a hyper body or an error.
type ResponseFuture = Box<dyn Future<Item = Response<Body>, Error = hyper::Error> + Send>;

// Custom echo service, handling two different routes and a
// catch-all 404 responder.
fn echo(req: Request<Body>) -> ResponseFuture {
    let mut response = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        // Help route.
        (&Method::GET, "/") => {
            *response.body_mut() = Body::from("Try POST /echo\n");
        }
        // Echo service route.
        (&Method::POST, "/echo") => {
            *response.body_mut() = req.into_body();
        }
        (&Method::GET, "/echo") => {
            *response.body_mut() = Body::from("Try GET /echo\n");
        }
        // Catch-all 404.
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };
    Box::new(future::ok(response))
}

fn echo_err(req: Request<Body>) -> ResponseFuture {
    let mut response = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };
    Box::new(future::ok(response))
}
// Load public certificate from file.
fn load_certs(filename: &str) -> io::Result<Vec<rustls::Certificate>> {
    // Open certificate file.
    let certfile = fs::File::open(filename)
        .map_err(|e| error(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = io::BufReader::new(certfile);

    // Load and return certificate.
    pemfile::certs(&mut reader).map_err(|_| error("failed to load certificate".into()))
}

// Load private key from file.
fn load_private_key(filename: &str) -> io::Result<rustls::PrivateKey> {
    // Open keyfile.
    let keyfile = fs::File::open(filename)
        .map_err(|e| error(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = io::BufReader::new(keyfile);

    // Load and return a single private key.
    let keys = pemfile::rsa_private_keys(&mut reader)
        .map_err(|_| error("failed to load private key".into()))?;
    if keys.len() != 1 {
        return Err(error("expected a single private key".into()));
    }
    Ok(keys[0].clone())
}

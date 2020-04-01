//use actix_http::Error;
//use actix_rt::System;
use futures::{future::lazy, Future};
use actix_web::client::Client;
use std::path::Iter;
use futures::future::Err;
use futures::Async;
use bytes::{Bytes, BytesMut};
use std::sync::Arc;
use rustls::{ClientConfig, AllowAnyAuthenticatedClient, RootCertStore, NoClientAuth};
use std::io::BufReader;
use std::fs::File;
//fn tcp_client_main() -> Result<(), Error> {
//
//    System::new("test").block_on(lazy(|| {
//        awc::Client::new()
//            .get("https://www.rust-lang.org/") // <- Create request builder
//            .header("User-Agent", "Actix-web")
//            .send() // <- Send http request
//            .from_err()
//            .and_then(|mut response| {
//                // <- server http response
//                println!("Response: {:?}", response);
//
//                // read response body
//                response
//                    .body()
//                    .from_err()
//                    .map(|body| println!("Downloaded: {:?} bytes", body.len()))
//            })
//    }))
//}


//mod danger {
//    pub struct NoCertificateVerification {}
//
//    impl ServerCertVerifier for NoCertificateVerification {
//        fn verify_server_cert(
//            &self,
//            _roots: &rustls::RootCertStore,
//            _presented_certs: &[rustls::Certificate],
//            _dns_name: webpki::DNSNameRef<'_>,
//            _ocsp: &[u8],
//        ) -> Result<rustls::ServerCertVerified, rustls::TLSError> {
//            Ok(ServerCertVerified::assertion())
//        }
//    }
//}

pub fn req_get_handle() -> impl Future<Item=(), Error=()> {
    // disable ssl verification
    let mut config = ClientConfig::new();
    let a = NoClientAuth::new();
    let protos = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    let mut client_auth_roots = RootCertStore::empty();
    let rootbuf = &mut BufReader::new(File::open("key.pem").unwrap());
    config.root_store.add_pem_file(rootbuf).unwrap();
    config.ciphersuites.clear();
    config.versions.clear();

    let client = awc::Client::build()
        .connector(awc::Connector::new().rustls(Arc::new(config)).finish())
        .finish();

    client.get("http://127.0.0.1:8080") // <- Create request builder
        .header("User-Agent", "Actix-web")
        .send()                             // <- Send http request
        .map_err(|_| ())
        .and_then(|mut response| {              // <- server http response
            println!("Response: {:?}", response);
            let a = match response.body().poll().ok().unwrap() {
                Async::Ready(bytes) => bytes,
                _ => Bytes::new(),
            };
            let a = String::from_utf8(a.to_vec()).unwrap_or_default();
            println!("{:?}", a);
            Ok(())
        })
}
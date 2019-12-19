//use actix_http::Error;
//use actix_rt::System;
use futures::{future::lazy, Future};
use actix_web::client::Client;
use std::path::Iter;
use futures::future::Err;
use futures::Async;
use bytes::{Bytes, BytesMut};
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


pub fn req_get_handle() -> impl Future<Item=(), Error=()> {
    let mut client = Client::default();
    client.get("http://127.0.0.1:8080") // <- Create request builder
        .header("User-Agent", "Actix-web")
        .send()                             // <- Send http request
        .map_err(|_| ())
        .and_then(|mut response| {              // <- server http response
            println!("Response: {:?}", response);
            let a= match response.body().poll().ok().unwrap() {
                Async::Ready(bytes) => bytes,
                _ => Bytes::new(),
            };
            let a = String::from_utf8(a.to_vec()).unwrap_or_default();
            println!("{:?}",a);
            Ok(())
        })
}
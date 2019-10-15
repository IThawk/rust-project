extern crate log;

mod http;
mod https;
mod reqwest_ex;
use std::thread;

use http::*;
use https::*;
use reqwest_ex::*;

fn main() {
//    http_client_main();
//    https_server_main();
//    thread::spawn(|| https_server_main());
//    thread::spawn(|| https_client_main());
    thread::spawn(|| reqwest_demo1_main());
    ///这个地方就是为了阻塞主线程
    loop {

    }
}
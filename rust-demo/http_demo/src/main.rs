mod http;
mod https;

use std::thread;

use http::*;
use https::*;

fn main() {
//    http_client_main();
//    https_server_main();
    thread::spawn(|| https_server_main());
    thread::spawn(|| https_client_main());
    ///这个地方就是为了阻塞主线程
    while true {}
}
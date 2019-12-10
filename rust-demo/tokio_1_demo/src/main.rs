#[macro_use]
extern crate futures;
#[macro_use]
extern crate serde_derive;

use std::thread;
use crate::example::tinydb;

mod example;

fn main() {
    thread::spawn(|| tinydb::tiny_db_main());

    while true {

    }
    println!("Hello, world!");
}

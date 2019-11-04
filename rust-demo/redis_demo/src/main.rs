extern crate redis;
mod single;
use single::*;

fn main() {
    single_redis_main();
    println!("Hello, world!");
}

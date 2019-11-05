extern crate redis;
mod single;
mod pool;

use pool::*;
use single::*;

fn main() {
    single_redis_main();
    println!("Hello, world!");
    pool_redis_main();
}

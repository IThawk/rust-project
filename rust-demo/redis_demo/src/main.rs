extern crate redis;

mod pool;
mod single;

use pool::*;
use single::*;
use std::time::{Duration, Instant};
use std::thread;

fn main() {
    //获取环境变量 Redis
    let a = std::env::var("redis").unwrap_or("hello".to_string());
    println!("{}", a);
    //    single_redis_main();
    println!("Hello, world!");
    loop {
        let a = Duration::from_secs(5);
        thread::sleep(a);
        my_redis_pool_demo::my_pool_redis_main();
    }


    //    single_pub_sub_main();
}





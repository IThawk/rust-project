extern crate redis;
mod pool;
mod single;

use pool::*;
use single::*;

fn main() {
    //获取环境变量 Redis
    let a = std::env::var("redis").unwrap_or("hello".to_string());
    println!("{}", a);
//    single_redis_main();
    println!("Hello, world!");
    //    pool_redis_main();

    //    single_pub_sub_main();
}

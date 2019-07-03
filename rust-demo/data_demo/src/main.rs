mod string;
mod hash_ex;
use  hash_ex::*;
mod arc_mutex_ex;
use arc_mutex_ex::*;

fn main() {
    hash_main();
    arc_main();
    println!("Hello, world!");

    println!("{}",string::new_string("ssss"));

//    hash_map::main();
}


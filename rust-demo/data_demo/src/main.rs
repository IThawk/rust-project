extern crate pretty_env_logger;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate chrono;
mod string;
mod hash_ex;
mod leetcode_ex;
use  hash_ex::*;
mod arc_mutex_ex;
use arc_mutex_ex::*;
mod arraystring_ex;
use arraystring_ex::*;


fn main() {
//    pretty_env_logger::init();
    init_log();
    hash_main();
    arc_main();
    array_string_main();
//    info!("Hello, world!");

//    info!("{}",string::new_string("ssss"));


}

fn init_log() {
    use chrono::Local;
    use std::io::Write;

    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
    env_logger::Builder::from_env(env)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} {} [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.module_path().unwrap_or("<unnamed>"),
                &record.args()
            )
        })
        .init();

    info!("env_logger initialized.");
}

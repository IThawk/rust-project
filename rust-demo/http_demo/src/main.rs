#[macro_use]
extern crate log;
extern crate env_logger;
extern crate chrono;
mod client;
fn main() {
    println!("Hello, world!");
    client::main();
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
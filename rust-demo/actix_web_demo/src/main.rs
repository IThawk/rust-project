mod server;
mod client;
use crate::server::http_server;


fn main() {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();
    http_server::http_main();
    loop {}
}

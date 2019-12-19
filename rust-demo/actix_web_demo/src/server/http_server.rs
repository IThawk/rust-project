use crate::client::http_client;
use futures::{IntoFuture, Future};

use actix_web::{
    get, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer,
};
use std::thread;
use futures::future::ok;

#[get("/resource1/{name}/index.html")]
fn index(req: HttpRequest, name: web::Path<String>) -> String {
//    println!("REQ: {:?}", req);
    format!("Hello: {}!\r\n", name)
}

///异步请求
fn index_async(req: HttpRequest) -> impl Future<Item=&'static str, Error=()> {
    println!("REQ: {:?}", req);
    http_client::req_get_handle().and_then(|_|{
        ok("Hello world!\r\n")
    })

}

#[get("/")]
fn no_params() -> &'static str {
    "Hello world!\r\n"
}

fn tcp_server_main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            ////这个地方是打印日志
//            .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2"))
//            .wrap(middleware::Compress::default())
//            .wrap(middleware::Logger::default())
            .service(index)
            .service(no_params)
            .service(
                web::resource("/resource2/index.html")
                    .wrap(
                        middleware::DefaultHeaders::new().header("X-Version-R2", "0.3"),
                    )
                    .default_service(
                        web::route().to(|| HttpResponse::MethodNotAllowed()),
                    )
                    .route(web::get().to_async(index_async)),
            )
            .service(web::resource("/test1.html").to(|| "Test\r\n"))
    })
        .bind("127.0.0.1:8080")?
        .workers(1)
        .run()
}

pub fn http_main() {
    let tcp_server = thread::Builder::new().name("child1".to_string()).spawn(move || {
        println!("Hello, world!");
        match tcp_server_main() {
            Ok(_) => println!("start server ok"),
            Err(e) => eprintln!("start server error :{:?}", e)
        };
    });
    if let Err(e) = tcp_server {
        eprintln!("start server error :{:?}", e);
    }
}
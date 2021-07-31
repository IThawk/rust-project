use futures::future::{ok, Future};
use redis::{Commands, Connection, RedisResult};
use std::sync::Arc;
use tokio_redis_pool::{Builder, RedisManager, RedisPool};
use tokio_resource_pool::{CheckOut, Manage, Pool, RealDependencies, Status};

pub fn pool_redis_main() {
    let pool = new_pool("redis://:123456@192.168.101.13:16379");
    task(pool);
}

fn task(pool: Arc<Pool<RedisManager>>) {
    tokio::run(
        get_string_value("test".to_string(), pool.clone()).then(move |_| {
            let b = get_string_value("test".to_string(), pool.clone()).and_then(|_| Ok(()));
            tokio::spawn(b);
            println!("11111");
            let c = get_string_value("test".to_string(), pool.clone()).and_then(|_| Ok(()));
            tokio::spawn(c);
            println!("2222");
            let d = set_string_value("test".to_string(), "test".to_string(), pool.clone())
                .and_then(|_| Ok(()));
            tokio::spawn(d);
            println!("3333");
            Ok(())
        }),
    );
}

fn new_pool(path: &str) -> Arc<Pool<RedisManager>> {
    let manager = RedisManager::new(path).unwrap();
    let pool = Builder::new().build(4, manager);
    let arc_pool = Arc::new(pool);

    arc_pool
}

fn get_string_value(
    key: String,
    pool: Arc<Pool<RedisManager>>,
) -> impl Future<Item = String, Error = ()> + Send + 'static {
    pool.check_out()
        .map_err(|e| {
            eprintln!("eee{:?}", e);
        })
        .and_then(move |connection| {
            redis::cmd("GET")
                .arg(key)
                .query_async::<_, String>(connection)
                .map(|(_, v)| v)
                .map_err(|error| {
                    eprintln!("...{:?}", error);
                })
        })
        .then(move |result| match result {
            Ok(v) => Ok(v),
            Err(e) => Err(()),
        })
}

fn set_string_value(
    key: String,
    value: String,
    pool: Arc<Pool<RedisManager>>,
) -> impl Future<Item = (), Error = ()> + Send + 'static {
    pool.check_out()
        .map_err(|e| {
            eprintln!("eee{:?}", e);
        })
        .and_then(move |connection| {
            redis::cmd("SET")
                .arg(key)
                .arg(value)
                .query_async::<_, ()>(connection)
                .map(|(_, v)| v)
                .map_err(|error| {
                    eprintln!("error{:?}", error);
                })
        })
        .then(move |result| match result {
            Ok(v) => Ok(v),
            Err(e) => Err(()),
        })
}

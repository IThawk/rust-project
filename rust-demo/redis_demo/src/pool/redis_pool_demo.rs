use redis::{Commands, Connection, RedisResult};
use tokio_redis_pool::{Builder, RedisManager, RedisPool};
use futures::future::Future;
use std::sync::Arc;
use tokio_resource_pool::{CheckOut, Manage, Pool, RealDependencies, Status};

pub fn pool_redis_main() {
    let a = new_pool("redis://:123456@192.168.101.13:16379");
    let b = set_string_value("test1".to_string(), a)
        .and_then(|value| {
            println!("get value:{}",value);
            Ok(())
        });
    tokio::run(b);
}


fn new_pool(path: &str) -> Arc<Pool<RedisManager>> {
    let manager = RedisManager::new(path).unwrap();
    let pool = Builder::new().build(4, manager);
    let arc_pool = Arc::new(pool);

    arc_pool
}

fn set_string_value(key: String, pool: Arc<Pool<RedisManager>>) -> impl Future<Item=String, Error=()> + Send + 'static {
    pool.check_out()
        .map_err(|e| {
            eprintln!("eee{:?}", e);
        })
        .and_then(move |connection| {
            redis::cmd("GET")
                .arg(key).query_async::<_, String>(connection)
                .map(|(_, v)| v)
                .map_err(|error| {
                    eprintln!("{:?}", error);
                })
        }
        ).then(move |result| {
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(())
        }
    })
}

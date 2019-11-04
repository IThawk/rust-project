use redis::{Commands, Connection, RedisResult};
use tokio_redis_pool::{Builder, RedisManager, RedisPool};
use futures::future::Future;

pub fn pool_redis_main() {
    if let Ok(mut conn) = new_pool("redis://:123456@192.168.101.13:16379") {
        set_string_value("test1", "test1", &mut conn);
    };
}


fn new_pool(path: &str) -> RedisResult<(RedisPool)> {
    let manager = RedisManager::new(path).unwrap();
    let pool = Builder::new().build(4, manager);

    /* do something here */

    Ok((pool))
}

fn set_string_value(key: &str, value: &str, pool: &mut RedisPool) -> Result<(), String> {
    tokio::run(
        pool
            .check_out()
            .and_then(|connection| redis::cmd("INFO").query_async::<_, redis::InfoDict>(connection))
            .map(|(_checked_out_connection, info)| println!("{:#?}", info))
            .map_err(|error| eprintln!("{}", error)),
    );
    Ok(())
}

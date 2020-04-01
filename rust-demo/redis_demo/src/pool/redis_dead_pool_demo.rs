use std::env;

use deadpool_redis::{Manager, Pool};
use futures::compat::Future01CompatExt;
use redis::FromRedisValue;
use futures::future::{Future, ok};

fn main() -> impl Future<Item=String, Error=()> + Send + 'static {
    let mgr = Manager::new("redis://127.0.0.1/").unwrap();
    let pool = Pool::new(mgr, 16);
//    {
//        let mut conn = pool.get().await.unwrap();
//        let mut cmd = redis::cmd("SET");
//        cmd.arg(&["deadpool/test_key", "42"]);
//        conn.query(&cmd).await.unwrap();
//    }

    pool.get().unwrap().and_then(|cc| {
        let mut cmd = redis::cmd("GET");
        cmd.arg(&["deadpool/test_key"]);
        cc.query(&cmd).map_err(|e| eprintln!("dddd", e)).and_then(|cc| {
            ok(cc)
        })
    })
}
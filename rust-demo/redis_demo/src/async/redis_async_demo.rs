use redis::{Commands, Connection, RedisResult};

pub fn single_redis_main() {
    if let Ok(mut conn) = get_connection("redis://:123456@192.168.101.13:16379") {
        set_string_value("test1", "test1", &mut conn);
    };
}

fn get_connection(path: &str) -> RedisResult<(Connection)> {
    let client = redis::Client::open(path)?;
    let mut con = client.get_connection()?;

    /* do something here */

    Ok((con))
}

fn set_string_value(key: &str, value: &str, con: &mut Connection) -> Result<(), String> {
    match con.set(key, value) {
        Ok(()) => Ok(()),
        Err(e) => {
            let a = format!("set key:{},value:{},error:{:?}", key, value, e);
            Err(a)
        }
    }
}

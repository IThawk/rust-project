use futures::{try_ready, Async, Future, Poll};
use redis::aio::{Connection, ConnectionLike};
use redis::{IntoConnectionInfo, RedisError, RedisFuture, RedisResult};
pub use tokio_resource_pool::{Builder, CheckOutFuture};
use tokio_resource_pool::{CheckOut, Manage, Pool, RealDependencies, Status};
use std::time::Duration;
use std::thread;

/// Manages the lifecycle of connections to a single Redis server.
pub struct RedisManager {
    client: redis::Client,
}

impl RedisManager {
    /// Creates a new `RedisManager` from anything that can be converted to a `ConnectionInfo`.
    ///
    /// # Example
    ///
    /// ```
    /// # use tokio_redis_pool::RedisManager;
    /// # use redis::RedisResult;
    /// # fn main() -> RedisResult<()> {
    /// let manager = RedisManager::new("redis://127.0.0.1:6379")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(params: impl IntoConnectionInfo) -> RedisResult<Self> {
        let client = redis::Client::open(params)?;
        Ok(Self { client })
    }
}

impl Manage for RedisManager {
    type Resource = Connection;

    type Dependencies = RealDependencies;

    type CheckOut = RedisCheckOut;

    type Error = RedisError;

    type CreateFuture = Box<dyn Future<Item=Self::Resource, Error=Self::Error> + Send>;

    fn create(&self) -> Self::CreateFuture {
        Box::new(self.client.get_async_connection().and_then(|conn| {
            test_create(conn)
        }))
    }

    fn status(&self, _: &Self::Resource) -> Status {
        Status::Valid
    }

    type RecycleFuture = RecycleFuture;

    fn recycle(&self, connection: Self::Resource) -> Self::RecycleFuture {
        let inner = redis::cmd("PING").query_async::<_, ()>(connection);
        RecycleFuture { inner }
    }
}

fn test_create(connection: Connection) -> impl Future<Item=Connection, Error=RedisError> + Send + 'static {
    redis::cmd("GET")
        .arg("test")
        .query_async::<Connection, String>(connection)
        .and_then(|(conn, value)| {
            println!("create conn.....{}",value);
            let a = Duration::from_secs(10);
            thread::sleep(a);
            println!("create conn...end..{}",value);
            Ok(conn)
        })
}


/// A resource `Pool` specialized for Redis connections.
pub type RedisPool = Pool<RedisManager>;

/// A check out of a Redis connection from the pool.
///
/// It implements `ConnectionLike`, so you can pass it directly to functions such as
/// `Pipeline::query`.
///
/// # Example
///
/// ```
/// # use futures::future::Future;
/// # use redis::RedisResult;
/// # use tokio_redis_pool::{Builder, RedisManager, RedisPool};
/// # let manager = RedisManager::new("redis://127.0.0.1:6379").unwrap();
/// # let pool = Builder::new().build(4, manager);
/// tokio::run(
///     pool
///         .check_out()
///         .and_then(|connection| redis::cmd("INFO").query_async::<_, redis::InfoDict>(connection))
///         .map(|(_checked_out_connection, info)| println!("{:#?}", info))
///         .map_err(|error| eprintln!("{}", error)),
/// );
/// ```
pub struct RedisCheckOut {
    inner: CheckOut<RedisManager>,
}

impl ConnectionLike for RedisCheckOut {
    fn req_packed_command(
        self,
        cmd: Vec<u8>,
    ) -> Box<dyn Future<Item=(Self, redis::Value), Error=RedisError> + Send> {
        let borrower = move |connection: Connection| connection.req_packed_command(cmd);
        Box::new(self.inner.lend(borrower))
    }

    fn req_packed_commands(
        self,
        cmd: Vec<u8>,
        offset: usize,
        count: usize,
    ) -> Box<dyn Future<Item=(Self, Vec<redis::Value>), Error=RedisError> + Send> {
        let borrower =
            move |connection: Connection| connection.req_packed_commands(cmd, offset, count);
        Box::new(self.inner.lend(borrower))
    }

    fn get_db(&self) -> i64 {
        self.inner.get_db()
    }
}

impl From<CheckOut<RedisManager>> for RedisCheckOut {
    fn from(inner: CheckOut<RedisManager>) -> Self {
        Self { inner }
    }
}

pub struct RecycleFuture {
    inner: RedisFuture<(Connection, ())>,
}

impl Future for RecycleFuture {
    type Item = Option<Connection>;

    type Error = RedisError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let (connection, ()) = try_ready!(self.inner.poll());
        Ok(Async::Ready(Some(connection)))
    }
}

#[cfg(test)]
mod tests {
    use super::{Builder, RedisManager};
    use futures::future::Future;

    #[test]
    fn checkout_and_read() {
        let redis_url = "redis://127.0.0.1/";
        {
            let mut client = redis::Client::open(redis_url).unwrap();
            redis::cmd("HSET")
                .arg("greetings")
                .arg("english")
                .arg("hello")
                .execute(&mut client);
        }

        let manager = RedisManager::new(redis_url).unwrap();
        let pool = Builder::new().build(4, manager);
        let mut runtime = tokio::runtime::Runtime::new().unwrap();

        let result = runtime.block_on(pool.check_out().and_then(|connection| {
            redis::cmd("HGET")
                .arg("greetings")
                .arg("english")
                .query_async::<_, String>(connection)
                .map(|(_, v)| v)
        }));
        assert_eq!(Ok("hello".to_string()), result);
    }
}

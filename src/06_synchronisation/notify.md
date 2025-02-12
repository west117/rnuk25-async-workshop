# Notify

The tokio docs suggest that you should use an async mutex to guard resources like a database connection.
I still disagree with this, although it is reasonable if you cannot afford any kind of refactor.
That said, I would still implore you to try and get a proper connection pool resource.

Writing your own connection pool is quite easy, if you need one. Simply, you might think of a pool
as a `VecDeque<T>`. You can `pop_front` from the pool to get access to a connection, and then use `push_back` to re-queue the connection. To have this between threads, you obviously need a Mutex to protect this vec. However, if there are no connections ready in the pool, how do you wait for one?

One less elegant way is to use a semaphore, Make sure the semaphore has the same number of permits as
connections. Before you lock the mutex, acquire a permit for it. This works fine, and I would not
mind if I saw this in production. That said, I think this is a great opportunity to show off one of my
favourite synchronisation primitives in tokio.

If all you need is a notification, how about a primitive called `Notify`?

```rust
struct Pool<T> {
    conns: Mutex<VecDeque<T>>,
    notifications: Notify,
}

impl<T> Pool<T> {
    pub async fn acquire(&self) -> Conn<'_, T> {
        // register our interest
        self.notifications.notified().await;

        // a value is now ready
        let mut conns = self.pool.lock().unwrap();

        let conn = conns.pop_front().expect("a conn should exist");

        // more connections ready, store a notification
        if !conns.is_empty() {
            self.notifications.notify_one();
        }

        Conn {
            pool: self,
            conn: Some(conn)
        }
    }

    pub fn insert_conn(&self, conn: T) {
        // insert the conn into the pool
        let mut conns = self.pool.lock().unwrap();
        conns.push_back(conn);

        // notify the next task that a connection is now ready
        self.pool.notifications.notify_one();
    }
}

struct Conn<'a, T> {
    pool: &'a Pool<T>,
    // option needed to return it to the pool on drop later
    // it will always be Some
    conn: Option<T>,
}

// return the conn on drop.
impl<T> Drop for Conn<'_, T> {
    fn drop(&mut self) {
        let conn = self.conn.take().unwrap();
        self.pool.insert_conn(conn);
    }
}
```

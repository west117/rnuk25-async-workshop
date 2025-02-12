# Chapter 1 - Actor Model

People often ask me what my favourite actor framework in Rust is. They are often surprised when I tell
them tokio!

Quoting Wikipedia:
> An actor is a computational entity that, in response to a message it receives, can concurrently:
> * send a finite number of messages to other actors;
> * create a finite number of new actors;
> * designate the behavior to be used for the next message it receives.

There are many actor frameworks for rust:
* [`kameo`](https://docs.rs/kameo/latest/kameo/)
* [`ractor`](https://docs.rs/ractor/latest/ractor/)
* [`actix`](https://docs.rs/actix/latest/actix/)
* [`elfo`](https://docs.rs/elfo/latest/elfo/)

Looking at kameo, we see

---

### Defining an Actor

```rust,ignore
use kameo::Actor;
use kameo::message::{Context, Message};

// Implement the actor
#[derive(Actor)]
struct Counter {
    count: i64,
}

// Define message
struct Inc { amount: i64 }

// Implement message handler
impl Message<Inc> for Counter {
    type Reply = i64;

    async fn handle(&mut self, msg: Inc, _ctx: Context<'_, Self, Self::Reply>) -> Self::Reply {
        self.count += msg.amount;
        self.count
    }
}
```

### Spawning and Interacting with the Actor

```rust,ignore
// Spawn the actor and obtain its reference
let actor_ref = kameo::spawn(Counter { count: 0 });

// Send messages to the actor
let count = actor_ref.ask(Inc { amount: 42 }).await?;
assert_eq!(count, 42);
```

---

## Actors and Tokio

So, what does this have to do with tokio? Actor programming is a nice paradigm but
as someone who is used to working with threads, tasks end up feeling more natural.
However, it turns out that these paradigms are not so different.

We need a way to send messages to other actors. Fortunately, we have
an `mpsc` (Multi-produce; single-consumer) channel at our disposal.
mpsc is perfect for our needs as we want many actors to be able to send
messages, but only the one actor needs to receive them.

We need a way to spawn new actors. Fortunately, we have also a `spawn` method
also at our disposal.

Lastly, we just need a way to define the behaviour to take when a certain message is received.
For this we can use Rust `enum`s and `match` to choose the behaviour.

An additional step we might want to consider is how actor replies work.
Either the request includes a request ID and mailbox and the reply can be sent as
a normal message, but from looking at existing frameworks, it seems more common to have a designated reply system.
Thankfully tokio also offers a `oneshot` channel, perfect for one-off messages (such a replies).

## Why you might still want actors

Actors are not a way to implement concurrent high-scale applications. They are a way to _structure_ applications.
Conveniently, this structure can scale to multi-node clusters where `tokio` cannot out of the box.

Ultimately with a multi-node setup you might be looking at a container orchaestrator like kubernetes to manage
multiple processes on multple nodes, and then a message queue like kafka to manage sending messages to different nodes,
where that node can then process the message using tokio. If you have a good actor framework, you might get this
out of the box.

If you have a great actor framework, it might even be durable and allow synchronising state between nodes to allow actors
to survive a node failure, although some state will inevitably be un-syncable (you cannot send a websocket connection to another node).

However, it is my opinion that the actor model is one you can still follow without needing to use an actor framework. We
will take a look at this in the next chapter.

# Chunking

Not all bytes are the same. If you are working with TCP or QUIC, you get to work with byte "streams".
Byte streams have no hard boundaries about the bytes you get. You could ask for 1 byte or you could ask for 10000 bytes.

Websockets or HTTP2, however, are "frame" based. You send and receive discrete chunks of bytes, and these
could be used to define individual messages.

Because of these differences, there's often different APIs when working with them.

For instance, The `tokio-tungstenite` crate for websockets exposes a `WebSocketStream` which is a `Stream<Item = Result<Message, Error>> + Sink<Message, Error = Error>`. `Sink` is something we haven't looked at yet, but it effectively abstracts a channel sender, where `Stream` is like the channel receiver.

`hyper` and by extension `axum` use a custom `http::Body` trait in requests/responses. This is similar to a `Stream<Item = Result<Frame<Buf>, Error>>`, but is designed specifically with http in mind. There are helpers in the `http-body-util` crate
that allow freely converting between `Body` and `Stream`, though.

Each of these only offer a way to receive the entire message only, no partial messages. `Body::next_frame()` always returns
a single frame. `WebSocketStream::next()` only returns a single message.

However, `tokio::net::TcpStream` and `quinn::RecvStream`/`quinn::SendStream` all work over the `tokio::io::AsyncRead`/`tokio::io::AsyncWrite` trait, which allows partial reading and writing.

## Excess Buffering

One problem that arises with these different abstractions is that layering them can cause excess buffering.

TCP is based on IP packets. IP packets are framed and have a maximum size depending on the
maximum transmission unit (MTU) of your network. When you write to a TCP stream, the data is split into buckets
for each IP packet to be sent. These are then buffered by your OS for re-delivery incase these IP packets get lost.

Let's say you use TLS on your connection. TLS sends data as frames, since each encrypted block of bytes needs some extra
metadata - like the authentication tag. Because of this, the TLS wrapper needs to buffer data from the TCP stream
until it can read a full TLS frame. `tokio_rustls::TlsStream` exposes this data back as a `tokio::io::AsyncRead` interface.

On top of this you might have a `WebSocketStream`. Since websocket messages are framed, if a large websocket message
is sent, then the websocket stream will need to buffer the data from the TLS stream until it has enough to
process the entire websocket message into one `Vec<u8>` to give you.

### Problems this causes

This manifested as a problem at Neon for the service I work on. Postgres uses its own TCP/TLS based protocol, but
we found that not all environments (edge/serverless) supports raw TCP. They did however support websockets.
We decided to implement a simple middleware client that turns the postgres byte stream into websocket messages,
then my service takes those websocket messages, and interprets them as bytes.

Postgres is itself a framed based protocol, so we had an extra layer of buffering. This caused a messurable amount
of lag as messages would need to fill each buffer before being processed. Notably, if a client sent a large
query in a single websocket message, then we would need to buffer the entire message until we could start then sending it
to postgres.

I fixed this by [forking and re-writing](https://github.com/neondatabase/framed-websockets) the websocket server implementation we were using. Fortunately, websockets has a concept of "partial messages", so in the server layer, I managed to modify it
to split a large message into smaller chunks, which reduced the amount of buffering needed significantly.

An additional optimsation could be sharing the buffer needed with `rustls`'s upcoming "unbuffered" API. This API
no longer has a buffer of its own, and asks the user to provide their own buffer. You could then use this same buffer as the one used for websocket processing.

# Codec

As I've eluded to, the framed/chunked/message based APIs of Websockets or HTTP2, or even TLS for that matter,
will end up being sent over a byte-stream write. Similarly, a byte-stream reader will end up being buffered then parsed
into messages.

Because this is so common in many protocols (HTTP2, TLS, Postgres, MySQL, Websockets), I want to share a great utility
provided by `tokio-util` called `codec`.

### Sending data

```rust
let mut stream = TcpStream::connect(...).await?;

// create a new buffer
let mut buf = BytesMut::new();

// receive some message from a stream
while let Some(msg) = recv_msg().await {
    // encode the message into our buffer
    msg.encode_into(&mut buf);

    // split the buffer in two.
    // `msg_data` contains all data written from the previous encoding
    // `buf` will now be empty
    // this will not need to allocate
    let msg_data = buf.split();

    // turn this buffer into a shared buffer
    let msg_data = msg_data.freeze();

    // send the data chunk to the tcp stream.
    stream.write_all(msg_data).await;
}
```

### Receiving data

```rust
let mut stream = TcpStream::connect(...).await?;

// create a new buffer
let mut buf = BytesMut::new();
loop {
    // read some data into our buffer
    if stream.read_buf(&mut buf).await? == 0 {
        break;
    }

    // try find where our next message separator is
    while let Some(end_of_msg) = peek_msg(&buf) {
        // split the buffer in two.
        // `msg_data` contains all the data for a single message
        // `buf` will be advanced forward to not contain that message
        // this will not need to allocate
        let msg_data = buf.split_to(end_of_msg);

        // turn this buffer into a shared buffer
        let msg_data = msg_data.freeze();

        // parse the data and process
        let msg = parse_msg(msg_data)?;
        handle(msg).await;
    }
}
```

---

[`tokio_util::codec`](https://docs.rs/tokio-util/latest/tokio_util/codec/index.html) allows you to abstract the "encode"/"peek"/"parse" APIs into an `Encoder` and `Decoder` trait. They then provide the types `FramedWrite` and `FramedRead` as appropriate
to convert from your messages/chunks/frames into a byte stream, and vice versa.

### Sending data

```rust
let stream = TcpStream::connect(...).await?;

let mut writer = FramedWrite::new(stream, MyCodec);

// receive some message from a stream
while let Some(msg) = recv_msg().await {
    writer.send(msg).await?;
}
```

### Receiving data

```rust
let stream = TcpStream::connect(...).await?;

let mut reader = FramedRead::new(stream, MyCodec);

// receive some message from a stream
while let Some(msg) = reader.try_next().await? {
    handle(msg).await;
}
```

---

Of course, I've skipped over how `MyCodec` works, but it's a couple traits to implement
that defines the functions like `encode_into` and `parse_msg` that I didn't define above,
and it helps clean up the core logic into a cleaner abstraction.

Let's see what a codec might look like for JSONLines using `serde` and `serde_json`.

If you're not familiar, JSONLines is a very simple modification to json:

```json
{"foo": "bar"}
{"foo": "baz"}
```

Each entry is on a separate line, and there are no new-lines in the individual json entry encoding.

### Encoding

Encoding data is always easier than decoding, as you don't have to program so defensively. Here's how it might look.

```rust
struct JsonLinesCodec<S>(PhantomData<S>);

impl<S: Serialize> Encoder<S> for JsonLinesCodec<S> {
    type Error = std::io::Error;

    fn encode(&mut self, item: S, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // json encode the item into the buffer
        serde_json::to_writer(dst.writer(), &item)?;
        // add new-line separator
        dst.put_u8(b'\n');
        Ok(())
    }
}
```

### Decoding

Although I suggested that decoding can be more challenging, it's still reasonably easy.

```rust
struct JsonLinesCodec<S>(PhantomData<S>);

impl<S: DeserializeOwned> Decoder for JsonLinesCodec<S> {
    type Item = S;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let Some(new_line) = src.iter().position(|&b| b == b'\n') else {
            // not enough data ready yet
            return Ok(None);
        };

        // remove full line from the buffer
        let line_end = new_line + 1;
        let json_line = src.split_to(line_end).freeze();

        // parse the json line.
        let item = serde_json::from_slice(&json_line)?;

        Ok(Some(item))
    }
}
```

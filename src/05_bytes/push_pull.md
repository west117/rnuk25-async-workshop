# Push vs Pull

Not all byte APIs are the same. At the bare minimum, we might look at `Read` and `Write` for example.
`Read` is a 'pull' based API (important, I don't mean _poll_ based API).
`Write`, meanwhile, is a 'push' based API.

When I want to read some data, I ask for some data. When I want to write some data, I send it to the writer.
This might seem trivial, but it depends on you having an active task to push data.

Imagine instead you are sending the contents of a file in a HTTP response. Your HTTP request handler
will need to read in some of the contents of the file, and then actively write it to the HTTP response.
This is all assuming you even have access to the HTTP connection, which hyper/axum don't give you.

Instead, some writing APIs end up being pull based too. Instead of axum giving you an `AsyncWrite`
interface that you push to, you instead return a `Response<Body>`, where this `Body` is a pull-based
frame stream. You could implement this over a file to read a chunk of data from the file, then frame it and
return that.

Ultimately, hyper will then convert that data pulled from your response body into pushes to the underlying TCP
stream, but the API presented is still pull based. hyper asks you for more data when it is able to send, rather
than your task waiting to push data. This isn't always super trivial to work with, however.

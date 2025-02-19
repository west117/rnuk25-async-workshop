# Graceful shutdown in hyper

Since we are using hyper for this project, I should point out that hyper has some graceful shutdown support out of the box. Since we want to partially stop the HTTP connection,
but keep processing in-flight requests, we will be forced to use it.

On the `Connection` type, there is a method conveniently called `graceful_shutdown` which we will need
to call. Calling that will stop accepting new requests but continue processing old requests.
We will need to continue awaiting the connection object.

Since this connection future isn't necessarily cancel safe,
this requires us to work with Pin, as we saw before.

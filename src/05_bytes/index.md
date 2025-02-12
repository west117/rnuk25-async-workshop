# Chapter 5 - Working with Bytes

When dealing with the network, it can often be desired to stream responses so the client can start
processing them faster, as well as reducing the amount of active memory is needed for an inflight request.

When it comes to stream processing, it usually means byte processing. However, applications
usually don't want to work with bytes, but with "messages" instead.

For example, this could be rows from a database, translated into JSON.

- The database will stream bytes to the server
- The server will interpret those bytes as rows
- The server will convert the row into JSON
- The server will send the JSON as bytes to the client.

Being comfortable with handling bytes will thus be very important in many applications.

## Project

Stream in a response of JSON Lines, translate the JSON into some other format, restream the output as JSON Lines.

# snowflake-rs

This project is an implementation of [Twitter's ID spec](https://github.com/twitter/snowflake/tree/snowflake-2010).
It generates approximately time-ordered ids with support for distributed id generation through machine ids. 

ID generation in this manner is useful for distributed storage systems such as Cassandra. Another use is to obscure ids
to an extent to make a dictionary attack more difficult (though not impossible) while maintaining the performance of 
integer keys as opposed to another solution such as UUIDs.


## Interface

For now, the server only implements a basic TCP server on port `47322`. The reason for this is that the connection is
meant to be left open for performance which is more difficult with an HTTP server. 

To request an id, the client sends the server the byte `0x50` (the ASCII letter `P`). The server will then respond with
a 64bit big-endian integer id. If anything else is sent, a `-1` in 64 bit two's complement will be returned.
Websocket - Unix framed bridge
==============================

A simple proxy that forwards Websocket messages to Unix sockets and vice versa.

About
-----

This is a very simple proxy between Websockets and Unix domain sockets. It acts
as a Websocket server while forwarding all messages to a specified Unix socket.

The messages forwarded to Unix socket are length-delimited, where length of
each message is encoded as 32-bit native-endian binary number. The length is
prepended to each message forwarded to Unix socket.

Conversely, each message coming from Unix socket is decoded as 32-bit
native-endian length followed by the payload. It is attempted to interpret the
message as utf-8 string and send it over websocket as text message. However, if
the conversion fails, the message is sent as binary.

Usage
-----

This program is written in Rust. Compile it using `cargo build --release`.
You'll find the binary in `target/release/` there's also a man page generated
in `target/`

In order to run use:
`ws-unix-framed-bridge --socket-path=SOCKET\_PATH --bind-addr=BIND\_ADDR`

Alternatively you may provide the arguments in toml file named
`ws-unix-bridge.conf` located in CWD.

License
-------

MITNFA

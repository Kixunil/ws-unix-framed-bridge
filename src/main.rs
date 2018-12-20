extern crate websocket;
extern crate tokio;
extern crate tokio_core;
extern crate bytes;
#[macro_use]
extern crate slog;
extern crate slog_term;

extern crate serde;
#[macro_use]
extern crate configure_me;

include_config!();

fn main() {
    use std::sync::Arc;
    use tokio::prelude::{Future, Stream, Sink};
    use websocket::message::OwnedMessage;
    use websocket::result::WebSocketError;
    use bytes::BytesMut;
    use slog::Drain;

    let drain = std::sync::Mutex::new(slog_term::term_full()).fuse();
    let logger = slog::Logger::root(drain, o!());

    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();

    let (cfg, _) = config::Config::including_optional_config_files(&["ws-unix-bridge.conf"]).unwrap();

    let socket_path = Arc::new(cfg.socket_path);

    let server = websocket::server::async::Server::bind(&cfg.bind_addr, &handle).unwrap();
    let handler = server
        .incoming()
        .map_err(|err| err.error)
        .for_each(move |(ws_conn, addr)| {
            info!(logger, "Client connected"; "client address" => %addr);
            let socket_path = socket_path.clone();
            let future = ws_conn
                .use_protocol("framed-bridge")
                .accept()
                .and_then(move |(ws_conn, _)| {
                    tokio::net::unix::UnixStream::connect(&*socket_path)
                        .map_err(Into::into)
                        .and_then(move |unix_conn| {
                            let (unix_sink, unix_stream) = tokio::codec::length_delimited::Builder::new()
                                .native_endian()
                                .new_framed(unix_conn)
                                .map_err(Into::into)
                                .sink_map_err(WebSocketError::from)
                                .split();

                            let (ws_sink, ws_stream) = ws_conn.filter_map(|msg| {
                                match msg {
                                    OwnedMessage::Binary(data) => Some(data.into()),
                                    OwnedMessage::Text(data) => Some(data.into()),
                                    _ => None,
                                }
                            })
                            .with(|data: BytesMut| -> Result<OwnedMessage, WebSocketError> {
                                Ok(std::str::from_utf8(&*data)
                                   .map(String::from)
                                   .ok()
                                   .map_or_else(|| OwnedMessage::Binary(Vec::from(&*data)), OwnedMessage::Text)
                                )
                            })
                            .split();

                            let ws_to_unix = ws_stream.forward(unix_sink);
                            let unix_to_ws = unix_stream.forward(ws_sink);

                            ws_to_unix.join(unix_to_ws).map(std::mem::drop)
                        })
                })
                .map_err(|err| eprintln!("Error: {:?}", err));
            handle.spawn(future);
            Ok(())
    });
    core.run(handler).unwrap();
}

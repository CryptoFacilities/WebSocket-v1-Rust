use std::env;

use cf_ws_v1::WebSocket;

fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let endpoint = env::args().nth(1).expect("no websocket endpoint provided");

    let mut ws = WebSocket::new(&endpoint, None, None);
    ws.subscribe("ticker", Some(&["PI_XBTUSD"]));

    for msg in ws.feed() {
        log::debug!("{msg:?}");
    }
}

use cf_ws_v1::WebSocket;

const API_PATH: &str = "wss://www.cryptofacilities.com/ws/v1";

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let mut ws = WebSocket::new(API_PATH, None, None).await;

    ws.subscribe("ticker", Some(&["PI_XBTUSD"])).await;

    while let Some(msg) = ws.next_msg().await {
        log::info!("{msg:?}");
    }
}

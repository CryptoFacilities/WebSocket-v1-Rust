use std::{
    io::{self, Read},
    thread,
};

use cf_ws_v1::WebSocket;
use log::info;
use tokio::sync::oneshot;

const API_PATH: &str = "wss://www.cryptofacilities.com/ws/v1";
const API_PUBLIC_KEY: Option<&str> = None;
const API_PRIVATE_KEY: Option<&str> = None;

async fn subscribe_api_tester(ws: &mut WebSocket) {
    ws.subscribe("trade", Some(&["PI_XBTUSD"])).await;
    ws.subscribe("book", Some(&["PI_XBTUSD"])).await;
    ws.subscribe("ticker", Some(&["PI_XBTUSD"])).await;
    ws.subscribe("ticker_lite", Some(&["PI_XBTUSD"])).await;
    ws.subscribe("heartbeat", None).await;

    ws.subscribe_private("account_balances_and_margins").await;
    ws.subscribe_private("account_log").await;
    ws.subscribe_private("deposits_withdrawals").await;
    ws.subscribe_private("fills").await;
    ws.subscribe_private("open_positions").await;
    ws.subscribe_private("open_orders").await;
    ws.subscribe_private("notifications_auth").await;
}

async fn unsubscribe_api_tester(ws: &mut WebSocket) {
    ws.unsubscribe("trade", Some(&["PI_XBTUSD"])).await;
    ws.unsubscribe("book", Some(&["PI_XBTUSD"])).await;
    ws.unsubscribe("ticker", Some(&["PI_XBTUSD"])).await;
    ws.unsubscribe("ticker_lite", Some(&["PI_XBTUSD"])).await;
    ws.unsubscribe("heartbeat", None).await;

    ws.unsubscribe_private("account_balances_and_margins").await;
    ws.unsubscribe_private("account_log").await;
    ws.unsubscribe_private("deposits_withdrawals").await;
    ws.unsubscribe_private("fills").await;
    ws.unsubscribe_private("open_positions").await;
    ws.unsubscribe_private("open_orders").await;
    ws.unsubscribe_private("notifications_auth").await;
}

fn input() {
    let mut buffer = [0; 1];
    let _ = io::stdin().read(&mut buffer);
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let mut ws = WebSocket::new(API_PATH, API_PUBLIC_KEY, API_PRIVATE_KEY).await;

    log::info!("-----------------------------------------------------------------");
    log::info!("****** PRESS ANY KEY TO SUBSCRIBE AND START RECEIVING INFO ******");
    log::info!("**** PRESS ANY KEY AGAIN TO UNSUBSCRIBE AND EXIT APPLICATION ****");
    log::info!("-----------------------------------------------------------------");

    input();
    subscribe_api_tester(&mut ws).await;

    let (stop_tx, mut stop_rx) = oneshot::channel();

    thread::spawn(move || {
        input();
        stop_tx.send(()).unwrap();
    });

    loop {
        tokio::select! {
            Some(msg) = ws.next_msg() => {
                info!("{msg:?}");
            }

            _ = &mut stop_rx => {
                break;
            }

            else => {
                log::warn!("else in main ?!?");
            }
        }
    }

    unsubscribe_api_tester(&mut ws).await;

    log::info!("-----------------------------------------------------------------");
    log::info!("********************* EXITING APPLICATION ***********************");
    log::info!("-----------------------------------------------------------------");
}

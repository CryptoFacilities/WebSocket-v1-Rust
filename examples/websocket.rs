// Crypto Facilities Ltd Web Socket API V1

// Copyright (c) 2019 Crypto Facilities

// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included
// in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
// WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.


use std::{
    io::{self, Read},
    sync::mpsc,
    thread
};

use cf_ws_v1::WebSocket;
use env_logger;
use log::info;


const API_PATH: &str = "wss://www.cryptofacilities.com/ws/v1";
const API_PUBLIC_KEY: Option<&str>  = None;
const API_PRIVATE_KEY: Option<&str> = None;

fn subscribe_api_tester(ws: &mut WebSocket) {
    ws.subscribe("trade", Some(&["PI_XBTUSD"]));
    ws.subscribe("book", Some(&["PI_XBTUSD"]));
    ws.subscribe("ticker", Some(&["PI_XBTUSD"]));
    ws.subscribe("ticker_lite", Some(&["PI_XBTUSD"]));
    ws.subscribe("heartbeat", None);

    ws.subscribe_private("account_balances_and_margins");
    ws.subscribe_private("account_log");
    ws.subscribe_private("deposits_withdrawals");
    ws.subscribe_private("fills");
    ws.subscribe_private("oprn_positions");
    ws.subscribe_private("open_orders");
    ws.subscribe_private("notifications_auth");
}

fn unsubscribe_api_tester(ws: &mut WebSocket) {
    ws.unsubscribe("trade", Some(&["PI_XBTUSD"]));
    ws.unsubscribe("book", Some(&["PI_XBTUSD"]));
    ws.unsubscribe("ticker", Some(&["PI_XBTUSD"]));
    ws.unsubscribe("ticker_lite", Some(&["PI_XBTUSD"]));
    ws.unsubscribe("heartbeat", None);

    ws.unsubscribe_private("account_balances_and_margins");
    ws.unsubscribe_private("account_log");
    ws.unsubscribe_private("deposits_withdrawals");
    ws.unsubscribe_private("fills");
    ws.unsubscribe_private("oprn_positions");
    ws.unsubscribe_private("open_orders");
    ws.unsubscribe_private("notifications_auth");
}

fn input() {
    let mut buffer = [0; 1];
    let _ = io::stdin().read(&mut buffer);
}

fn main() {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

    //let mut ws = WebSocket::new(API_PATH, Some((API_PUBLIC_KEY, API_PRIVATE_KEY)));
    let mut ws = WebSocket::new(API_PATH, API_PUBLIC_KEY, API_PRIVATE_KEY);

    println!("-----------------------------------------------------------------");
    println!("*******PRESS ANY KEY TO SUBSCRIBE AND START RECEIVING INFO*******");
    println!("*****PRESS ANY KEY AGAIN TO UNSUBSCRIBE AND EXIT APPLICATION*****");
    println!("-----------------------------------------------------------------");

    input();
    subscribe_api_tester(&mut ws);
    
    let (sender, receiver) = mpsc::channel();

    let t = thread::spawn(move || {
        for msg in ws.feed() {
            if receiver.try_recv().is_ok() { break; }
            info!("{}", msg.0);
            info!("{:?}", msg.1);
        }
        ws
    });

    input();
    sender.send(());
    let mut ws = t.join().unwrap();
    unsubscribe_api_tester(&mut ws);

    println!("-----------------------------------------------------------------");
    println!("**********************EXITING APPLICATION************************");
    println!("-----------------------------------------------------------------");
    
} 

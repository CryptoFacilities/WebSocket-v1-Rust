// Crypto Facilities Ltd Web Socket API V1

// Copyright (c) 2022 Crypto Facilities

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
    sync::mpsc::{self, Receiver, SyncSender},
    thread::{self, JoinHandle},
    time::Duration,
};

use futures::{
    future::{self, Future},
    sink::{Sink, Wait},
    stream::Stream,
    sync::mpsc as async_mpsc,
};
use log::info;
use openssl::{hash::MessageDigest, pkey::PKey, sha, sign::Signer};
use serde::{Deserialize, Serialize};
use tokio::{self, prelude::StreamExt};
use websocket::{client::builder::ClientBuilder, message::OwnedMessage, result::WebSocketError};

#[derive(Serialize, Debug)]
struct SubscribeMsg<'a> {
    event: &'a str,
    feed: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    product_ids: Option<&'a [&'a str]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    api_key: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    original_challenge: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    signed_challenge: Option<&'a str>,
}

#[derive(Serialize, Debug)]
struct ChallengeMsg<'a> {
    event: &'a str,
    api_key: &'a str,
}

#[derive(Deserialize, Debug)]
pub struct Header {
    pub feed: String,
    #[serde(default)]
    pub product_ids: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
pub struct Trade {
    #[serde(flatten)]
    pub header: Header,
    #[serde(default)]
    pub product_id: Option<String>,
    pub side: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub seq: u64,
    pub time: u64,
    pub qty: f64,
    pub price: f64,
}

#[derive(Deserialize, Debug)]
pub struct TradeSnapshot {
    #[serde(flatten)]
    pub header: Header,
    pub trades: Vec<Trade>,
}

#[derive(Deserialize, Debug)]
pub struct BookValue {
    #[serde(default)]
    pub feed: Option<String>,
    #[serde(default)]
    pub product_id: Option<String>,
    #[serde(default)]
    pub side: Option<String>,
    #[serde(default)]
    pub seq: Option<u64>,
    pub price: f64,
    pub qty: f64,
}

#[derive(Deserialize, Debug)]
pub struct BookSnapshot {
    pub feed: String,
    pub product_id: String,
    pub seq: u64,
    pub timestamp: u64,
    pub bids: Vec<BookValue>,
    pub asks: Vec<BookValue>,
}

#[derive(Deserialize, Debug)]
pub struct TickerLite {
    pub feed: String,
    pub product_id: String,
    pub bid: f64,
    pub ask: f64,
    pub change: f64,
    pub premium: f64,
    pub volume: f64,
    pub tag: String,
    pub pair: String,
    pub dtm: i64,
    #[serde(rename = "maturityTime")]
    pub maturity_time: u64,
}

#[derive(Deserialize, Debug)]
pub struct Ticker {
    #[serde(flatten)]
    pub ticker_lite: TickerLite,
    pub bid_size: f64,
    pub ask_size: f64,
    pub leverage: String,
    pub index: f64,
    pub last: f64,
    pub time: f64,
    #[serde(rename = "openInterest")]
    pub open_interest: f64,
    #[serde(rename = "markPrice")]
    pub mark_price: f64,
    #[serde(rename = "fundingRate")]
    pub funding_rate: Option<f64>,
    #[serde(rename = "relativeFundingRatePrediction", default)]
    pub relative_funding_rate_prediction: Option<f64>,
    #[serde(rename = "fundingRatePrediction", default)]
    pub funding_rate_prediction: Option<f64>,
    #[serde(rename = "nextFundingRateTime", default)]
    pub next_funding_rate_time: Option<f64>,
}

#[derive(Deserialize, Debug)]
pub struct Heartbeat {
    #[serde(flatten)]
    pub header: Header,
    pub time: u64,
}

#[derive(Deserialize, Debug)]
pub struct Challenge {
    pub feed: String,
    pub event: String,
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct Subscribed {
    pub event: String,
    #[serde(flatten)]
    pub header: Header,
}

#[derive(Deserialize, Debug)]
pub struct Error {
    pub event: String,
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct Version {
    pub event: String,
    pub version: u64,
}

//// Private ////

#[derive(Deserialize, Debug)]
pub struct MarginAccount {
    name: String,
    balance: f64,
    pnl: f64,
    pv: f64,
    am: f64,
    im: f64,
    mm: f64,
}

#[derive(Deserialize, Debug)]
pub struct AccountBalancesAndMargins {
    feed: String,
    account: String,
    seq: u64,
    margin_accounts: Vec<MarginAccount>,
}

#[derive(Deserialize, Debug)]
pub struct Log {
    id: u64,
    date: String,
    asset: String,
    info: String,
    booking_uid: String,
    margin_account: String,
    old_balance: f64,
    new_balance: f64,
    old_average_entry_price: f64,
    new_average_entry_price: f64,
    trade_price: f64,
    mark_price: f64,
    fee: f64,
    execution: String,
    collateral: String,
    funding_rate: f64,
    realised_funding: f64,
}

#[derive(Deserialize, Debug)]
pub struct AccountLog {
    feed: String,
    logs: Vec<Log>,
}

#[derive(Deserialize, Debug)]
pub struct DepositWithdrawal {
    uid: String,
    time: String,
    amount: f64,
    unit: String,
    receiving_address: String,
    status: String,
    confirmations: u64,
    tx_reference: String,
}

#[derive(Deserialize, Debug)]
pub struct DepositsWithdrawals {
    feed: String,
    elements: Vec<DepositWithdrawal>,
}

#[derive(Deserialize, Debug)]
pub struct Fill {
    instrument: String,
    time: String,
    price: f64,
    seq: u64,
    buy: bool,
    qty: f64,
    order_id: String,
    fill_id: String,
    fill_type: String,
}

#[derive(Deserialize, Debug)]
pub struct Fills {
    feed: String,
    account: String,
    fills: Vec<Fill>,
}

#[derive(Deserialize, Debug)]
pub struct Position {
    instrument: String,
    balance: f64,
    entry_price: f64,
    mark_price: f64,
    index_price: f64,
    pnl: f64,
    liquidation_threashold: f64,
    return_on_equity: f64,
    effective_leverage: f64,
}

#[derive(Deserialize, Debug)]
pub struct OpenPositions {
    feed: String,
    account: String,
    positions: Vec<Position>,
}

#[derive(Deserialize, Debug)]
pub struct Order {
    instrument: String,
    time: u64,
    qty: f64,
    filled: f64,
    limit_price: f64,
    stop_price: f64,
    #[serde(rename = "type")]
    ty: String,
    order_id: String,
    #[serde(default)]
    cli_order_id: Option<String>,
    direction: i64,
}

#[derive(Deserialize, Debug)]
pub struct OpenOrdersSnapshot {
    feed: String,
    account: String,
    orders: Vec<Order>,
}

#[derive(Deserialize, Debug)]
pub struct OpenOrders {
    feed: String,
    is_cancel: bool,
    reason: String,
    #[serde(default)]
    order_id: Option<String>,
    #[serde(default)]
    cli_ord_id: Option<String>,
    #[serde(default)]
    order: Option<Order>,
}

#[derive(Deserialize, Debug)]
pub struct Notification {
    id: u64,
    #[serde(rename = "type")]
    ty: String,
    priority: String,
    note: String,
    effective_time: u64,
}

#[derive(Deserialize, Debug)]
pub struct Notifications {
    feed: String,
    notifications: Vec<Notification>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Msg {
    Version(Version),
    Subscribed(Subscribed),
    Error(Error),
    Trade(Trade),
    TradeSnapshot(TradeSnapshot),
    Book(BookValue),
    BookSnapshot(BookSnapshot),
    Ticker(Ticker),
    TickerLite(TickerLite),
    Heartbeat(Heartbeat),
    Challenge(Challenge),
    AccountBalancesAndMargins(AccountBalancesAndMargins),
    AccountLog(AccountLog),
    DepositsWithdrawals(DepositsWithdrawals),
    Fills(Fills),
    OpenPositions(OpenPositions),
    OpenOrders(OpenOrders),
    OpenOrdersSnapshot(OpenOrdersSnapshot),
    Notifications(Notifications),
}

pub struct WebSocket {
    keys: Option<(String, String)>,
    ws_url: String,
    challenge: Option<String>,
    signed_challenge: Option<String>,
    sub_sender: Wait<async_mpsc::Sender<OwnedMessage>>,
    receiver: Receiver<Msg>,
    listener: Option<JoinHandle<()>>,
}

impl WebSocket {
    pub fn new(ws_url: &str, public_key: Option<&str>, private_key: Option<&str>) -> WebSocket {
        let (sender, receiver) = mpsc::sync_channel(1);
        let (sub_sender, sub_receiver) = async_mpsc::channel(0);

        let sub_sender_sync = sub_sender.wait();

        let mut ws = WebSocket {
            keys: public_key.and_then(|pb| private_key.map(|pv| (pb.to_owned(), pv.to_owned()))),
            ws_url: ws_url.to_owned(),
            challenge: None,
            signed_challenge: None,
            sub_sender: sub_sender_sync,
            receiver,
            listener: None,
        };

        ws.listen(sender, sub_receiver);

        ws
    }

    pub fn feed(&mut self) -> mpsc::Iter<Msg> {
        self.receiver.iter()
    }

    //// public feeds ////

    pub fn subscribe(&mut self, feed: &str, products: Option<&[&str]>) {
        let msg = serde_json::to_string(&SubscribeMsg {
            event: "subscribe",
            feed,
            product_ids: products,
            api_key: None,
            original_challenge: None,
            signed_challenge: None,
        })
        .unwrap();

        info!("subscribe to public feed: {}", feed);

        let _ = self.sub_sender.send(OwnedMessage::Text(msg));
    }

    pub fn unsubscribe(&mut self, feed: &str, products: Option<&[&str]>) {
        let msg = serde_json::to_string(&SubscribeMsg {
            event: "unsubscribe",
            feed,
            product_ids: products,
            api_key: None,
            original_challenge: None,
            signed_challenge: None,
        })
        .unwrap();

        info!("unsubscribe from public feed: {}", feed);

        let _ = self.sub_sender.send(OwnedMessage::Text(msg));
    }

    //// private feeds ////

    pub fn subscribe_private(&mut self, feed: &str) -> Option<()> {
        if self.challenge.is_none() {
            self.sign_challenge()?;
        }

        let msg = serde_json::to_string(&SubscribeMsg {
            event: "subscribe",
            feed,
            product_ids: None,
            api_key: self.keys.as_ref().map(|(pb, _)| &**pb),
            original_challenge: self.challenge.as_deref(),
            signed_challenge: self.signed_challenge.as_deref(),
        })
        .unwrap();

        info!("subscribe to private feed: {}", feed);

        let _ = self.sub_sender.send(OwnedMessage::Text(msg));
        None
    }

    pub fn unsubscribe_private(&mut self, feed: &str) -> Option<()> {
        if self.challenge.is_none() {
            self.sign_challenge()?;
        }

        let msg = serde_json::to_string(&SubscribeMsg {
            event: "subscribe",
            feed,
            product_ids: None,
            api_key: self.keys.as_ref().map(|(pb, _)| &**pb),
            original_challenge: self.challenge.as_deref(),
            signed_challenge: self.signed_challenge.as_deref(),
        })
        .unwrap();

        info!("unsubscribe from private feed: {}", feed);

        let _ = self.sub_sender.send(OwnedMessage::Text(msg));
        None
    }

    // sign challenge request
    fn sign_challenge(&mut self) -> Option<()> {
        match (self.keys.as_ref(), self.challenge.as_ref()) {
            (Some(_), Some(_)) => Some(()),
            (Some((ref pb, ref pv)), None) => {
                Self::request_challenge(&mut self.sub_sender, &*pb);
                let c = self.wait_for_challenge();
                self.signed_challenge = Some(Self::sign(&*pv, &*c));
                self.challenge = Some(c);
                Some(())
            }
            _ => None,
        }
    }

    // blocks until challenge event arrives
    fn wait_for_challenge(&self) -> String {
        let mut challenge = String::new();

        info!("waiting for challenge");

        for msg in self.receiver.iter() {
            if let Msg::Challenge(c) = msg {
                challenge = c.message;
                break;
            }
        }
        challenge
    }

    fn request_challenge(sender: &mut Wait<async_mpsc::Sender<OwnedMessage>>, public_key: &str) {
        let msg = serde_json::to_string(&ChallengeMsg {
            event: "challenge",
            api_key: public_key,
        })
        .unwrap();

        let _ = sender.send(OwnedMessage::Text(msg));
    }

    fn hmac(secret: &[u8], data: &[u8]) -> Vec<u8> {
        let key = PKey::hmac(secret).unwrap();
        let mut signer = Signer::new(MessageDigest::sha512(), &key).unwrap();
        signer.update(data).unwrap();
        signer.sign_to_vec().unwrap()
    }

    fn sign(private_key: &str, challenge: &str) -> String {
        let mut hasher = sha::Sha256::new();
        hasher.update(challenge.as_bytes());
        let hash = hasher.finish();
        let secret_decoded = base64::decode(private_key.as_bytes()).unwrap();
        let digest = Self::hmac(&*secret_decoded, &hash);
        base64::encode(&*digest)
    }

    // async listener
    fn listen(
        &mut self,
        sender: SyncSender<Msg>,
        sub_receiver: async_mpsc::Receiver<OwnedMessage>,
    ) {
        let mut runtime = tokio::runtime::Builder::new()
            .blocking_threads(1)
            .core_threads(1)
            .build()
            .unwrap();

        // Establish connection to websocket server
        let client = ClientBuilder::new(&*self.ws_url)
            .unwrap()
            .async_connect(None)
            .and_then(move |(duplex, _)| {
                let (sink, stream) = duplex.split();
                stream
                    .timeout(Duration::from_secs(20))
                    .map_err(|_| WebSocketError::NoDataAvailable)
                    .filter_map(move |message| match message {
                        OwnedMessage::Text(data) => {
                            let _ = serde_json::from_str::<Msg>(&*data).map(|m| {
                                let _ = sender.send(m);
                            });
                            None
                        }
                        OwnedMessage::Ping(data) => Some(OwnedMessage::Pong(data)),
                        OwnedMessage::Close(e) => Some(OwnedMessage::Close(e)),
                        _ => None,
                    })
                    .select(sub_receiver.map_err(|_| WebSocketError::NoDataAvailable))
                    .take_while(|x| future::ok(!x.is_close()))
                    .forward(sink)
            })
            .map_err(|_| ())
            .map(|_| ());

        self.listener = Some(thread::spawn(move || {
            let _ = runtime.spawn(client);
            let _ = runtime.shutdown_on_idle().wait();
        }));
    }
}

impl Drop for WebSocket {
    fn drop(&mut self) {
        let _ = self.sub_sender.send(OwnedMessage::Close(None));
        if let Some(l) = self.listener.take() {
            let _ = l.join();
        }
    }
}

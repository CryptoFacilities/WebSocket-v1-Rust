// Crypto Facilities Ltd Web Socket API V1

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
use tokio::{self, prelude::StreamExt};
use websocket::{client::builder::ClientBuilder, message::OwnedMessage, result::WebSocketError};

mod models;
pub use models::*;

pub struct WebSocket {
    keys: Option<(String, String)>,
    ws_url: String,
    challenge: Option<String>,
    signed_challenge: Option<String>,
    sub_sender: Wait<async_mpsc::Sender<OwnedMessage>>,
    receiver: Receiver<models::Msg>,
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

    pub fn feed(&mut self) -> mpsc::Iter<models::Msg> {
        self.receiver.iter()
    }

    //// public feeds ////

    pub fn subscribe(&mut self, feed: &str, products: Option<&[&str]>) {
        let msg = serde_json::to_string(&models::SubscribeMsg {
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
        let msg = serde_json::to_string(&models::SubscribeMsg {
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

        let msg = serde_json::to_string(&models::SubscribeMsg {
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

        let msg = serde_json::to_string(&models::SubscribeMsg {
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
                Self::request_challenge(&mut self.sub_sender, pb);
                let c = self.wait_for_challenge();
                self.signed_challenge = Some(Self::sign(pv, &c));
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
            if let models::Msg::Challenge(c) = msg {
                challenge = c.message;
                break;
            }
        }
        challenge
    }

    fn request_challenge(sender: &mut Wait<async_mpsc::Sender<OwnedMessage>>, public_key: &str) {
        let msg = serde_json::to_string(&models::ChallengeMsg {
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
        let digest = Self::hmac(&secret_decoded, &hash);
        base64::encode(&*digest)
    }

    // async listener
    fn listen(
        &mut self,
        sender: SyncSender<models::Msg>,
        sub_receiver: async_mpsc::Receiver<OwnedMessage>,
    ) {
        let mut runtime = tokio::runtime::Builder::new()
            .blocking_threads(1)
            .core_threads(1)
            .build()
            .unwrap();

        // Establish connection to websocket server
        let client = ClientBuilder::new(&self.ws_url)
            .unwrap()
            .async_connect(None)
            .and_then(move |(duplex, _)| {
                let (sink, stream) = duplex.split();
                stream
                    .timeout(Duration::from_secs(20))
                    .map_err(|_| WebSocketError::NoDataAvailable)
                    .filter_map(move |message| match message {
                        OwnedMessage::Text(data) => {
                            let _ = serde_json::from_str::<models::Msg>(&data).map(|m| {
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

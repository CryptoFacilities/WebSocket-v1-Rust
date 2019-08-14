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
    sync::{
        mpsc::{self, Receiver, SyncSender},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use base64;
use futures::{
    future::{self, Future},
    sink::{Sink, Wait},
    stream::Stream,
    sync::mpsc as async_mpsc,
};
use log::info;
use openssl::{
    sha,
    hash::MessageDigest,
    pkey::PKey,
    sign::Signer,
};
use serde_json::json;
use tokio;
use websocket::{
    client::{
        builder::ClientBuilder,
    },
    message::OwnedMessage,
    result::WebSocketError,
};

pub struct WebSocket {
    keys: Option<(String, String)>,
    ws_url: String,
    challenge: Option<String>,
    signed_challenge: Option<String>,
    sub_sender: Wait<async_mpsc::Sender<OwnedMessage>>,
    receiver: Receiver<(String, serde_json::Value)>,
    listener: Option<JoinHandle<()>>,
}

impl WebSocket {
    pub fn new(ws_url: &str, public_key: Option<&str>, private_key: Option<&str>) -> WebSocket {
        let (sender, receiver) = mpsc::sync_channel(1);
        let (sub_sender, sub_receiver) = async_mpsc::channel(0);
        
        let sub_sender_sync = sub_sender.wait();
        
        let mut ws = WebSocket {
            keys: public_key.and_then(|pb| private_key.map(|pv| (pb.to_owned(), pv.to_owned()) )),
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

    pub fn feed(&mut self) -> mpsc::Iter<(String, serde_json::Value)> {
        self.receiver.iter()
    }

    //// public feeds ////

    pub fn subscribe(&mut self, feed: &str, products: Option<&[&str]>) { 
        let msg = match products {
            Some(ps) => {
                json!({ 
                    "event": "subscribe",
                    "feed": feed,
                    "product_ids": ps
                }).to_string()
            },
            None => {
                json!({ 
                    "event": "subscribe",
                    "feed": feed
                }).to_string()
            }
        };

        info!("subscribe to public feed: {}", feed);
        
        let _ = self.sub_sender.send(OwnedMessage::Text(msg));
    }

    pub fn unsubscribe(&mut self, feed: &str, products: Option<&[&str]>) { 
        let msg = match products {
            Some(ps) => {
                json!({ 
                    "event": "unsubscribe",
                    "feed": feed,
                    "product_ids": ps
                }).to_string()
            },
            None => {
                json!({ 
                    "event": "unsubscribe",
                    "feed": feed
                }).to_string()
            }
        };
        
        info!("unsubscribe from public feed: {}", feed);

        let _ = self.sub_sender.send(OwnedMessage::Text(msg));
    }

    //// private feeds ////

    pub fn subscribe_private(&mut self, feed: &str) -> Option<()> {
        if self.challenge.is_none() {
            self.sign_challenge()?;
        }

        let msg = json!({ 
            "event": "subscribe",
            "feed": feed,
            "api_key": self.keys.as_ref().map(|(pb, _)| pb).unwrap(),
            "original_challenge": self.challenge.as_ref().unwrap(),
            "signed_challenge": self.signed_challenge.as_ref().unwrap(),
        }).to_string();

        info!("subscribe to private feed: {}", feed);

        let _ = self.sub_sender.send(OwnedMessage::Text(msg));
        None
    }

    pub fn unsubscribe_private(&mut self, feed: &str) -> Option<()> {
        if self.challenge.is_none() {
            self.sign_challenge()?;
        }

        let msg = json!({ 
            "event": "unsubscribe",
            "feed": feed,
            "api_key": self.keys.as_ref().map(|(pb, _)| pb).unwrap(),
            "original_challenge": self.challenge.as_ref().unwrap(),
            "signed_challenge": self.signed_challenge.as_ref().unwrap(),
        }).to_string();

        info!("unsubscribe from private feed: {}", feed);

        let _ = self.sub_sender.send(OwnedMessage::Text(msg));
        None
    }

    // sign challenge request  
    fn sign_challenge(&mut self) -> Option<()> {
        match (self.keys.as_ref(), self.challenge.as_ref()) { 
            (Some((ref pb, ref pv)), None) => { 
                Self::request_challenge(&mut self.sub_sender, &*pb);
                let c = self.wait_for_challenge();
                self.signed_challenge = Some(Self::sign(&*pv, &*c));
                self.challenge = Some(c);
                Some(())
            },
            _ => None
        }
    }

    // blocks until challenge event arrives
    fn wait_for_challenge(&self) -> String {
        let mut challenge = String::new();

        info!("waiting for challenge");

        for msg in self.receiver.iter() {
            if msg.0 == "challenge" {
                challenge = msg.1["message"].as_str().unwrap().to_owned();
                break;
            }
        }
        challenge
    }

    fn request_challenge(sender: &mut Wait<async_mpsc::Sender<OwnedMessage>>, public_key: &str) {
        let msg = json!({
            "event": "challenge",
            "api_key": public_key,
        }).to_string();
        
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
    fn listen(&mut self, sender: SyncSender<(String, serde_json::Value)>, sub_receiver: async_mpsc::Receiver<OwnedMessage>) {
        let mut runtime = tokio::runtime::Builder::new()
            //.keep_alive(Some(Duration::from_secs(10)))
            .blocking_threads(1)
            .core_threads(1)
            .build()
            .unwrap();

        // Establish connection to websocket server 
        let client = ClientBuilder::new(&*self.ws_url).unwrap()
            .async_connect_secure(None)
            .and_then(move |(duplex, _)| {
                let (sink, stream) = duplex.split();
                stream.filter_map(move |message| {
                    match message {
                        OwnedMessage::Text(data) => { 
                            if let Ok(d) = serde_json::from_str::<serde_json::Value>(&*data) {
                                if let Some(event) = d.get("event").and_then(|e| e.as_str()) {
                                    if event == "challenge" {
                                        let _ = sender.send((event.to_owned(), d));
                                    }
                                }
                                else if let Some(feed) = d.get("feed").and_then(|f| f.as_str()) {
                                    let _ = sender.send((feed.to_owned(), d));
                                }
                            }

                            None
                        },
                        OwnedMessage::Ping(data) => Some(OwnedMessage::Pong(data)),
                        OwnedMessage::Close(e) => Some(OwnedMessage::Close(e)),
                        _ => None,
                    }
                })
                .select(sub_receiver.map_err(|_| WebSocketError::NoDataAvailable))
                .take_while(|x| future::ok(!x.is_close()))
                .forward(sink)
            });
        
        self.listener = Some(thread::spawn(move || {
            let _ = runtime.block_on(client);
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


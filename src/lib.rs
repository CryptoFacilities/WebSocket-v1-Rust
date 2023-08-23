//! Crypto Facilities Ltd WebSocket API v1

#![deny(rust_2018_idioms, nonstandard_style, future_incompatible)]

use std::time::Duration;

use base64::prelude::*;
use futures_util::{SinkExt as _, StreamExt as _};
use hmac::{Hmac, Mac as _};
use log::info;
use sha2::{Digest as _, Sha256, Sha512};
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_tungstenite::{connect_async, tungstenite::Message};

mod models;
pub use models::*;

type HmacSha512 = Hmac<Sha512>;

/// How frequently to ping on the websocket.
///
/// As per [the docs](https://docs.futures.kraken.com/#websocket-api-websocket-api-introduction-subscriptions-keeping-the-connection-alive)
/// this should be at least once every 60 seconds, so we use of one second less to ensure minor
/// delays don't result in a closed connection.
const PING_TIMER: Duration = Duration::from_secs(60 - 1);

pub struct WebSocket {
    tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<models::Msg>,
    keys: Option<(String, String)>,
    challenge: Option<String>,
    signed_challenge: Option<String>,
    _handle: JoinHandle<()>,
}

impl WebSocket {
    pub async fn new(
        ws_url: &str,
        public_key: Option<&str>,
        private_key: Option<&str>,
    ) -> WebSocket {
        let ws_url = ws_url.to_owned();

        let (send_tx, mut send_rx) = mpsc::channel(42);
        let (recv_tx, recv_rx) = mpsc::channel(42);

        let (mut ws, _res) = connect_async(&ws_url).await.unwrap();

        let handle = tokio::spawn(async move {
            loop {
                log::trace!("waiting for message or event");

                tokio::select! {
                    msg = send_rx.recv() => {
                        match msg {
                            Some(msg) => {
                                log::trace!("sending TEXT: {msg:?}");
                                ws.send(msg).await.unwrap();
                                ws.flush().await.unwrap();
                            }

                            None => {
                                panic!("out tx closed ?!");
                            }
                        };
                    }

                    res = ws.next() => {
                        let msg = match res {
                            Some(Ok(msg)) => msg,
                            Some(Err(err)) => {
                                log::error!("{err}");
                                continue;
                            }
                            None => break,
                        };

                        match msg {
                            Message::Text(text) => {
                                log::debug!("TEXT received: {} bytes", text.len());
                                log::trace!("TEXT received: {text}");

                                // parse and send to channel
                                let msg = serde_json::from_str(&text).unwrap();
                                recv_tx.send(msg).await.unwrap();
                            }
                            Message::Binary(_) => {
                                unimplemented!("BINARY message received; not supported");
                            }
                            Message::Ping(msg) => {
                                log::trace!("PING received; sending PONG");
                                ws.send(Message::Pong(msg)).await.unwrap();
                            }
                            Message::Pong(msg) => {
                                log::debug!("PONG received: {msg:X?}");
                            },
                            Message::Close(msg) => {
                                log::debug!("CLOSE received: {msg:X?}");
                                ws.close(msg).await.unwrap();
                                break;
                            },
                            Message::Frame(_) => unreachable!("raw frames are not exposed here"),
                        };
                    }

                    // If we're otherwise sending and receiving messages, there's no need to send
                    // a new PING. So we start a new one each time we re-enter [`tokio::select`].
                    _ = tokio::time::sleep(PING_TIMER) => {
                        log::trace!("Timer expired; sending PING");
                        ws.send(Message::Ping("PING".into())).await.unwrap();
                    }

                    else => {
                        panic!("else branch reached somehow ?!?");
                    }
                };
            }

            log::warn!("WS management task is done");
        });

        WebSocket {
            tx: send_tx,
            rx: recv_rx,
            keys: public_key
                .map(ToOwned::to_owned)
                .zip(private_key.map(ToOwned::to_owned)),
            challenge: None,
            signed_challenge: None,
            _handle: handle,
        }
    }

    pub async fn next_msg(&mut self) -> Option<models::Msg> {
        self.rx.recv().await
    }

    //// public feeds ////

    pub async fn subscribe(&mut self, feed: &str, products: Option<&[&str]>) {
        let msg = serde_json::to_string(&models::SubscribeMsg {
            event: "subscribe",
            feed,
            product_ids: products,
            api_key: None,
            original_challenge: None,
            signed_challenge: None,
        })
        .unwrap();

        info!("subscribe to public feed: {feed}");

        self.tx.send(Message::Text(msg)).await.unwrap();
    }

    pub async fn unsubscribe(&mut self, feed: &str, products: Option<&[&str]>) {
        let msg = serde_json::to_string(&models::SubscribeMsg {
            event: "unsubscribe",
            feed,
            product_ids: products,
            api_key: None,
            original_challenge: None,
            signed_challenge: None,
        })
        .unwrap();

        info!("unsubscribe from public feed: {feed}");

        self.tx.send(Message::Text(msg)).await.unwrap();
    }

    //// private feeds ////

    pub async fn subscribe_private(&mut self, feed: &str) -> Option<()> {
        if self.challenge.is_none() {
            self.sign_challenge().await?;
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

        info!("subscribe to private feed: {feed}");

        self.tx.send(Message::Text(msg)).await.unwrap();

        None
    }

    pub async fn unsubscribe_private(&mut self, feed: &str) -> Option<()> {
        if self.challenge.is_none() {
            self.sign_challenge().await?;
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

        self.tx.send(Message::Text(msg)).await.unwrap();

        None
    }

    // sign challenge request
    async fn sign_challenge(&mut self) -> Option<()> {
        match (&self.keys, self.challenge.clone()) {
            (Some(_), Some(_)) => Some(()),
            (Some((pb, ref pv)), None) => {
                let pb = pb.clone();
                let pv = pv.clone();
                self.request_challenge(&pb).await;
                let challenge = self.wait_for_challenge().await;
                log::debug!("found challenge: {challenge}");
                self.signed_challenge = Some(Self::sign(&pv, &challenge));
                self.challenge = Some(challenge);
                Some(())
            }
            _ => None,
        }
    }

    async fn request_challenge(&mut self, public_key: &str) {
        let msg = serde_json::to_string(&models::ChallengeMsg {
            event: "challenge",
            api_key: public_key,
        })
        .unwrap();

        self.tx.send(Message::Text(msg)).await.unwrap();
    }

    // waits until challenge event arrives
    async fn wait_for_challenge(&mut self) -> String {
        let mut challenge = String::new();

        info!("waiting for challenge");

        while let Some(msg) = self.next_msg().await {
            match msg {
                models::Msg::Error(c) if c.event == "challenge" => {
                    challenge = c.message;
                    break;
                }
                _ => {}
            }
        }

        challenge
    }

    fn hmac(secret: &[u8], data: &[u8]) -> Vec<u8> {
        let mut signer = HmacSha512::new_from_slice(secret).unwrap();
        signer.update(data);
        signer.finalize().into_bytes().to_vec()
    }

    fn sign(private_key: &str, challenge: &str) -> String {
        let challenge_hash = Sha256::digest(challenge);
        let secret = BASE64_STANDARD.decode(private_key).unwrap();
        let digest = Self::hmac(&secret, &challenge_hash);
        BASE64_STANDARD.encode(digest)
    }
}

impl Drop for WebSocket {
    fn drop(&mut self) {
        drop(self.tx.send(Message::Close(None)));
    }
}

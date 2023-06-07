use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub(crate) struct SubscribeMsg<'a> {
    pub(crate) event: &'a str,
    pub(crate) feed: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) product_ids: Option<&'a [&'a str]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) api_key: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) original_challenge: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) signed_challenge: Option<&'a str>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ChallengeMsg<'a> {
    pub(crate) event: &'a str,
    pub(crate) api_key: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct Header {
    pub feed: String,
    #[serde(default)]
    pub product_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct TradeSnapshot {
    #[serde(flatten)]
    pub header: Header,
    pub trades: Vec<Trade>,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct BookSnapshot {
    pub feed: String,
    pub product_id: String,
    pub seq: u64,
    pub timestamp: u64,
    pub bids: Vec<BookValue>,
    pub asks: Vec<BookValue>,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct Heartbeat {
    #[serde(flatten)]
    pub header: Header,
    pub time: u64,
}

#[derive(Debug, Deserialize)]
pub struct Challenge {
    pub feed: String,
    pub event: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct Subscribed {
    pub event: String,
    #[serde(flatten)]
    pub header: Header,
}

#[derive(Debug, Deserialize)]
pub struct Error {
    pub event: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct Version {
    pub event: String,
    pub version: u64,
}

#[derive(Debug, Deserialize)]
pub struct MarginAccount {
    pub(crate) name: String,
    pub(crate) balance: f64,
    pub(crate) pnl: f64,
    pub(crate) pv: f64,
    pub(crate) am: f64,
    pub(crate) im: f64,
    pub(crate) mm: f64,
}

#[derive(Debug, Deserialize)]
pub struct AccountBalancesAndMargins {
    pub(crate) feed: String,
    pub(crate) account: String,
    pub(crate) seq: u64,
    pub(crate) margin_accounts: Vec<MarginAccount>,
}

#[derive(Debug, Deserialize)]
pub struct Log {
    pub(crate) id: u64,
    pub(crate) date: String,
    pub(crate) asset: String,
    pub(crate) info: String,
    pub(crate) booking_uid: String,
    pub(crate) margin_account: String,
    pub(crate) old_balance: f64,
    pub(crate) new_balance: f64,
    pub(crate) old_average_entry_price: f64,
    pub(crate) new_average_entry_price: f64,
    pub(crate) trade_price: f64,
    pub(crate) mark_price: f64,
    pub(crate) fee: f64,
    pub(crate) execution: String,
    pub(crate) collateral: String,
    pub(crate) funding_rate: f64,
    pub(crate) realised_funding: f64,
}

#[derive(Debug, Deserialize)]
pub struct AccountLog {
    pub(crate) feed: String,
    pub(crate) logs: Vec<Log>,
}

#[derive(Debug, Deserialize)]
pub struct DepositWithdrawal {
    pub(crate) uid: String,
    pub(crate) time: String,
    pub(crate) amount: f64,
    pub(crate) unit: String,
    pub(crate) receiving_address: String,
    pub(crate) status: String,
    pub(crate) confirmations: u64,
    pub(crate) tx_reference: String,
}

#[derive(Debug, Deserialize)]
pub struct DepositsWithdrawals {
    pub(crate) feed: String,
    pub(crate) elements: Vec<DepositWithdrawal>,
}

#[derive(Debug, Deserialize)]
pub struct Fill {
    pub(crate) instrument: String,
    pub(crate) time: String,
    pub(crate) price: f64,
    pub(crate) seq: u64,
    pub(crate) buy: bool,
    pub(crate) qty: f64,
    pub(crate) order_id: String,
    pub(crate) fill_id: String,
    pub(crate) fill_type: String,
}

#[derive(Debug, Deserialize)]
pub struct Fills {
    pub(crate) feed: String,
    pub(crate) account: String,
    pub(crate) fills: Vec<Fill>,
}

#[derive(Debug, Deserialize)]
pub struct Position {
    pub(crate) instrument: String,
    pub(crate) balance: f64,
    pub(crate) entry_price: f64,
    pub(crate) mark_price: f64,
    pub(crate) index_price: f64,
    pub(crate) pnl: f64,
    pub(crate) liquidation_threashold: f64,
    pub(crate) return_on_equity: f64,
    pub(crate) effective_leverage: f64,
}

#[derive(Debug, Deserialize)]
pub struct OpenPositions {
    pub(crate) feed: String,
    pub(crate) account: String,
    pub(crate) positions: Vec<Position>,
}

#[derive(Debug, Deserialize)]
pub struct Order {
    pub(crate) instrument: String,
    pub(crate) time: u64,
    pub(crate) qty: f64,
    pub(crate) filled: f64,
    pub(crate) limit_price: f64,
    pub(crate) stop_price: f64,
    #[serde(rename = "type")]
    pub(crate) ty: String,
    pub(crate) order_id: String,
    #[serde(default)]
    pub(crate) cli_order_id: Option<String>,
    pub(crate) direction: i64,
}

#[derive(Debug, Deserialize)]
pub struct OpenOrdersSnapshot {
    pub(crate) feed: String,
    pub(crate) account: String,
    pub(crate) orders: Vec<Order>,
}

#[derive(Debug, Deserialize)]
pub struct OpenOrders {
    pub(crate) feed: String,
    pub(crate) is_cancel: bool,
    pub(crate) reason: String,
    #[serde(default)]
    pub(crate) order_id: Option<String>,
    #[serde(default)]
    pub(crate) cli_ord_id: Option<String>,
    #[serde(default)]
    pub(crate) order: Option<Order>,
}

#[derive(Debug, Deserialize)]
pub struct Notification {
    pub(crate) id: u64,
    #[serde(rename = "type")]
    pub(crate) ty: String,
    pub(crate) priority: String,
    pub(crate) note: String,
    pub(crate) effective_time: u64,
}

#[derive(Debug, Deserialize)]
pub struct Notifications {
    pub(crate) feed: String,
    pub(crate) notifications: Vec<Notification>,
}

#[derive(Debug, Deserialize)]
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

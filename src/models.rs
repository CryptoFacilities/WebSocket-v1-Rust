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
    pub name: String,
    pub balance: f64,
    pub pnl: f64,
    pub pv: f64,
    pub am: f64,
    pub im: f64,
    pub mm: f64,
}

#[derive(Debug, Deserialize)]
pub struct AccountBalancesAndMargins {
    pub feed: String,
    pub account: String,
    pub seq: u64,
    pub margin_accounts: Vec<MarginAccount>,
}

#[derive(Debug, Deserialize)]
pub struct Log {
    pub id: u64,
    pub date: String,
    pub asset: String,
    pub info: String,
    pub booking_uid: String,
    pub margin_account: String,
    pub old_balance: f64,
    pub new_balance: f64,
    pub old_average_entry_price: f64,
    pub new_average_entry_price: f64,
    pub trade_price: f64,
    pub mark_price: f64,
    pub fee: f64,
    pub execution: String,
    pub collateral: String,
    pub funding_rate: f64,
    pub realised_funding: f64,
}

#[derive(Debug, Deserialize)]
pub struct AccountLog {
    pub feed: String,
    pub logs: Vec<Log>,
}

#[derive(Debug, Deserialize)]
pub struct DepositWithdrawal {
    pub uid: String,
    pub time: String,
    pub amount: f64,
    pub unit: String,
    pub receiving_address: String,
    pub status: String,
    pub confirmations: u64,
    pub tx_reference: String,
}

#[derive(Debug, Deserialize)]
pub struct DepositsWithdrawals {
    pub feed: String,
    pub elements: Vec<DepositWithdrawal>,
}

#[derive(Debug, Deserialize)]
pub struct Fill {
    pub instrument: String,
    pub time: String,
    pub price: f64,
    pub seq: u64,
    pub buy: bool,
    pub qty: f64,
    pub order_id: String,
    pub fill_id: String,
    pub fill_type: String,
}

#[derive(Debug, Deserialize)]
pub struct Fills {
    pub feed: String,
    pub account: String,
    pub fills: Vec<Fill>,
}

#[derive(Debug, Deserialize)]
pub struct Position {
    pub instrument: String,
    pub balance: f64,
    pub entry_price: f64,
    pub mark_price: f64,
    pub index_price: f64,
    pub pnl: f64,
    pub liquidation_threashold: f64,
    pub return_on_equity: f64,
    pub effective_leverage: f64,
}

#[derive(Debug, Deserialize)]
pub struct OpenPositions {
    pub feed: String,
    pub account: String,
    pub positions: Vec<Position>,
}

#[derive(Debug, Deserialize)]
pub struct Order {
    pub instrument: String,
    pub time: u64,
    pub qty: f64,
    pub filled: f64,
    pub limit_price: f64,
    pub stop_price: f64,
    #[serde(rename = "type")]
    pub ty: String,
    pub order_id: String,
    #[serde(default)]
    pub cli_order_id: Option<String>,
    pub direction: i64,
}

#[derive(Debug, Deserialize)]
pub struct OpenOrdersSnapshot {
    pub feed: String,
    pub account: String,
    pub orders: Vec<Order>,
}

#[derive(Debug, Deserialize)]
pub struct OpenOrders {
    pub feed: String,
    pub is_cancel: bool,
    pub reason: String,
    #[serde(default)]
    pub order_id: Option<String>,
    #[serde(default)]
    pub cli_ord_id: Option<String>,
    #[serde(default)]
    pub order: Option<Order>,
}

#[derive(Debug, Deserialize)]
pub struct Notification {
    pub id: u64,
    #[serde(rename = "type")]
    pub ty: String,
    pub priority: String,
    pub note: String,
    pub effective_time: u64,
}

#[derive(Debug, Deserialize)]
pub struct Notifications {
    pub feed: String,
    pub notifications: Vec<Notification>,
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

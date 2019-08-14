Crypto Facilities Websocket API v1
==================================

This is a Rust web socket application for [Crypto Facilities Ltd](https://www.cryptofacilities.com/)


Getting Started
---------------

1. Amend `examples/websocket.rs` file to enter your api keys.
2. Run the example application with ```$ cargo run --example websocket```

Functionality Overview
----------------------

* This application subscribes to all available feeds


Application Sample Output
-------------------------

The following is some of what you can expect when running this application:

```
[2019-08-14T13:35:04Z INFO  websocket] book
[2019-08-14T13:35:04Z INFO  websocket] Object({"feed": String("book"), "price": Number(10685.5), "product_id": String("PI_XBTUSD"), "qty": Number(518.0), "seq": Number(5095109), "side": String("sell"), "timestamp": Number(1565789702665)})
[2019-08-14T13:35:04Z INFO  websocket] book
[2019-08-14T13:35:04Z INFO  websocket] Object({"feed": String("book"), "price": Number(10565.0), "product_id": String("PI_XBTUSD"), "qty": Number(168632.0), "seq": Number(5095110), "side": String("buy"), "timestamp": Number(1565789702837)})
[2019-08-14T13:35:05Z INFO  websocket] ticker
[2019-08-14T13:35:05Z INFO  websocket] Object({"ask": Number(10579.5), "ask_size": Number(243388.0), "bid": Number(10579.0), "bid_size": Number(19529.0), "change": Number(-5.163835223452418), "dtm": Number(-18122), "feed": String("ticker"), "funding_rate": Number(0.000000000367517486), "funding_rate_prediction": Number(0.000000006293301999), "index": Number(10572.12), "last": Number(10578.5), "leverage": String("50x"), "markPrice": Number(10579.25), "maturityTime": Number(0), "next_funding_rate_time": Number(1565798400000), "openInterest": Number(29867422.0), "pair": String("XBT:USD"), "premium": Number(0.1), "product_id": String("PI_XBTUSD"), "relative_funding_rate": Number(0.000003866140625), "relative_funding_rate_prediction": Number(0.000066539270833333), "suspended": Bool(false), "tag": String("perpetual"), "time": Number(1565789703117), "volume": Number(93165442.0)})
```

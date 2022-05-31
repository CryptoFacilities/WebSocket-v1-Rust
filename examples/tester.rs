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

use std::env;

use cf_ws_v1::WebSocket;

fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let endpoint = env::args().nth(1).expect("no websocket endpoint provided");

    let mut ws = WebSocket::new(&endpoint, None, None);
    ws.subscribe("ticker", Some(&["PI_XBTUSD"]));

    for msg in ws.feed() {
        log::debug!("{msg:?}");
    }
}

use std::collections::BTreeMap; // similar to map in cpp
use ordered_float::OrderedFloat; // f64 type does not implement Ord
use serde_json::Value;

pub enum Side {
    Buy,
    Sell,
}

pub struct OrderBook {
    bids: BTreeMap<OrderedFloat<f64>, i64>,
    asks: BTreeMap<OrderedFloat<f64>, i64>,
}

impl OrderBook {
    // create order book
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    // takes in raw string message from kucoin websocket and processes it for the three fields
    pub fn process_message(&mut self, message: &str) {
        let parsed: Value = match serde_json::from_str(message) {
            Ok(json) => json,
            Err(_) => return, // Ignore invalid messages
        };

        // ignore pings
        if parsed["type"] == "pong" {
            return;
        }

        // Extract and parse the `change` field
        if let Some(change) = parsed["data"]["change"].as_str() {
            let parts: Vec<&str> = change.split(',').collect();
            if parts.len() != 3 {
                return;
            }

            let price: f64 = parts[0].parse().unwrap();
            let side = if parts[1] == "buy" { Side::Buy } else { Side::Sell };
            let quantity: i64 = parts[2].parse().unwrap();

            // Update order book
            self.ingest(price, side, quantity);
        }
    }

    // ingest update into order book
    fn ingest(&mut self, price: f64, side: Side, quantity: i64) {
        let price_key = OrderedFloat(price); // to be used as map key

        let book = match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };

        if quantity == 0 {
            book.remove(&price_key);
        } else {
            book.insert(price_key, quantity);
        }
    }

    // prints out 5 best in order book
    pub fn print(&self) {
        println!("\n=====================================================");
        println!("{: <10} {: <10}   |   {: <10} {: <10}", "Bid Price", "Qty", "Ask Price", "Qty");
    
        let mut bid_it = self.bids.iter().rev();
        let mut ask_it = self.asks.iter();
    
        for _ in 0..5 {
            let bid = bid_it.next();
            let ask = ask_it.next();
    
            // to handle None and for alignment
            match (bid, ask) {
                (Some((bid_price, bid_qty)), Some((ask_price, ask_qty))) => {
                    println!("{: <10} {: <10}   |   {: <10} {: <10}", bid_price, bid_qty, ask_price, ask_qty);
                }
                (Some((bid_price, bid_qty)), None) => {
                    println!("{: <10} {: <10}   |   {: <10} {: <10}", bid_price, bid_qty, "", "");
                }
                (None, Some((ask_price, ask_qty))) => {
                    println!("{: <10} {: <10}   |   {: <10} {: <10}", "", "", ask_price, ask_qty);
                }
                (None, None) => break,
            }
        }
    
        println!("=====================================================\n");
    }
}

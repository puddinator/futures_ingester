use reqwest::Client;
use serde_json::Value;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};

const KUCOIN_ENDPOINT: &str = "wss://ws-api-spot.kucoin.com/";
const PING_DURATION: u64 = 10;

// fetch WebSocket token
async fn get_kucoin_ws_token() -> Result<String, Box<dyn std::error::Error>> { // return different error types dynamically on the heap
    let client = Client::new();
    let url = "https://api.kucoin.com/api/v1/bullet-public";

    let response: Value = client.post(url)
        .send()
        .await?
        .json()
        .await?;

    let token = response["data"]["token"]
        .as_str()
        .ok_or("Failed to get WebSocket token")?
        .to_string();

    Ok(token)
}

// connect to KuCoin WebSocket and subscribe
async fn kucoin_connection(token: String) -> Result<(), Box<dyn std::error::Error>> { 
    let ws_url = format!("{}?token={}", KUCOIN_ENDPOINT, token);

    let (ws_stream, _) = connect_async(ws_url).await.expect("WebSocket connection failed");
    println!("Connected to KuCoin WebSocket!");
    let (mut write, mut read) = ws_stream.split();

    let subscribe_msg = r#"
    {
      "type": "subscribe",
      "topic": "/contractMarket/level2:ETHUSDTM",
      "privateChannel": false,
      "response": true
    }"#;

    write.send(Message::Text(subscribe_msg.to_string().into())).await?;

    // spawn pinger
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(PING_DURATION)).await;
            let ping_msg = r#"{"id": "123456789:)", "type": "ping"}"#;
            if let Err(e) = write.send(Message::Text(ping_msg.into())).await {
                eprintln!("Failed to send ping: {}", e);
                break;
            } else {
                println!("Sent ping message.");
            }
        }
    });

    while let Some(message) = read.next().await {
        match message {
            Ok(msg) => println!("Received: {}", msg),
            Err(e) => eprintln!("WebSocket error: {}", e),
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    match get_kucoin_ws_token().await {
        Ok(token) => {
            if let Err(e) = kucoin_connection(token).await {
                eprintln!("Error in WebSocket connection: {}", e);
            }
        }
        Err(e) => eprintln!("Error fetching WebSocket token: {}", e),
    }
}

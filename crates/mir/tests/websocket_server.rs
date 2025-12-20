use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("WebSocket server listening on {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
}

async fn handle_connection(stream: tokio::net::TcpStream) {
    let ws_stream = accept_async(stream).await.unwrap();
    let (mut sender, mut receiver) = ws_stream.split();

    while let Some(Ok(message)) = receiver.next().await {
        if let Message::Text(text) = message {
            println!("Received: {}", text);
            let echo_msg = Message::Text(format!("Echo: {}", text));
            if sender.send(echo_msg).await.is_err() {
                break;
            }
        }
    }
}
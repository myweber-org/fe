use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

async fn handle_connection(stream: TcpStream) {
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Failed to accept WebSocket connection");

    let (mut write, mut read) = ws_stream.split();

    while let Some(message) = read.next().await {
        match message {
            Ok(msg) => {
                if let Message::Text(text) = msg {
                    println!("Received: {}", text);
                    let response = format!("Echo: {}", text);
                    if let Err(e) = write.send(Message::Text(response)).await {
                        eprintln!("Failed to send message: {}", e);
                        break;
                    }
                } else if let Message::Close(_) = msg {
                    println!("Client closed connection");
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error reading message: {}", e);
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Failed to bind to address");

    println!("WebSocket echo server listening on ws://127.0.0.1:8080");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
}
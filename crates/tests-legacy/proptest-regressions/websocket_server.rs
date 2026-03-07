use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::accept_async;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket echo server listening on {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream).await {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }
    Ok(())
}

async fn handle_connection(stream: tokio::net::TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let ws_stream = accept_async(stream).await?;
    let (mut sender, mut receiver) = ws_stream.split();

    while let Some(msg) = receiver.next().await {
        let msg = msg?;
        if msg.is_close() {
            break;
        }
        if msg.is_text() || msg.is_binary() {
            sender.send(msg).await?;
        }
    }
    Ok(())
}use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::protocol::Message;

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:9001";
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("WebSocket echo server listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
}

async fn handle_connection(raw_stream: tokio::net::TcpStream) {
    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during WebSocket handshake");
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received text message: {}", text);
                let echo_msg = Message::Text(text);
                if ws_sender.send(echo_msg).await.is_err() {
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                println!("Client initiated close.");
                break;
            }
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                break;
            }
            _ => {}
        }
    }
    println!("Connection closed.");
}
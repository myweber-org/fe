use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::{SinkExt, StreamExt};
use std::error::Error;

pub async fn run_websocket_server(addr: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream.peer_addr()?;
        println!("New connection from: {}", peer);
        
        tokio::spawn(async move {
            match accept_async(stream).await {
                Ok(ws_stream) => {
                    println!("WebSocket connection established: {}", peer);
                    let (mut write, mut read) = ws_stream.split();
                    
                    while let Some(msg) = read.next().await {
                        match msg {
                            Ok(message) => {
                                if message.is_text() || message.is_binary() {
                                    if let Err(e) = write.send(message).await {
                                        eprintln!("Error sending message to {}: {}", peer, e);
                                        break;
                                    }
                                } else if message.is_close() {
                                    break;
                                }
                            }
                            Err(e) => {
                                eprintln!("Error receiving message from {}: {}", peer, e);
                                break;
                            }
                        }
                    }
                    println!("Connection closed: {}", peer);
                }
                Err(e) => eprintln!("Error during WebSocket handshake with {}: {}", peer, e),
            }
        });
    }
    Ok(())
}
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::protocol::Message;

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");

    println!("WebSocket echo server listening on {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
}

async fn handle_connection(raw_stream: tokio::net::TcpStream) {
    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("WebSocket handshake failed");

    let (mut sender, mut receiver) = ws_stream.split();

    while let Some(Ok(message)) = receiver.next().await {
        match message {
            Message::Text(text) => {
                println!("Received text message: {}", text);
                let echo_msg = Message::Text(format!("Echo: {}", text));
                if let Err(e) = sender.send(echo_msg).await {
                    eprintln!("Error sending echo: {}", e);
                    break;
                }
            }
            Message::Close(_) => {
                println!("Client disconnected");
                break;
            }
            _ => {
                println!("Received non-text message, ignoring");
            }
        }
    }
}
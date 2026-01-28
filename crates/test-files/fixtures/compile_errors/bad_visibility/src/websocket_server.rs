use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};

async fn handle_connection(stream: TcpStream) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Error during WebSocket handshake: {}", e);
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Err(e) = write.send(Message::Text(text)).await {
                    eprintln!("Error sending message: {}", e);
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                break;
            }
            Err(e) => {
                eprintln!("Error reading message: {}", e);
                break;
            }
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket echo server listening on {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }

    Ok(())
}use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{SinkExt, StreamExt};
use std::error::Error;

pub async fn run_websocket_server(addr: &str) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server listening on {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
    Ok(())
}

async fn handle_connection(stream: TcpStream) {
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Failed to accept WebSocket connection");

    let (mut write, mut read) = ws_stream.split();

    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(text)) => {
                println!("Received text message: {}", text);
                let echo_msg = Message::Text(format!("Echo: {}", text));
                if let Err(e) = write.send(echo_msg).await {
                    eprintln!("Failed to send echo: {}", e);
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                println!("Client closed connection");
                break;
            }
            Err(e) => {
                eprintln!("Error reading message: {}", e);
                break;
            }
            _ => {}
        }
    }
    println!("Connection handler finished");
}
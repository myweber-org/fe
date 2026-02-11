use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::accept_async;

async fn handle_connection(stream: TcpStream) {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    let (mut sender, mut receiver) = ws_stream.split();

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let echo_msg = Message::Text(format!("Echo: {}", text));
                sender.send(echo_msg).await.expect("Failed to send message");
            }
            Ok(Message::Close(_)) => {
                break;
            }
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("WebSocket server listening on ws://127.0.0.1:8080");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
}use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{accept_async, WebSocketStream};

async fn handle_connection(stream: TcpStream) {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    let (mut write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
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
            Ok(_) => {
                println!("Received non-text message, ignoring");
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
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("WebSocket echo server listening on ws://127.0.0.1:8080");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
}
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::protocol::Message;

async fn handle_connection(stream: tokio::net::TcpStream) {
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during WebSocket handshake");
    let (mut write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received text message: {}", text);
                let response = format!("Echo: {}", text);
                if let Err(e) = write.send(Message::Text(response)).await {
                    eprintln!("Error sending message: {}", e);
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                println!("Client disconnected");
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
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Failed to bind to address");

    println!("WebSocket server listening on ws://127.0.0.1:8080");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
}
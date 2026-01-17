use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};

async fn handle_connection(stream: TcpStream, addr: SocketAddr) {
    println!("New WebSocket connection from: {}", addr);
    
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Error during WebSocket handshake: {}", e);
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();

    while let Some(message) = read.next().await {
        match message {
            Ok(msg) => {
                match msg {
                    Message::Text(text) => {
                        println!("Received text message: {}", text);
                        let response = format!("Echo: {}", text);
                        if let Err(e) = write.send(Message::Text(response)).await {
                            eprintln!("Error sending message: {}", e);
                            break;
                        }
                    }
                    Message::Close(_) => {
                        println!("Client disconnected: {}", addr);
                        break;
                    }
                    _ => {
                        println!("Received non-text message from {}", addr);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading message: {}", e);
                break;
            }
        }
    }
}

pub async fn start_server(host: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr).await?;
    println!("WebSocket server listening on: {}", addr);

    loop {
        let (stream, addr) = listener.accept().await?;
        tokio::spawn(async move {
            handle_connection(stream, addr).await;
        });
    }
}

#[tokio::main]
async fn main() {
    let host = "127.0.0.1";
    let port = 8080;

    if let Err(e) = start_server(host, port).await {
        eprintln!("Server error: {}", e);
    }
}
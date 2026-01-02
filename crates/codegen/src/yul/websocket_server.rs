use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::accept_async;

async fn handle_connection(stream: TcpStream, addr: SocketAddr) {
    println!("New WebSocket connection from: {}", addr);
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Error during WebSocket handshake: {}", e);
            return;
        }
    };

    let (mut sender, mut receiver) = ws_stream.split();

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received text message: {}", text);
                let echo_msg = Message::Text(format!("Echo: {}", text));
                if let Err(e) = sender.send(echo_msg).await {
                    eprintln!("Error sending echo message: {}", e);
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                println!("Client closed the connection.");
                break;
            }
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                break;
            }
            _ => {}
        }
    }

    println!("Connection closed: {}", addr);
}

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("WebSocket echo server listening on: {}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, addr));
    }
}
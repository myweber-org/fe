use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{SinkExt, StreamExt};
use std::error::Error;

pub async fn run_websocket_server(addr: &str) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
    Ok(())
}

async fn handle_connection(raw_stream: TcpStream) {
    let addr = raw_stream.peer_addr().unwrap();
    println!("New connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Failed to accept WebSocket connection");

    let (mut write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received text from {}: {}", addr, text);
                let echo_msg = Message::Text(format!("Echo: {}", text));
                if let Err(e) = write.send(echo_msg).await {
                    eprintln!("Failed to send echo to {}: {}", addr, e);
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                println!("Connection closed by {}", addr);
                break;
            }
            Ok(_) => {
                println!("Received non-text message from {}", addr);
            }
            Err(e) => {
                eprintln!("Error receiving message from {}: {}", addr, e);
                break;
            }
        }
    }
    println!("Connection terminated: {}", addr);
}
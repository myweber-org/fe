
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{stream::StreamExt, SinkExt};
use std::error::Error;

pub async fn run_websocket_server(addr: &str) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
    Ok(())
}

async fn handle_connection(stream: TcpStream) {
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
                eprintln!("Error receiving message: {}", e);
                break;
            }
            _ => {}
        }
    }
}
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
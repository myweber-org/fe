use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::{SinkExt, StreamExt};

pub async fn run_websocket_server(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream.peer_addr()?;
        println!("New connection from: {}", peer);
        
        tokio::spawn(async move {
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
                    Ok(message) => {
                        if message.is_text() || message.is_binary() {
                            if let Err(e) = write.send(message).await {
                                eprintln!("Error sending message: {}", e);
                                break;
                            }
                        } else if message.is_close() {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error receiving message: {}", e);
                        break;
                    }
                }
            }

            println!("Connection closed: {}", peer);
        });
    }

    Ok(())
}
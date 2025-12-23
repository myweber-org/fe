use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::{SinkExt, StreamExt};
use std::error::Error;

pub async fn run_websocket_server(addr: &str) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let ws_stream = accept_async(stream).await?;
        let (mut write, mut read) = ws_stream.split();

        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(message) => {
                        if let Err(e) = write.send(message).await {
                            eprintln!("Error sending message: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error receiving message: {}", e);
                        break;
                    }
                }
            }
        });
    }
    Ok(())
}
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            match accept_async(stream).await {
                Ok(ws_stream) => {
                    let (mut write, mut read) = ws_stream.split();
                    while let Some(msg) = read.next().await {
                        match msg {
                            Ok(Message::Text(text)) => {
                                println!("Received text message: {}", text);
                                if let Err(e) = write.send(Message::Text(text)).await {
                                    eprintln!("Error sending message: {}", e);
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
                }
                Err(e) => eprintln!("WebSocket handshake error: {}", e),
            }
        });
    }
    Ok(())
}use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{SinkExt, StreamExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server listening on {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            let ws_stream = tokio_tungstenite::accept_async(stream).await;
            if let Ok(mut ws_stream) = ws_stream {
                while let Some(msg) = ws_stream.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            println!("Received: {}", text);
                            let reply = format!("Echo: {}", text);
                            if let Err(e) = ws_stream.send(Message::Text(reply)).await {
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
        });
    }
    Ok(())
}
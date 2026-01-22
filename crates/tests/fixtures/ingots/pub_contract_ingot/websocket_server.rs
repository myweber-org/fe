
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::accept_async;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("WebSocket server listening on ws://127.0.0.1:8080");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            match accept_async(stream).await {
                Ok(ws_stream) => {
                    let (mut write, mut read) = ws_stream.split();
                    while let Some(msg) = read.next().await {
                        match msg {
                            Ok(Message::Text(text)) => {
                                let echo_msg = Message::Text(format!("Echo: {}", text));
                                if let Err(e) = write.send(echo_msg).await {
                                    eprintln!("Failed to send message: {}", e);
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
                Err(e) => eprintln!("WebSocket handshake failed: {}", e),
            }
        });
    }
    Ok(())
}use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::{SinkExt, StreamExt};

pub async fn run_websocket_server(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server listening on {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream).await {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }
    Ok(())
}

async fn handle_connection(stream: tokio::net::TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let ws_stream = accept_async(stream).await?;
    let (mut write, mut read) = ws_stream.split();

    while let Some(message) = read.next().await {
        let message = message?;
        if message.is_text() || message.is_binary() {
            write.send(message).await?;
        }
    }
    Ok(())
}
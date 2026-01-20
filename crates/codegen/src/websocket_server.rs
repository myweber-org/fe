use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use std::sync::Arc;
use tokio::sync::broadcast;

async fn handle_connection(stream: TcpStream, sender: broadcast::Sender<String>) {
    let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
    let (mut write, mut read) = ws_stream.split();
    let mut receiver = sender.subscribe();

    let read_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = read.next().await {
            if let Message::Text(text) = msg {
                let _ = sender.send(text);
            }
        }
    });

    let write_task = tokio::spawn(async move {
        while let Ok(msg) = receiver.recv().await {
            if write.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let _ = tokio::join!(read_task, write_task);
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    let (sender, _) = broadcast::channel(32);
    let sender = Arc::new(sender);

    println!("WebSocket server listening on ws://127.0.0.1:8080");

    while let Ok((stream, _)) = listener.accept().await {
        let sender_clone = Arc::clone(&sender);
        tokio::spawn(async move {
            handle_connection(stream, sender_clone).await;
        });
    }
}
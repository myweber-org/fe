use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::accept_async;
use futures_util::{SinkExt, StreamExt};

type Clients = Arc<Mutex<HashMap<String, tokio_tungstenite::WebSocketStream<TcpStream>>>>;

async fn handle_connection(stream: TcpStream, clients: Clients) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Failed to accept WebSocket connection: {}", e);
            return;
        }
    };

    let addr = ws_stream.get_ref().peer_addr().unwrap().to_string();
    println!("New client connected: {}", addr);

    let (mut sender, mut receiver) = ws_stream.split();

    clients.lock().unwrap().insert(addr.clone(), sender);

    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            println!("Received from {}: {}", addr, text);
            let broadcast_msg = format!("{}: {}", addr, text);
            let mut clients_lock = clients.lock().unwrap();
            for (client_addr, client_sender) in clients_lock.iter_mut() {
                if *client_addr != addr {
                    let _ = client_sender.send(Message::Text(broadcast_msg.clone())).await;
                }
            }
        }
    }

    clients.lock().unwrap().remove(&addr);
    println!("Client disconnected: {}", addr);
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("WebSocket server listening on ws://127.0.0.1:8080");

    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    while let Ok((stream, _)) = listener.accept().await {
        let clients = clients.clone();
        tokio::spawn(async move {
            handle_connection(stream, clients).await;
        });
    }
}
use futures_util::{SinkExt, StreamExt};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::protocol::Message;

type Tx = futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>, Message>;
type Rx = futures_util::stream::SplitStream<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>>;
type PeerSet = Arc<Mutex<HashSet<Tx>>>;

async fn handle_connection(peer_set: PeerSet, stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>) {
    let (tx, mut rx) = stream.split();
    peer_set.lock().await.insert(tx);

    while let Some(Ok(msg)) = rx.next().await {
        match msg {
            Message::Text(text) => {
                let peers = peer_set.lock().await;
                for mut peer in peers.iter() {
                    let _ = peer.send(Message::Text(text.clone())).await;
                }
            }
            Message::Close(_) => break,
            _ => (),
        }
    }

    peer_set.lock().await.retain(|p| !p.is_closed());
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let peer_set: PeerSet = Arc::new(Mutex::new(HashSet::new()));

    println!("WebSocket server listening on ws://127.0.0.1:8080");

    while let Ok((stream, _)) = listener.accept().await {
        let ws_stream = tokio_tungstenite::accept_async(stream).await?;
        let peer_set = peer_set.clone();
        tokio::spawn(async move {
            handle_connection(peer_set, ws_stream).await;
        });
    }

    Ok(())
}
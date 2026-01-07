use std::net::TcpListener;
use std::io::{Read, Write};
use std::thread;

fn handle_client(mut stream: std::net::TcpStream) -> std::io::Result<()> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;

    let request = String::from_utf8_lossy(&buffer[..]);
    if !request.contains("Upgrade: websocket") {
        let response = "HTTP/1.1 400 Bad Request\r\n\r\n";
        stream.write_all(response.as_bytes())?;
        return Ok(());
    }

    let key_line = request.lines()
        .find(|line| line.starts_with("Sec-WebSocket-Key:"))
        .unwrap_or("");
    let key = key_line.trim_start_matches("Sec-WebSocket-Key:").trim();

    let accept_key = generate_accept_key(key);

    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
         Upgrade: websocket\r\n\
         Connection: Upgrade\r\n\
         Sec-WebSocket-Accept: {}\r\n\r\n",
        accept_key
    );
    stream.write_all(response.as_bytes())?;

    loop {
        let mut frame = [0; 2];
        stream.read_exact(&mut frame)?;

        let opcode = frame[0] & 0x0F;
        if opcode == 8 {
            break;
        }

        let payload_len = frame[1] & 0x7F;
        let mut payload = vec![0; payload_len as usize];
        stream.read_exact(&mut payload)?;

        let response_frame = create_frame(&payload);
        stream.write_all(&response_frame)?;
    }

    Ok(())
}

fn generate_accept_key(key: &str) -> String {
    use sha1::{Sha1, Digest};
    let mut hasher = Sha1::new();
    hasher.update(key.as_bytes());
    hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    let result = hasher.finalize();
    base64::encode(&result)
}

fn create_frame(payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::new();
    frame.push(0x81);
    frame.push(payload.len() as u8);
    frame.extend_from_slice(payload);
    frame
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("WebSocket server listening on ws://127.0.0.1:8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    let _ = handle_client(stream);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
    Ok(())
}
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use bytes::Bytes;
mod db;
use db::db::Db;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Added -> std::io::Result<()> to allow using '?'
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    println!("Server listening on port 6379");

    let db = Db::new();

    loop {
        let (socket, _) = listener.accept().await?;
        print!("New connection established\n");
        let db = db.clone();

        tokio::spawn(async move {
            handle_connection(socket, db).await;
        });
    }
}

async fn handle_connection(mut socket: TcpStream, db: Db) {
    let mut buffer = [0; 4096];

    loop {
        // 1. Read data from socket
        let n = match socket.read(&mut buffer).await {
            Ok(0) => return, // Connection closed
            Ok(n) => n,
            Err(e) => {
                eprintln!("failed to read from socket; err = {:?}", e);
                return;
            }
        };

        // 2. Parse input
        // FIX: Changed &buf to &buffer
        let input = String::from_utf8_lossy(&buffer[..n]);
        let frames = parse_resp(&input);

        if frames.is_empty() {
            continue;
        }

        let cmd_name = frames[0].to_uppercase();
        match cmd_name.as_str() {
            "PING" => {
                let response = "+PONG\r\n";
                if let Err(e) = socket.write_all(response.as_bytes()).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
            "SET" => {
                if frames.len() != 3 {
                    let response = "-ERR wrong number of arguments for 'SET' command\r\n";
                    let _ = socket.write_all(response.as_bytes()).await;
                } else {
                    let key = frames[1].to_string();
                    let value = Bytes::from(frames[2].to_string());
                    db.set(key, value);
                    let response = "+OK\r\n";
                    if let Err(e) = socket.write_all(response.as_bytes()).await {
                        eprintln!("failed to write; err = {:?}", e);
                        return;
                    }
                }
            }
            "GET" => {
                if frames.len() != 2 {
                    let response = "-ERR wrong number of arguments for 'GET' command\r\n";
                    let _ = socket.write_all(response.as_bytes()).await;
                } else {
                    let key = frames[1].to_string();
                    match db.get(key) {
                        Some(value) => {
                            // Header
                            let header = format!("${}\r\n", value.len());
                            if let Err(_) = socket.write_all(header.as_bytes()).await { return; }
                            // Body
                            if let Err(_) = socket.write_all(&value).await { return; }
                            // Footer
                            if let Err(_) = socket.write_all(b"\r\n").await { return; }
                        }
                        None => {
                            let response = "$-1\r\n";
                            let _ = socket.write_all(response.as_bytes()).await;
                        }
                    }
                }
            }
            "COMMAND" | "DOCS" => {
                // Clients often send these on startup; ignore them gracefully
                let _ = socket.write_all(b"+OK\r\n").await;
            }
            _ => {
                let msg = format!("-ERR unknown command '{}'\r\n", cmd_name);
                let _ = socket.write_all(msg.as_bytes()).await;
            }
        }
    }
}

// FIX: Changed return type from Vector<String> to Vec<String>
fn parse_resp(input: &str) -> Vec<String> {
    let mut lines = input.lines();
    let mut result = Vec::new();

    // Check if it starts with array indicator
    if let Some(line) = lines.next() {
        if !line.starts_with('*') {
            return result;
        }
    }

    while let Some(line) = lines.next() {
        if line.starts_with('$') {
            // The next line contains the actual data
            if let Some(data) = lines.next() {
                result.push(data.to_string());
            }
        }
    }
    result
}
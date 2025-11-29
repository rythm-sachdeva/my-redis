use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use bytes::Bytes;
use crate::db::db::Db;
use crate::connection::respHandler::parse_resp;
pub async fn handle_connection(mut socket: TcpStream, db: Db) {
    let mut buffer = [0; 4096];

    loop {
        let n = match socket.read(&mut buffer).await {
            Ok(0) => return, 
            Ok(n) => n,
            Err(e) => {
                eprintln!("failed to read from socket; err = {:?}", e);
                return;
            }
        };
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
                            let header = format!("${}\r\n", value.len());
                            if let Err(_) = socket.write_all(header.as_bytes()).await { return; }
                            if let Err(_) = socket.write_all(&value).await { return; }
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
                let _ = socket.write_all(b"+OK\r\n").await;
            }
            _ => {
                let msg = format!("-ERR unknown command '{}'\r\n", cmd_name);
                let _ = socket.write_all(msg.as_bytes()).await;
            }
        }
    }
}


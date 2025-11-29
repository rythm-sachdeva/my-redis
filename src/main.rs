use tokio::net::{TcpListener};
mod db;
use db::db::Db;
mod connection;
use connection::connectionHandler::handle_connection;

#[tokio::main]
async fn main() -> std::io::Result<()> {
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


use db::Db;
use tokio::io::{AsyncReadExt,AsyncWriteExt};
use tokio::net::{TcpListener,TcpStream};
use bytes::BytesMut;


#[tokio::main]
async fn main(){
     
     let listener = TcpListener::bind("127.0.0.1:6379").await?;
     let db = Db::new();
    

     loop {

      let (socket,_) = listener.accept().await?;   
      let db = db.clone();


     }



}

use bytes::{BytesMut,Bytes,Buf};
use tokio::net::{TcpListener,TcpStream};
use tokio::io::AsyncRead;

use std::collections::HashMap;
use std::sync::{Arc,Mutex};

type Db = Arc<Mutex<HashMap<String,Vec<Bytes>>>>;



#[tokio::main]
async fn main(){
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let db:Db = Arc::new(Mutex::new(HashMap::new()));
    println!("Listening at 127.0.0.1:6379");   
    loop{ 
        let db = db.clone();
        let (socket,_) = listener.accept().await.unwrap();
        tokio::spawn(async move{
            process_socket(socket,db).await;    
        });
    }

}




async fn process_socket(socket:TcpStream, db:Db){
    let mut buf = [0u8;4096];  
    loop {
        socket.readable().await.unwrap();
        match socket.try_read(&mut buf){
            Ok(0) => break,
            Ok(n) =>{
                let d = buf.iter().map(|c|{*c as char}).collect::<String>();
                println!("read {}",d);
            }

            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }

            Err(e) => {
            }
        } 
    }
}







/*
 *  work on bytes 
 *
 * */

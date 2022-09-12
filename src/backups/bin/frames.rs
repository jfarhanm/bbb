use bytes::BytesMut;
use tokio::net::TcpStream;

use bytes::Bytes;

pub enum Frame{
    Topic(String),
    Error(String),
    Integer(u64),
    Bulk(Bytes),
    Float(f64),
    Null,
    Array(Vec<Frame>)
}


struct Connection{
    stream:TcpStream
}












#[tokio::main]
async fn main(){
    
}



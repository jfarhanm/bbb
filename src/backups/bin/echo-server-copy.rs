use tokio::net::TcpListener;
use tokio::io::{AsyncRead,AsyncWrite,self};



#[tokio::main]
async fn main() -> io::Result<()>{
    let listener = TcpListener::bind("127.0.0.1:6142").await?;
    loop{
        let (mut socket,_) = listener.accept().await?;
        tokio::spawn(async move{

        });
    }
}

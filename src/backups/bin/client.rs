use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::net::TcpStream;
use std::io::prelude::*;



#[derive(Serialize, Deserialize)]
struct LaserData {
    name: String,
    data:Vec<f32>,
}

fn main() -> Result<()> {
    let data = LaserData{
        name:String::from("SICK"),
        data:(0..1000).map(|x|{ x as f32 }).collect::<Vec<f32>>()
    };    
    
    let z= serde_json::to_vec(&data)?;

    let mut strm = TcpStream::connect("127.0.0.1:8008").unwrap();
    strm.write_all(&z).unwrap();
    
    let mut buf = [0u8;4096];
    
    strm.read(&mut buf).unwrap();
    let v = buf.iter().filter(|m|{**m>0}).map(|m|{*m as char}).collect::<String>();


    println!("{:?}",v);

    Ok(())
}

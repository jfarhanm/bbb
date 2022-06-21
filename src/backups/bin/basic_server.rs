use bytes::{Bytes,BytesMut,Buf};
use std::net::{TcpListener,TcpStream};
use std::sync::{Arc,Mutex};
use std::collections::VecDeque;
type centralStore = Arc<Mutex<VecDeque<Bytes>>>;



// Essentially the transmitted DATA is JSON; 
// taken from redis specs 
// For Description(Simple String) the first byte of the reply is "+" [JSON schema]
// For Errors the first byte of the reply is "-"
// For Bulk Strings the first byte of the reply is "$"



// let's define this 
// I get my data 

use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender,Receiver};
use std::thread::Thread;

pub enum MPasser{
    Request((usize,Bytes)),
    Respone((usize,Bytes)),
    Broadcast((Vec<usize>,Bytes)),
    BroadcastData(Arc<Bytes>),
    BroadcastAck,
    Error
}


struct BroadcastHandle{
    id:usize,
    recievers:Vec<usize>,
    data:Bytes
}



// Make a ROS service 
pub fn main(){
    let mut listener = TcpListener::bind("127.0.0.1:8008").unwrap();
    let datathreads :Arc<Vec<Mutex<(Sender<Bytes>,Receiver<Bytes>)>>> = Arc::new(Vec::new());
    let mut references : Arc<Vec<Mutex<Option<usize>>>> = Arc::new(Vec::new());
        
    let mut messages:Arc<Mutex<VecDeque<MPasser>>> = Arc::new(Mutex::new(VecDeque::new())) ;
    let mut datas:Vec<(Receiver<MPasser>,Sender<MPasser>)> = Vec::new();
    let mut broadcast_queue :VecDeque<BroadcastHandle> = VecDeque::new();


    for it in 0..10{
        for m in 0..2{
           let (tx,rx) = channel();
           let (tx2,rx2) = channel();
           datas.push((rx,tx2));
           std::thread::spawn(move||{
                let a = tx;
                let b = rx2;
           });
        }

        for d in datas.iter(){
            if let Ok(v) = d.0.try_recv(){
                let num = 2;
                datas[num].1.send(MPasser::Error);           
            }
        }

        for m in broadcast_queue.pop_front(){
            let tt= Arc::new(m.data);
            for n in m.recievers.iter(){
                datas[*n].1.send(MPasser::BroadcastData(tt.clone()));
            }
        }
    }








    for m in 0..2{
        let z = messages.clone();
        let index = m;
        std::thread::spawn(move||{
            // scan for new requests 
            let mut mate = z.lock().unwrap();
            if let Some(v) = (*mate).front(){
                if let MPasser::Request(num)=v{
                    //do_smth
                }else{

                } 
            }

            // In a different thread, or the same , add a request  
            (*mate).push_back(MPasser::Error);
        });
    }




    


    // .lock().unwrap().push(channel());    
    for m in 0..10{
        let nt = datathreads.clone();
        std::thread::spawn(move||{
            let a = nt;
            let mut r = a[2].lock().unwrap();
            *r = channel();
            let dt = Bytes::new();
            (*r).0.send(dt);
            //m.= channel();
        });
    }




    for stream in listener.incoming(){
        match stream{
            Ok(stream) => {
                handle(stream)
            }
            Err (e) =>
            {
                println!("Connection failed");
            }
        }
    }
}






// multiple +s is not smth you will encounter 
use std::io::Read;
use bytes::{BufMut};
pub fn handle(mut stream:TcpStream){
    //get a json request 
    //return a json response 
    let mut buf = bytes::BytesMut::with_capacity(10); 
    let mut cursor = 0; 
    let INIT_SIZE=96;
    println!("{}",buf.len());

    // just so you know , multiple +s will fuck you
    // stopgap until a good solution comes along 
    // or until I get better internet 
    buf.resize(96,0);
    while let Ok(n) = stream.read(&mut buf[cursor..]){
        if n==0{
            // no bytes read cond 
            if buf.is_empty(){
                println!("EMPTY BUFF");
            }else{
            }
        }else{
            cursor+=n;
            if cursor == buf.remaining(){
                buf.resize(buf.len()*2,0);
                println!("BUFLEN {}",buf.len());
            }

            
            if let Ok(index) = parse(&mut buf,cursor){
                let cursor_final = cursor - index-1;
                let (mut p, q) = buf.split_at_mut(index+1);
                p.chunk_mut()[0..cursor_final].copy_from_slice(&q[0..cursor_final]); 
                buf.truncate(( (cursor_final/INIT_SIZE) +1 )*INIT_SIZE); 
                cursor = cursor_final;
            }
            
            let z = std::str::from_utf8(&buf).unwrap();
            println!("{}",z);
            println!("LEN {}",buf.len());
        }
    }
}




use std::io::Cursor;
pub fn parse(buf: &mut BytesMut, curs:usize)->Result<usize ,()>{
    let mut cur =  Cursor::new(&buf[..]);
    let mut index=0;
    while(index<curs){
        let d = cur.get_u8();
        if d==b'+'{
            return Ok(index)
        }
        index+=1;
    }
    Err(()) 
}







pub enum MessageTypes{
    Description(Bytes),
    Bulk(Bytes),
    Error(String)
}




// NOTE (jfarhanm) : Check the len and the ind variables , they could fuck up ; bad !
pub fn shameless_copied_parse(buf: &mut BytesMut, curs:usize)->Result<(MessageTypes,usize),()>{
   let mut cur = Cursor::new(&buf[..]);
   match cur.get_u8(){
       b'+'=>{
            // JSON format string
           let mut ind = 0;
           while(cur.get_u8()!=b'\n'){
                ind+=1;
           }
           if(cur.get_u8()==b'\n'){
               let jdata =  MessageTypes::Description(Bytes::copy_from_slice(&buf[..ind+1]) ); 
               return Ok((jdata,ind+1));
           }else{
               return Err(())
           }
       } 

       b'$'=>{
            // huge binary JSON stream
            let len = cur.get_u64();
            if (curs as u64) < len{
                return Err(())
            }else{
                let jdata =  MessageTypes::Description(Bytes::copy_from_slice(&buf[..(len as usize) ] ) ); 
                return Ok((jdata,len as usize))
            }
       }
        

       b'&'=>{
            // Defines service 
           let mut ind = 0;
           while(cur.get_u8()!=b'\n'){
                ind+=1;
           }
           if(cur.get_u8()==b'\n'){
               let jdata =  MessageTypes::Description(Bytes::copy_from_slice(&buf[..ind+1]) ); 
               return Ok((jdata,ind+1));
           }else{
               return Err(())
           }
       } 

       _ =>{}
   }
   Err(())
}















fn pparse(buf: &mut Cursor<&[u8]>){
    
}




fn get_u8(buf: &mut Cursor<&[u8]>)->Result<u8,()>{
    if !buf.has_remaining(){
        return Err(())
    }
    Ok(buf.get_u8())
}







pub fn maino(){
    let mut buf = bytes::BytesMut::with_capacity(4096);
    println!("{}",buf.len());
    
    buf.resize(96,0);

    
    buf.advance(90);
    
    if buf.is_empty(){
        println!("BUF is not mt");
    }

    
    let mut m = vec![0,1,2,3];
    




    buf.advance(6);
    
    if buf.is_empty(){
        println!("BUF is empty");
    }

    println!("{}",buf.len());
}

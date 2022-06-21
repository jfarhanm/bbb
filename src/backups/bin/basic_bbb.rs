use bytes::{Bytes,BytesMut,Buf};
use std::net::{TcpListener,TcpStream};
use std::sync::{Arc,Mutex,RwLock};
use std::collections::VecDeque;
use std::collections::HashMap;
type centralStore = Arc<Mutex<VecDeque<Bytes>>>;
use std::io::Cursor;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender,Receiver};
use std::thread::Thread;


mod protocol_defs{
    pub const REQ:u8 = 0x51;    //Q
    pub const RESP:u8 = 0x41;   //A
    pub const ACK:u8 = 0x69;
}
// message types
// TODO (jfarhanm): Change name
// NOTE (jfarhanm): Possibly deprecated 
type TaskID = usize;
pub enum MPasser{
    Request{
        sender: TaskID,
        receiver: TaskID,
        data:Bytes
    },
    Response{
        receiver: TaskID,
        data:Bytes
    },
    Broadcast((Vec<TaskID>,Bytes)),
    BroadcastData(Arc<Bytes>),
    Register((usize,String)),
    GetID((TaskID,String)),
    IDResponse(Option<TaskID>),
    BroadcastAck,
    Error,
    Blabber(String),
    TestVal( (FrameTypes,TaskID)),
    DataPacket{
        sender:TaskID,
        receiver: TaskID,
        data:FrameTypes
    }
}

struct BroadcastHandle{
    id:usize,
    receivers:Vec<usize>,
    data:Bytes
}




// NOTE: Main Code 
// Make a ROS service 
// TODO: handle_listener does not need to be a thing , rx.recv() will block anyway 
// Get the datas.iter() in  a separate thread 
pub fn main(){
    let mut listener = TcpListener::bind("127.0.0.1:8008").unwrap();
    let (tx,rx) = channel::<TcpStream>();
    std::thread::spawn(move||{
        handle_listener(rx)
    });
    for stream in listener.incoming(){
        match stream{
            Ok(stream) => {
                println!("New Connection {}",stream.peer_addr().unwrap());
                match tx.send(stream){
                    Ok(a) =>{
                        println!("a {:?}",a);
                    }
                    Err(e) =>{
                        println!("{:?}",e);
                    }
                };
            }
            Err (e) =>
            {
                println!("Connection failed");
            }
        }
    }
}



pub fn printframe(frame:&FrameTypes){
  match frame{
        FrameTypes::RequestFrame{service_id,message_id,message} =>{
            println!("REQUEST FRAME: \nSER:{}\nMSGID:{}\nMSG:{}",service_id,message_id,message);
        },
        FrameTypes::ResponseFrame{message_id,message}=>{
            println!("RESPONSE FRAME: \nMSGID:{}\nMSG:{}",message_id,message);
        }
        _ =>{
            println!("Other")
        }
    }
}



// test code
// OK this one handles the listener 
pub fn handle_listener(rx:Receiver<TcpStream>){
    let mut idmaker = 0;

    let mut datas: Vec<(Receiver<MPasser>,Sender<MPasser>)>= Vec::new();    
    loop {
        match rx.try_recv(){        // NOTE: This works , for now 
            Ok(stream) => {      // NOTE: This blocks!
                println!("Connected to {}",stream.peer_addr().unwrap()); 
                let ((txaway,rxhome),(txhome,rxaway)) = (channel(),channel());
                std::thread::spawn(move||{
                    test_handle_stream(stream,rxaway,txaway,idmaker);
                });
                datas.push((rxhome,txhome));
                idmaker+=1;
            }
            Err(e) => {
                //println!("RECV ERROR {:?}",e);
            } 
        }

    // NOTE : THIS HAS TO ALWAYS RUN !
    datas.iter().for_each(|m|{              //TODO basic test here 
        let mut iterator = m.0.try_iter();
            while let Some(v) = iterator.next(){
                //m.1.send(v);
                if let MPasser::DataPacket{sender,receiver, data} = v{
                    println!("Found Datas");
                    datas[receiver].1.send( MPasser::DataPacket{sender,receiver,data});
                }

            } 
        });
    }
}



// test code 
pub fn test_handle_stream(mut stream:TcpStream,mut rx:Receiver<MPasser>,mut tx:Sender<MPasser>,id:usize){
    let mut node = if id==0{
        Node::new_service()
    }else{
        Node::new_client()
    };
    node.id = id;
    node.handle_node(stream,rx,tx);
}





// NOTE: Everything pertaining to a frame
// NOTE: Message is not u16 
pub enum FrameTypes{
    RequestFrame{
        service_id:u8,
        message_id:u32,
        message:u16
    },
    ResponseFrame{
        message_id:u32,
        message:u16
    },
    FrameParseError,
    DefaultFrame,
    NoData
}


pub enum FrameParseResult{
    Ok(FrameTypes),
    IncompleteFrameError,
    FrameParseError,
    ConnectionError,
    NoData 
}








// NOTE: Everything pertaining to a node 
use protocol_defs::*;
const BUFFER_SIZE:usize=4;
pub enum NodeType{
    Service{id:TaskID},
    Req{id:TaskID},
    DefaultNode
}

pub struct Node{
    topic:Option<String>,
    id:TaskID,
    node_type:NodeType,
    buffer:Vec<u8>,         // Bytes in the Future 
    buffer_read_cur:usize
}

impl Node{
    pub fn new()->Node{
        Node{
            topic:None,
            id:0,
            node_type:NodeType::DefaultNode,
            buffer:vec![0;BUFFER_SIZE],
            buffer_read_cur:0
        }
    }


   // TODO client is 0 , service is 1  
    pub fn new_client()->Node{
        Node{
            topic:None,
            id:0,
            node_type:NodeType::Req{id:0},
            buffer:vec![0;BUFFER_SIZE],
            buffer_read_cur:0
        }
    }


    pub fn new_service()->Node{
        Node{
            topic:None,
            id:0,
            node_type:NodeType::Service{id:1},
            buffer:vec![0;BUFFER_SIZE],
            buffer_read_cur:0
        }
    }

    
    // NOTE: When pyramid scheme happens , handle 
    pub fn check_resize_buf(&mut self){
        if self.buffer.len()==self.buffer_read_cur{
            self.buffer.resize(self.buffer.len()*2,0);
        }
    }
    

    // for testing purposes same as get_frames
    // TODO: Delete After
    #[deprecated]
    pub fn _test_process_stream(&mut self){
        let read_n = 3; 
        let mut index = 0;
        let mut tcp_test_data:Vec<u8> = vec![
                                REQ ,20 ,2 ,1 ,0 ,1   ,127 ,126   ,b'\n' ,b'\n',
                                RESP,0  ,1 ,0 ,1 ,127 ,126 ,b'\n' ,b'\n',
                                REQ,20,0,1,0,1,127,126,b'\n',b'\n',
                                RESP,0,1,0,1,127,126,b'\n',b'\n'
                                    ];
        while index<=14{
            let mut read_data=0;
            for m in 0..read_n{
                if self.buffer.len()>index{
                    self.buffer[index] = tcp_test_data[index];
                    index+=1;
                    read_data+=1;
                }
            }
            self.buffer_read_cur+=read_data;
            self.check_resize_buf();
            match self.get_frames(read_data){
                FrameParseResult::IncompleteFrameError=>{
                    println!("INCOMPLETE FRAME")
                },
                FrameParseResult::Ok(d)=>{
                    match d{
                        FrameTypes::RequestFrame{service_id,message_id,message} =>{
                            println!("REQUEST FRAME: \nSER:{}\nMSGID:{}\nMSG:{}",service_id,message_id,message);
                        },
                        
                        _=>()
                    };
                
                    break;
                }

                _ =>{
                    break;
                }
            }
            //println!("BUFFER SIZE{}",self.buffer.len());
            println!("BUFFER {:?}",self.buffer);
        }
    }
    

    // TODO : For future reference, 
    // To improve efficiency move self.buffer() rather than copy 
    pub fn get_self_buffer(&mut self)->Vec<u8>{
        unimplemented!()
    }


    // First Real world Test 
    pub fn handle_stream(&mut self, stream:&mut TcpStream)->FrameTypes{
        let mut frame_result = FrameTypes::DefaultFrame;
        loop{//loop{
            //println!("Looping");
            let out = self.process_stream(stream);
            //println!("BUFFER : {:?}",self.buffer);
            match out{
                FrameParseResult::IncompleteFrameError =>{
                    //println!("Incomplete frame")
                }
                // NOTE Here is where I deleted the print statements 
                FrameParseResult::Ok(f) => {
                    match f{
                        FrameTypes::RequestFrame{service_id,message_id,message} =>{
                            //println!("REQUEST FRAME: \nSER:{}\nMSGID:{}\nMSG:{}",service_id,message_id,message);
                            stream.write(&[ACK,10,10,10]);
                            frame_result = FrameTypes::RequestFrame{service_id,message_id,message};
                        },
                        FrameTypes::ResponseFrame{message_id,message}=>{
                            //println!("RESPONSE FRAME: \nMSGID:{}\nMSG:{}",message_id,message);
                            stream.write(&[ACK,10,10,10]);
                            frame_result = FrameTypes::ResponseFrame{message_id,message};
                        }
                        _ =>{
                            println!("Other")
                        }
                    }
                    break;
                }
                _=>{
                        //println!("Other FrameParseResult"); 
                        frame_result = FrameTypes::NoData;
                        break;
                    }
            }
        }
        //println!("Exiting Loop");
        frame_result
    }


    // NOTE: Ground rule : You may send next request only after you get an ACK for this one 
    pub fn process_stream(&mut self, stream:&mut TcpStream)->FrameParseResult{
        //println!("PROCESSING STREAM"); 
        match stream.read(&mut self.buffer[self.buffer_read_cur..]){
            Ok(n) =>{
                if n==0{
                    return FrameParseResult::NoData
                }
                // IMMEDIATE TODO: handle no bytes read issue 
                //println!("N is {}",n);
                self.buffer_read_cur+=n;
                self.check_resize_buf();
                self.get_frames(n)
            }
            Err(_) =>{
                //println!("Connection Error");
                FrameParseResult::ConnectionError    
            }
        } 
    }


    //  TODO: return processed frame as message 
    pub fn get_frames(&mut self, bytes_read:usize)->FrameParseResult{
        let mut cur = std::io::Cursor::new(&self.buffer[..self.buffer_read_cur]);   // error here 
        let frame_data =  Self::parse(&mut cur);
        let cur_pos = cur.position();
        if let Ok(resp) = frame_data{
            self.buffer_read_cur=0;
            FrameParseResult::Ok(resp)    
        }else{
            FrameParseResult::IncompleteFrameError
        }
    }
    
    // TODO : make another module 
    // TODO : move this to another directory 
    // NOTE (jfarhanm) : In caller function, take cursor.position()
    // TODO (jfarhanm) : Err(()) returns when incomplete Frame has been used
    //                   if frame parsing goes wrong FrameTypes::FrameParseError is returned
    pub fn parse(cursor:&mut std::io::Cursor<&[u8]>)->Result<FrameTypes,()>{
        if let Ok(val) = get_u8(cursor){
            match val{
                REQ =>{
                    if cursor.remaining()>=9{        //Problematic 
                        //println!("Works");
                        let service_id = cursor.get_u8();
                        let message_id = cursor.get_u32();
                        let message = cursor.get_u16();
                        cursor.get_u8();
                        if cursor.get_u8()!=b'\n'{
                            return Ok(FrameTypes::FrameParseError);
                        }else{
                            return Ok(FrameTypes::RequestFrame{
                                service_id,
                                message_id,
                                message
                            });
                        }
                    }else{
                        println!("Else Error");
                        return Err(())
                    }
                }

                RESP =>{                        // Problematic 
                    if cursor.remaining()>=8{
                        let message_id = cursor.get_u32();
                        let message = cursor.get_u16();
                        cursor.get_u8();
                        if cursor.get_u8()!=b'\n'{
                            return Ok(FrameTypes::FrameParseError)
                        }else{
                            return Ok(FrameTypes::ResponseFrame{
                                message_id,
                                message
                            });
                        }
                    }else{
                        return Err(())
                    }
                }

                _   =>{
                    return Ok(FrameTypes::FrameParseError);
                }
            }
        }
        Err(())
    }
    

    // TODO when initialisation protocol has been created 
    pub fn handle_initialisation(&mut self,mut stream:TcpStream,mut rx:Receiver<MPasser>,mut tx:Sender<MPasser> ){
        if let NodeType::DefaultNode = self.node_type{
            let output  = self.handle_stream(&mut stream);
            /*Init stream --handle this*/
        }
    }
    
    pub fn handle_node( &mut self, mut stream:TcpStream,mut rx:Receiver<MPasser>,mut tx:Sender<MPasser>){
        match self.node_type{
            NodeType::Service{id} =>{
                self.handle_service(stream,rx,tx);
            }
            NodeType::Req{id} =>{
                self.handle_client(stream,rx,tx);
            },
            NodeType::DefaultNode => {}
        }
    }

    pub fn handle_client( &mut self, mut stream:TcpStream,mut rx:Receiver<MPasser>,mut tx:Sender<MPasser>){
        loop{
            let recv_data = self.handle_stream(&mut stream);
            if let FrameTypes::NoData = recv_data{}else{
                println!("Sending to client");
                printframe(&recv_data);
                tx.send(
                    MPasser::DataPacket{sender:self.id, receiver:0,data:recv_data}    
                );
                
                // waits for receiving data 
                let out = rx.recv().expect("handle_client receiver error");
                if let MPasser::DataPacket{sender,receiver,data} = out{
                    println!("From handle_client:");
                    printframe(&data);
                }; 
                stream.write_all(&['C' as u8, 'L' as u8,'I' as u8]);
            }
        }
        println!("Exiting Thread");
    }



    pub fn handle_service(&mut self, mut stream:TcpStream,mut rx:Receiver<MPasser>,mut tx:Sender<MPasser>  ){
        stream.write_all(&['S' as u8, 'E' as u8, 'R' as u8]);
        let mut rxid:Option<TaskID> = None;
        loop{
            // NOTE :  goes into infinite loop here
            println!("Starting Loop");
            let recv_data = rx.recv().expect("Could not read from receiver"); //NOTE why does this not block ?
            

            if let MPasser::DataPacket{sender,receiver,data}  = recv_data{
                println!("From Handle service");
                rxid = Some(sender);
                printframe(&data);
            }
            stream.write_all(&['D' as u8,'U' as u8]);   // TODO :  change 
            std::thread::sleep_ms(1000);
            

            // get DU 
            // reply with response 
            let out  = self.handle_stream(&mut stream);
            println!("handle_service loop test");
            
            //printframe(&out); 
            if let Some(id) = rxid{
                tx.send( MPasser::DataPacket{sender:self.id,receiver:id,data:out});
                rxid  = None;
            }
            
        } 
    }

    pub fn self_register(){}
    pub fn run(&mut self)->Result<MPasser,()>{
        unimplemented!()
    }
}



fn get_u8(buf: &mut Cursor<&[u8]>)->Result<u8,()>{
    if !buf.has_remaining(){
        return Err(())
    }
    Ok(buf.get_u8())
}


use std::io::{Read,Write};
use bytes::BufMut;


#[cfg(test)]
mod tests
{
    #[test]
    fn test_parser(){
        let my_data = ""; //what does my packet look like ?
    }

}








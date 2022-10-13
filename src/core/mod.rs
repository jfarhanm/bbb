use std::net::{TcpListener,TcpStream};
use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender,Receiver};
use std::thread::Thread;

use crate::basic_bbb::conn;
use crate::basic_bbb::node;
use crate::basic_bbb::message;
use message::IPCMessage;

pub struct NodeChannels{
    rx:Receiver<IPCMessage>,
    tx:Sender<IPCMessage>,
}


// TODO check if handle_listener_array works 
pub fn run(){
    let mut listener = TcpListener::bind("127.0.0.1:8008").unwrap();
    let (tx,rx) = channel::<TcpStream>();
    std::thread::spawn(move||{
        handle_listener_array(rx)
    });
    
    for stream in listener.incoming(){
        match stream{
            Ok(stream) => {
                println!("New Connection {}",stream.peer_addr().unwrap());
                match tx.send(stream){
                    Ok(a)   =>  {println!("TX Send OK {:?}",a);}
                    Err(e)  =>  {println!("TX Send Error{:?}",e);}
                };
            }
            Err (e) =>
            {   println!("Connection failed");}
        }
    }
}

// PRIORITY : Reverify and create a proper spec for endianness of service IDs and the likes 
// TODO PRIORITY : Create a proper REG_CALLER_ACK
// SOFT TODO : Status for services 
// SOFT TODO : Multicaller services 

// TODO PRIORITY : On connecting a caller, if the service does not exist the main thread fails.
//                  - problems caused by this :
//                      Causes a delay in the main thread; ie if multiple things have to start they
//                      cannot  do so easily
//                  - Fix : ?
// TODO : improve errors 
// TODO : Explore crossbeam 
// TODO : Create a manager struct to manage 
//              - taking data from channels
//              - service list and clients connected
//                      
// TODO : Eliminate the need for this thread 
// TODO : How to handle the names 
// TODO : How to create and delete nodes 
// TODO : How to handle dependent nodes on creation and deletion.
// GROUND RULES:
//      You cannot delete a service  unless all callers to it have been deleted 
// TODO : A HashSet 
pub fn handle_listener_thread(rx:Receiver<TcpStream>){
    let mut idmaker = node::IDMaker::new();
    let mut datas: Vec<(Receiver<IPCMessage>,Sender<IPCMessage>)>= Vec::new(); 
    
    let mut node_channels:HashMap<usize,NodeChannels> = HashMap::new();
    let mut service_list :HashMap<String,usize> = HashMap::new();

    loop { 
        match rx.try_recv(){        
            Ok(stream) => {      
                println!("Connected to {}",stream.peer_addr().unwrap());
                let ((txaway,rxhome),(txhome,rxaway)) = (channel(),channel());
                let  id = idmaker.id();
                if let Ok(mut node_) = node::Node::new().with_id(id).with_stream(stream).register(&service_list){    
                    std::thread::spawn(move||{
                        let mut chan = node::comm_channels::MpscChan::<IPCMessage>::new(rxaway,txaway);
                        node_.run(chan);
                    });
                }
                datas.push((rxhome,txhome));
                idmaker.next();
            }
            Err(e) => {
                println!("LISTENER HANDLE ERROR");
            } 
        }
    
        
        // Handle Communication 
    }
}


use crate::basic_bbb::node::comm_channels::ChanManagerCArray;
pub fn handle_listener_array(rx:Receiver<TcpStream>){
    let mut array_chan = ChanManagerCArray::<IPCMessage>::new();
    // create joinHandle here 
    loop{
        match rx.try_recv(){
            Ok(stream)=>{
                if let Ok((mut node_,channel)) = node::Node::from_chan_manager_array(&mut array_chan,stream){    
                    std::thread::spawn(move||{
                        node_.run(channel);
                    });
                }
            }
            Err(e)=>{
                //println!("Array channel Error {:?}",e);
            }
        }
        array_chan.process_queue();
    }
}













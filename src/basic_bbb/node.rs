use crate::basic_bbb::conn;
use crate::basic_bbb::message;


pub mod comm_channels{    
    use std::sync::{Arc,mpsc::{Sender,Receiver}};
    use std::collections::HashMap;
    use crossbeam::queue::ArrayQueue;
    pub type ChanSnd = std::sync::mpsc::Sender<crate::basic_bbb::message::IPCMessage>;
    pub type ChanRecv = std::sync::mpsc::Receiver<crate::basic_bbb::message::IPCMessage>;
    pub type ServiceList = std::collections::HashMap<String,usize>;
    
    pub trait BBBChan<T>{
        fn send(&self,data:T)->Result<(),()>;
        fn recv(&self)->Result<T,()>;
    }

    // sync::mpsc channel 
    pub struct MpscChan<T>{
        rx:Receiver<T>,
        tx:Sender<T>
    }
    impl <T>BBBChan<T> for MpscChan<T>{
        fn send(&self,data:T)->Result<(),()>{
            if let Ok(_) = self.tx.send(data){
                Ok(())
            }else{
                Err(())
            }
        }
        
        // blocks 
        fn recv(&self)->Result<T,()>{
            if let Ok(v) = self.rx.recv(){
                return Ok(v)
            }else{
                return Err(())
            }
        }
    }

    impl <T>MpscChan<T>{
        pub fn new(rx:Receiver<T>,tx:Sender<T>)->Self{
            Self{
                rx,
                tx
            }
        }
    }
    




    // crossbeam::ArrayQueue as send 
    pub struct CArrayChan<T>{
        rx:Receiver<T>,
        tx:Arc<ArrayQueue<T>>
    }

    impl <T> BBBChan <T> for CArrayChan<T>{
        fn send(&self,data:T)->Result<(),()>{
            if let Ok(_) = self.tx.as_ref().push(data){
                Ok(())
            }else{
                Err(())
            }
        }
        
        fn recv(&self)->Result<T,()>{
            if let Ok(v) = self.rx.recv(){
                return Ok(v)
            }else{
                return Err(())
            }
        }

    } 
    impl <T>CArrayChan<T>{
        pub fn new(rx:Receiver<T>,tx:Arc<ArrayQueue<T>>)->Self{
            Self{
                rx,
                tx
            }
        }
    }


    
    // traits for all things sent over channels 
    pub trait ChanMsg{
        fn recv_id(&self)->Option<usize>;   // TODO : change to TaskID 
        fn send_id(&self)->Option<usize>;
        fn msg_type(&self) -> Option<u8>;
    }
    








    use std::sync::mpsc::channel;
    
    // NOTE : Limit the number of nodes to 100
    // NOTE : Create the message passing systems 
    // Channel managers;
    pub struct ChanManagerMpsc<T:ChanMsg>{
        channels:HashMap<usize,MpscChan<T>>,
        channel_names:HashMap<String,usize>,
        index:usize,
    }
    impl <T>ChanManagerMpsc<T> where T:ChanMsg{
        pub fn new()->Self{
            Self{
                channels:HashMap::new(),
                channel_names:HashMap::new(),
                index:0
            }
        }

        pub fn spawn_new_channel_set(&mut self)->(MpscChan<T>,MpscChan<T>){
            let ((txaway,rxhome),(txhome,rxaway)) = (channel(),channel());
            let away = MpscChan::new(rxaway,txaway);
            let home = MpscChan::new(rxhome,txhome);
            (home,away)
        }

        
        pub fn service_id(&self,name:&String)->Result<usize,()>{
            if let Some(&id) = self.channel_names.get(name){
                return Ok(id)
            }
            Err(())
        }

        // TODO : Err(()) could have &'str instead of a ()
        pub fn add_service(&mut self, name:String)->Result<MpscChan<T>,()>{ 
            if let Err(()) = self.service_id(&name){
                let (home,away) = self.spawn_new_channel_set();
                let index = self.index;
                self.index+=1;
                self.channels.insert(index,home);
                self.channel_names.insert(name,index);
                return Ok(away)
            }else{
                return Err(())
            }
        }

        // TODO : Err(()) could have &'str instead of a ()
        pub fn add_caller(&mut self, service_name:&String)->Result<(MpscChan<T>,usize),&'static str>{
            if let Ok(id) = self.service_id(service_name){
                let (home,away) = self.spawn_new_channel_set();
                let index = self.index;
                self.index+=1;
                self.channels.insert(index,home);
                return Ok((away,id))
            }else{
                return Err("A service with the following name does not exist!")
            }
        } 
        
        pub fn remove_client(&mut self, index:usize)->Result<(),()>{
            if let Some(_) = self.channels.remove(&index){
                return Ok(())
            }else{
                Err(())
            }
        }

        pub fn remove_service(&mut self,index:usize,name:&String)->Result<(),()>{
            if let Some(_) = self.channels.remove(&index){
                self.channel_names.remove(name);
                Ok(())
            }else{
                Err(())
            }
        }

        pub fn send(&self,index:usize,data:T)->Result<(),()>{
            if let Some(c) = self.channels.get(&index){
                c.send(data).expect("Cannot send");
                Ok(())
            }else{
                Err(())
            }
        }

        pub fn recv(&self,index:usize)->Result<T,()>{
            if let Some(c) = self.channels.get(&index){
                if let Ok(data) = c.recv(){
                    Ok(data)
                }else{
                    Err(())
                }
            }else{
                Err(())
            }
        }
        
        // NOTE : Non trait implementation  
        pub fn recv_nonblocking(&self,index:usize)->Result<T,()>{
            if let Some(c) = self.channels.get(&index){
                if let Ok(v) = c.rx.try_recv(){
                    Ok(v)
                }else{
                    Err(())
                }
            }else{
                Err(())
            }
        }
        
        pub fn process_messages(&mut self){
            // loop over all receivers , with try_recv 
            // and send if needed 
        }
    }


    // ArrayQueue Manager 
    pub struct ChanManagerCArray<T:ChanMsg>{
        array:Arc<ArrayQueue<T>>,
        receivers:HashMap<usize,Sender<T>>,
        channel_names:HashMap<String,usize>,
        index:usize
    }

    impl <T>ChanManagerCArray<T> where T:ChanMsg{
        pub fn new()->Self{
            Self{
                array:Arc::new(ArrayQueue::new(100)),   //100 is the maximum number of nodes possible for now 
                receivers:HashMap::new(),
                channel_names:HashMap::new(),
                index:0
            }
        }
        
        
        fn spawn_new_channel_set(&mut self)->(Sender<T>,CArrayChan<T>){
            let (tx,rx) = channel();
            let arr = CArrayChan::new(rx,Arc::clone(&self.array));
            (tx,arr)
        }
        


        pub fn service_id(&self,name_list:&Vec<String>)->Result<Vec<usize>,()>{
            let mut id_list = Vec::new();
            for name in name_list.iter(){    
                if let Some(&id) = self.channel_names.get(name){
                    id_list.push(id);
                }else{
                    return Err(())
                }
            }
            Ok(id_list)
        }


        pub fn does_service_exist(&self,name:&String)->bool{
            self.channel_names.contains_key(name)
        }

        
        // Channel, local ID 
        // NOTE : Return local ID
        pub fn add_service(&mut self, name:String)->Result<(CArrayChan<T>,usize),&'static str>{ 
            if let Err(()) = self.service_id(&vec![name.clone()]){  // Ugly hack but worth it 
                let (tx,array_ref) = self.spawn_new_channel_set();
                let index = self.index;
                self.index+=1;
                self.receivers.insert(index,tx);
                self.channel_names.insert(name,index);
                return Ok((array_ref,index))
            }else{
                return Err("Service Already Exists")
            }
        }
        
        // Channel , local ID , service ID (s in the future)
        // NOTE : Return ID of thing called 
        pub fn add_caller(&mut self, service_name:&Vec<String>)->Result<(CArrayChan<T>,usize,Vec<usize>),&'static str>{
            if let Ok(id_list) = self.service_id(service_name){
                let (tx,array_ref) = self.spawn_new_channel_set();
                let index = self.index;
                self.index+=1;
                self.receivers.insert(index,tx);
                return Ok((array_ref,index,id_list))
            }else{
                return Err("Service for Caller Does Not Exist")
            }
        } 
        
        pub fn remove_caller(&mut self, index:usize)->Result<(),()>{
            if let Some(_) = self.receivers.remove(&index){
                return Ok(())
            }else{
                Err(())
            }
        }
        
        // TODO : 
        // NOTE: Use some other data structure for storing Name:Index Pairs 
        // This part of the code already is the bottleneck
        // It will , in the worst case take O(N) Time to remove a service from this list
        // and that time will be consumed from time allocated to transfer message between strings
        pub fn remove_service(&mut self,index:usize)->Result<(),()>{
            if let Some(_) = self.receivers.remove(&index){
                let mut name = String::new();
                for (key,value) in self.channel_names.iter(){
                    if index == *value{
                        let name = key.clone();
                    }
                }
                self.channel_names.remove(&name);
                Ok(())
            }else{
                Err(())
            }
        }

        pub fn process_queue(&mut self){
            while let Some(msg) = self.array.as_ref().pop(){
                let id = msg.recv_id().expect("Receive ID not provided");
                let id_snd = msg.send_id().unwrap();
                let msg_type = msg.msg_type().unwrap();
                
                match msg_type{
                    bbb_parser::protocol_defs::methods::STOP_CALLER =>{
                        self.remove_caller(id_snd).expect("A method with this ID does not exist");
                        println!("Stopped caller {}, {:X}, {:X}",id_snd,msg_type, bbb_parser::protocol_defs::methods::STOP_CALLER );
                    }

                    bbb_parser::protocol_defs::methods::STOP_SERVICE =>{
                        self.remove_service(id_snd).expect("A service with this ID does not exist");
                        println!("Stopped Service {}",id_snd);
                    }
                    _ =>{        
                        println!("Sender : {} , Receiver :{}",id_snd,id);
                        self.receivers.get(&id).unwrap().send(msg).expect("Cannot send to channel");
                    }
                }


            }
        }

        pub fn index(&self)->usize{
            self.index
        }

    }

}

// TODO : change the tx and rx to implp BBBChan  
use comm_channels::{ServiceList,BBBChan,ChanMsg};
pub type TaskID =  usize;


// NOTE :  this is a stupid thing to do; but aesthetics!
// NOTE : change later
pub struct IDMaker{
    current:TaskID
}
impl IDMaker{
    pub fn new()->IDMaker{
        IDMaker{
            current:0
        }
    }
    pub fn next(&mut self)->IDMaker{
        IDMaker{
            current:self.current+1
        }
    }
    pub fn id(&self)->TaskID{
        self.current
    }
}
// IMMEDIATE TODO : Handle shutdown of service gracefully 
// IMMEDIATE TODO : Handle shutdowns gracefully
// IMMEDIATE TODO : handle stop service and stop caller     -- Might need debugging  
// TODO : multiservice callers      -- almost there  
// TODO : multiparameter services   -- not needed 
// TODO : create trait for sending stuff over BBBchans 
// TODO : provision to get node to be busy
// TODO : A service must maintain a counter of the number of things connected to it 
// TODO : We require something to manage the IDs of things 
//          A better ID for a service would be a Hash rather than an integer
pub enum NodeState{
    Busy,
    Free
}

pub enum NodeKind{
    Service(String),
    Caller{
        service_ids:Option<Vec<TaskID>>,
        service_names:Vec<String>             // Temporary, till I make the variable text not private 
    },
    // Broadcast,
    // BroadCastRecv
}



pub struct Node{
    state:NodeState,
    kind:NodeKind,
    id:Option<TaskID>,
    stream:Option<std::net::TcpStream>
}

impl Default for Node{
    fn default()->Self{
        Node{
            state:NodeState::Free,
            kind:NodeKind::Service(String::from("NULL")),
            id:None,
            stream:None
        }
    }
}

impl Node{
    pub fn new()->Self{
       Self::default() 
    }

    pub fn with_id(mut self, id:TaskID) -> Self{
        self.id = Some(id );
        self
    }

    pub fn with_stream(mut self,stream:std::net::TcpStream)->Self{
        self.stream = Some(stream);
        self
    }
    
    fn get_kind_from_client(&mut self, stream:&mut std::net::TcpStream)->Result<NodeKind,&'static str>{
        let mut conn_ = conn::ConnHandle::default();
        if let conn::ConnResult::Ok(res) = conn_.handle_connection(stream){
            match res.header(){
                bbb_parser::protocol_defs::methods::REG_CALLER  =>{  
                    return Ok(NodeKind::Caller{
                        service_ids:None,
                        service_names:res.text_list().unwrap().iter().map(|x|{x.clone()}).collect::<Vec<String>>()   //NOTE : res.text.unwrap()
                    }); 
                }

                bbb_parser::protocol_defs::methods::REG_SERVICE => {
                    return Ok(NodeKind::Service(res.text().unwrap())); // NOTE: res.text.unwrap() 
                }

                _ =>{
                    return Err("An initialisation method was not called");
                }
            }
        }
        Err("Parsing Error on connection")
    }



    #[deprecated]
    pub fn register(mut self, service_names:&ServiceList)->Result<Self,&'static str>{ 
        let mut conn_ = conn::ConnHandle::default();
        let mut _temp_conn = self.stream.take().unwrap();
        if let conn::ConnResult::Ok(res) = conn_.handle_connection(&mut _temp_conn){
            match res.header(){
                bbb_parser::protocol_defs::methods::REG_CALLER  =>{ 
                    let service_name = String::from("NULL");
                    self.kind = NodeKind::Caller{
                        service_ids:None,
                        service_names:vec![service_name.clone()]   //NOTE : res.text.unwrap()
                    };

                    if service_names.contains_key(&service_name) == false{
                        self.handle_failure();      // TODO : create this 
                        return Err("No service of this name exists");
                    }
                    
                }

                bbb_parser::protocol_defs::methods::REG_SERVICE => {
                    let service_name = String::from("NULL");
                    self.kind = NodeKind::Service(service_name.clone()); // NOTE: res.text.unwrap() 
                    if service_names.contains_key(&service_name){
                        self.handle_failure();      // TODO : create this 
                        return Err("A service of this name already exists");
                    }
                }

                _ =>{
                    return Err("An initialisation method was not called");
                }
            }
        }
        self.stream = Some(_temp_conn);
        Ok(self)
    }
        
    // PRIORITY : Change Manager.add_caller to account for more than one service name  
    // TODO :  Create a manager trait 
    pub fn from_chan_manager_array<T>(manager:&mut comm_channels::ChanManagerCArray<T>, mut stream:std::net::TcpStream)->Result<(Self,impl BBBChan<T>),&'static str> where T:comm_channels::ChanMsg{
        let mut node_ = Self::new();
        if let Ok(kind) = node_.get_kind_from_client(&mut stream){
            node_.kind = kind;
            //node_.stream = Some(stream);
            match & mut node_.kind{
                NodeKind::Caller{service_names,service_ids}=>{
                    let (res,self_id,srv_id) = manager.add_caller(&service_names).unwrap(); //FIXME : Handle if name does not exist 
                    *service_ids = Some(srv_id);
                    conn::ConnHandle::send(
                        get_reg_caller_ack_protocol(self_id as u8).to_vec(),        // FIXME : The ID sent here is objectively WRONG 
                        &mut stream         
                       ).unwrap();

                    node_.id = Some(self_id);
                    node_.stream = Some(stream);
                    println!("Connected to caller");
                    return Ok((node_,res))
                }

                NodeKind::Service(name) =>{
                    let (res,self_id) =  manager.add_service(name.clone()).unwrap();      // FIXME : Handle if name already exists  
                    conn::ConnHandle::send(
                        get_reg_service_ack_protocol(self_id as u8).to_vec(),
                        &mut stream         
                       ).unwrap();
                    node_.id = Some(self_id);
                    node_.stream = Some(stream);
                    println!("Connected to service");
                    return Ok((node_,res))
                }
            }
        }else{
            Err("Could not get kind ") // FIXME use a match{} here 
        }
    }
    

    pub fn from_chan_manager_mpsc<T>(manager:&comm_channels::ChanManagerMpsc<T>)->Self where T:ChanMsg{unimplemented!()}




    // NOTE :  Returns negative ACK message on failure to register
    // TODO (jfarhan) : On failure to REGISTER 
    pub fn handle_failure(&self){}
        
    
    // INFO : Chooses whether to run as a client or server 
    pub fn run(&mut self, chan:impl BBBChan<message::IPCMessage>){
        println!("Finding type");
        match &self.kind{
            NodeKind::Caller{..} => {
                println!("Running as caller");
                self.run_as_caller(chan)
            }
            NodeKind::Service(_) => {
                println!("Running as service");
                self.run_as_service(chan)
            }
        }
        println!("Unmatched");
    }
    
    
    pub fn run_as_caller(&mut self, chan:impl BBBChan<message::IPCMessage>){
        let mut stream = self.stream.take().unwrap();   //NOTE : handle this 
        loop{
            let mut conn_ = conn::ConnHandle::default();
            println!("Waiting for data from caller");
            if let conn::ConnResult::Ok(frame_meta) = conn_.handle_connection(&mut stream){  
                println!("\n From caller \n {:#?}",frame_meta);         // XXX
                let header = frame_meta.header();
                // PRIORITY : Extract information from data 
                // Result type represents the id of the receiver
                let current_recv_id = frame_meta.result_as_usize();  
                let msg = message::IPCMessage::new(frame_meta,self.id,current_recv_id,conn_.consume());
                chan.send(msg).expect("Caller : Receiving channel does not work");
                
                if header == STOP_CALLER{  // NOTE : STOPGAP SOLUTION 
                    break;
                }
            }else{
                println!("Frame Parsing Error for client");
            }
            let reply = chan.recv().unwrap();
            conn::ConnHandle::send(reply.consume(),&mut stream).expect("Send Failed for Caller ");      // send reply here 
        }
        println!("Graceful Shutdown!");
    }

    pub fn run_as_service(&mut self, chan:impl BBBChan<message::IPCMessage>){
        let mut stream  = self.stream.take().unwrap();
        loop{
            let mut conn_ = conn::ConnHandle::default();
            println!("Waiting for receiving data");
            let data = chan.recv().unwrap();
            let recv_id = data.snd_id();
            println!("Data received");
            conn::ConnHandle::send(data.consume(),&mut stream).expect("Send Failed for Service");      // send request here to service process 
            if let conn::ConnResult::Ok(frame_meta) = conn_.handle_connection(&mut stream){
                let msg = message::IPCMessage::new(frame_meta, self.id,recv_id,conn_.consume());
                chan.send(msg).expect("Service : Receiving channel does not work");
            }
        }
    }
    
    
    pub fn is_busy(&self)->Result<(),()>{
        todo!()
    }
    

    pub fn name(&self)->Option<String>{
        if let NodeKind::Service(name) = &self.kind{
            return Some(name.clone());
        }else{
            return None;
        }
    }
    

    // XXX
    pub fn new_caller()->Self{    
        unimplemented!()
    }
    // XXX
    pub fn new_service()->Self{
        unimplemented!()
    }


}


// TODO : MOVE ELSEWHERE 
use bbb_parser::protocol_defs::{self,methods::*};
pub fn get_reg_caller_ack_protocol(id:u8)->[u8;10]{
   return [protocol_defs::START,REG_CALLER_ACK,OK,OK_CODE,id,0,0x69,0x69,protocol_defs::CR,protocol_defs::CR]; 
}


pub fn get_reg_service_ack_protocol(id:u8)->[u8;8]{
   return [protocol_defs::START,REG_SERVICE_ACK,OK,OK_CODE,id,0,protocol_defs::CR,protocol_defs::CR]; 
}



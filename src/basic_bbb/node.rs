use crate::basic_bbb::conn;
use crate::basic_bbb::message;


pub type ChanSnd = std::sync::mpsc::Sender<crate::basic_bbb::message::IPCMessage>;
pub type ChanRecv = std::sync::mpsc::Receiver<crate::basic_bbb::message::IPCMessage>;
pub type ServiceList = std::collections::HashMap<String,usize>;
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
        service_id:Option<TaskID>,
        service_name:String             // Temporary, till I make the variable text not private 
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

    pub fn register(mut self, service_names:&ServiceList)->Result<Self,&'static str>{ 
        let mut conn_ = conn::ConnHandle::default();
        let mut _temp_conn = self.stream.take().unwrap();
        if let conn::ConnResult::Ok(res) = conn_.handle_connection(&mut _temp_conn){
            match res.header(){

                bbb_parser::protocol_defs::methods::REG_CALLER  =>{ 
                    let service_name = String::from("NULL");
                    self.kind = NodeKind::Caller{
                        service_id:None,
                        service_name:service_name.clone()   //NOTE : res.text.unwrap()
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
    
    // NOTE :  Returns negative ACK message on failure to register
    // TODO (jfarhan) : On failure to REGISTER 
    pub fn handle_failure(&self){}
        
    
    // INFO : Chooses whether to run as a client or server 
    pub fn run(&mut self, rx:ChanRecv,tx:ChanSnd){
        match &self.kind{
            NodeKind::Caller{..} => {
                self.run_as_caller(rx,tx)
            }
            NodeKind::Service(_) => {
                self.run_as_service(rx,tx)
            }
        }
    }
    

    pub fn run_as_caller(&mut self, rx:ChanRecv,tx: ChanSnd){
        let mut stream = self.stream.take().unwrap();   //NOTE : handle this 
        let recv_id = if let NodeKind::Caller{service_id,..} = self.kind{service_id}else{panic!("Ran a non-caller type as a caller")};
        loop{
            let mut conn_ = conn::ConnHandle::default();
            if let conn::ConnResult::Ok(frame_meta) = conn_.handle_connection(&mut stream){  
                let msg = message::IPCMessage::new(frame_meta,self.id,recv_id,conn_.consume());
                tx.send(msg).expect("Caller : Receiving channel does not work");
            }
            let reply = rx.recv().unwrap();
            conn::ConnHandle::send(reply.consume(),&mut stream).expect("Send Failed for Caller ");      // send reply here 
        }
    }

    pub fn run_as_service(&mut self, rx:ChanRecv, tx:ChanSnd){
        let mut stream  = self.stream.take().unwrap();
        loop{
            let mut conn_ = conn::ConnHandle::default();
            let data = rx.recv().unwrap();
            let recv_id = data.recv_id();      // create this field;
            conn::ConnHandle::send(data.consume(),&mut stream).expect("Send Failed for Service");      // send request here to service process 
            if let conn::ConnResult::Ok(frame_meta) = conn_.handle_connection(&mut stream){
                let msg = message::IPCMessage::new(frame_meta, self.id,recv_id,conn_.consume());
                tx.send(msg).expect("Service : Receiving channel does not work");
            }
        }
    }
    
    
    pub fn is_busy(&self)->Result<(),()>{
        todo!()
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

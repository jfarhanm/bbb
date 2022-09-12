use bbb_parser::ParsedFrame;
use crate::basic_bbb::node::TaskID;
use bbb_parser::protocol_defs::methods::*;
pub type Container = Vec<u8>;

pub enum ControlTypes{
    Call,
    CallReturn,
    RegService,
    StopService,
    RegCaller,
    StopCaller,
    UnMapped
}

impl ControlTypes{
    pub fn from_header(header:u8)->Self{
        match header{
            CALL =>{Self::Call},
            CALLRESP=>{Self::CallReturn}
            REG_SERVICE=>{Self::RegService}
            REG_CALLER=>{Self::RegCaller}
            STOP_SERVICE=>{Self::StopService}
            STOP_CALLER =>{Self::StopCaller}
            _ =>{Self::UnMapped}
        }
    }
}



pub struct IPCMessage{
    snd:Option<TaskID>,         //sender
    recv: Option<TaskID>,       // receiver
    control:ControlTypes,
    meta_data:ParsedFrame,
    data: Container
}

impl IPCMessage{
    pub fn new(meta_data:ParsedFrame,snd:Option<TaskID>,recv:Option<TaskID>,data:Container)->Self{
        Self{
            snd,
            recv,
            control:ControlTypes::from_header(meta_data.header()),
            meta_data,
            data
        }
    }


    pub fn consume(self)->Container{
        self.data
    }

    pub fn rcv_id(&self)->Option<TaskID>{
        self.recv
    }

    pub fn snd_id(&self)->Option<TaskID>{
        self.snd
    }
}


use crate::basic_bbb::node::comm_channels::ChanMsg;
impl ChanMsg for IPCMessage{
    fn send_id(&self)->Option<usize>{
        self.snd_id()
    }

    fn recv_id(&self)->Option<usize>{
        self.rcv_id()
    }

    fn msg_type(&self)->Option<u8>{
        Some(self.meta_data.header())
    }
}




// Do go gentle into that tech rabbit hole 
// FOMO shall  burn and rave at the close of day
// rage , rage against the dying of your projects 

#[cfg(test)]
mod tests{
}

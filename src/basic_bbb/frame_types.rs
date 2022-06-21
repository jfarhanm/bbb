use bbb_parser::ParsedFrame;
use crate::basic_bbb::node::TaskID;

pub enum ControlTypes{
    Call,
    CallReturn,
    RegService,
    StopService,
    RegCaller,
    StopCaller
}

pub struct IPCMessage{
    snd:Option<TaskID>,         //sender
    recv: Option<TaskID>,       // receiver
    control:ControlTypes,
    data:ParsedFrame 
}


// Do go gentle into that tech rabbit hole 
// Old age shall burn and rave at the close of day
// rage , rage against the dying of your projects 

#[cfg(test)]
mod tests{
}

// Initial Debug Commands
pub const REQ:u8 = 0x51;    //Q
pub const RESP:u8 = 0x41;   //A
pub const ACK:u8 = 0x69;
pub const ERR:u8 = 0x70;
pub const CR:u8 = 0x10;

pub const START:u8 = 0xBB;
pub const END:u8 = 0x10;
pub mod methods{    
    // These pertain to common responses
    pub const ACK:u8 = 0x69;
    pub const ERR:u8 = 0x70;
    pub const OK:u8 = 0x71;
    pub const OK_CODE:u8 = 0x72;
    // These pertain to services
    pub const CALLRESP:u8 = 0x20;   // respond to a remote call.
    pub const REG_SERVICE:u8 = 0x30;
    pub const REG_SERVICE_ACK:u8 = 0x32;
    pub const STOP_SERVICE:u8 = 0x33;
    pub const STOP_SERVICE_ACK:u8 = 0x34;

    // These pertain to service callers
    pub const CALL:u8 = 0x10;
    pub const REG_CALLER:u8 = 0x40;
    pub const REG_CALLER_ACK:u8 = 0x41;
    pub const STOP_CALLER:u8 = 0x42;
    pub const STOP_CALLER_ACK:u8 = 0x43;

    // For the future when streaming will be added.
    // Stream Broadcaster
    pub const BROADCAST:u8 = 0x50;
    pub const BROADCAST_REG:u8 = 0x60;
    pub const BROADCAST_REG_ACK:u8 = 0x61;

    // Stream Receiver
    pub const BROADCAST_RECV_ACK:u8 = 0x70;
    pub const BROADCAST_RECV_RDY:u8 = 0x80;
    pub const BROADCAST_RECV_REG:u8 = 0x90;
    pub const BROADCAST_RECV_REG_ACK:u8 = 0x91;

}


pub mod errors{
    pub const FRAME_PARSE_ERROR:u8 =        0xA0;
    pub const INCOMPLETE_FRAME_ERROR:u8 =   0xA1;
    pub const SERVICE_DOWN_ERROR:u8 =       0xA2;
    pub const NAME_INVALID_ERROR:u8 =       0xA3;
    pub const ALREADY_EXISTS_ERROR:u8 =     0xA4;
    pub const DOES_NOT_EXIST_ERROR:u8 =     0xA5;

}






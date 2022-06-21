use crate::basic_bbb::protocol_defs::{self,methods::*,errors::*,CR};
use std::fmt;

pub struct ParsedFrame{
    header:u8,
    data_size:Option<usize>,
    result_type:Option<(u8,u8)>,
    data:Option<(usize,usize)>,     //(start,end)
    text:Option<String>
}
impl ParsedFrame{
    pub fn with_text(header:u8,text:String)->Self{
        let mut frame = Self::default();
        frame.text = Some(text);
        frame.header = header;
        frame
    }

    pub fn with_data(header:u8,data_start:usize,data_end:usize)->Self{
        let mut frame= Self::default();
        frame.header = header;
        frame.data_size = Some(data_end - data_start);
        frame.data = Some((data_start,data_end));
        frame
    }

    // NOTE : deprecated use with_all_raw
    pub fn with_all(header:u8,data_start:usize,data_end:usize, result_type:(u8,u8),text:Option<String>)->Self{
        let mut frame= Self::default();
        frame.header = header;
        frame.data_size = Some(data_end - data_start);
        frame.data = Some((data_start,data_end));
        frame.text = text;
        frame.result_type = Some(result_type);
        frame
    }
    
    pub fn with_all_raw(header:u8,data_bounds:Option<(usize,usize)>, result_type:Option<(u8,u8)>,text:Option<String>)->Self{ 
        let mut frame= Self::default();
        frame.header = header;
        frame.data_size = if let Some(v) = data_bounds{Some(v.1-v.0)}else{None};
        frame.data = data_bounds;
        frame.text = text;
        frame.result_type = result_type;
        frame

    }
    

    pub fn with_header(header:u8)->Self{
        let mut frame= Self::default();
        frame.header = header;
        frame
    }

}

impl Default for ParsedFrame{
    fn default()->Self{
        ParsedFrame{
            header:0x00,
            data_size:None,
            result_type:None,
            data:None,
            text:None
        } 
    } 
}

impl fmt::Debug for ParsedFrame{
    fn fmt(&self,f:&mut fmt::Formatter<'_> )->fmt::Result{
        f.debug_struct("ParsedFrame")
            .field("Header",&self.header)
            .field("Data_size",&self.data_size)
            .field("Result_type",&self.result_type)
            .field("Data",&self.data)
            .field("Text",&self.text)
            .finish()
    }
}




// TODO (jfarhanm) : replace with_all with with_all_raw
// TODO (jfarhanm) : Test marked
pub enum ParseResult{
    Frame(ParsedFrame),
    IncompleteFrame(usize),
    ParseError(&'static str),  //TODO define error types
    DebugOk,
    Debug
}


pub struct BBBParse{
    data_bytes:Option<usize>,
    header_type:Option<u8>,
    parse_cursor:Option<usize>,
    error_bytes:Option<(u8,u8)>
}
impl BBBParse{
    pub fn new()->Self{
        Self{
            data_bytes:None,
            parse_cursor:None,
            header_type:None,
            error_bytes:None
        }
    }
    
    pub fn incr_parse_cursor(&mut self,len:usize)->Result<usize,&'static str>{
        if let Some(c) = &mut self.parse_cursor{
            *c+=len;
            return Ok(*c)
        }
        Err("The cursor has not been initiliased")
    }
    
    pub fn u8_to_char(code:u8)->char{
        if code.is_ascii_alphanumeric(){
            return code as char
        }else{
            return  std::char::from_digit(code as u32,10).unwrap();
        }
    }
    pub fn parse_textual(&mut self,data:&[u8])->Result<String,&'static str>{  
        if let Some(point) = data.iter().position(|x|{*x==protocol_defs::CR}){
            println!("Found Parse textual {}",point);
            let (m,_) = data.split_at(point);
            let name = m.iter().map(|d|{
                Self::u8_to_char(*d)
            }).collect::<String>();
            println!("name : {}",name);
            // NOTE: use the error here 
            self.incr_parse_cursor(point);
            return Ok(name)
        }
        Err("Text could not be parsed")
    }

    
    // TODO : data.len() is not what IncompleteFrame should return by any means
    // TODO : Make this more lisp-like. More Cons-ey 
    pub fn parse(&mut self, data:&[u8])->ParseResult{
        
        let mut data_iter = data.iter();
        if let None = self.header_type{
            if protocol_defs::START == *data_iter.next().expect("Empty Slice"){
                if let Some(v)= data_iter.next(){ 
                    self.header_type = Some(*v)
                }else{
                    return ParseResult::IncompleteFrame(1);
                }
                self.parse_cursor = Some(2);
            }else{
                return ParseResult::ParseError("Incorrect packet header");
            }
        }
    

        if let None = self.data_bytes{
            match self.header_type.unwrap(){
                // The only argument for a REG_CALLER or reg_service is a string
                REG_CALLER=>{
                    let (_,text_data) = data.split_at(self.parse_cursor.unwrap());
                    if let Ok(parsed) = self.parse_textual(text_data){
                        return ParseResult::Frame(ParsedFrame::with_text(REG_CALLER,parsed));
                    }else{
                        return ParseResult::IncompleteFrame(data.len());
                    }
                }

                REG_SERVICE =>{                 
                    let (_,text_data) = data.split_at(self.parse_cursor.unwrap());
                    if let Ok(parsed) = self.parse_textual(text_data){
                        return ParseResult::Frame(ParsedFrame::with_text(REG_SERVICE,parsed));
                    }else{
                        return ParseResult::IncompleteFrame(data.len());
                    }
                }

                // The only argument for STOP_{*} is the header         
                STOP_CALLER =>{
                    return ParseResult::Frame(ParsedFrame::with_header(STOP_CALLER))
                },


                STOP_SERVICE => {
                    return ParseResult::Frame(ParsedFrame::with_header(STOP_SERVICE))
                },
                
                
                // CALL and CALL_RESP have data packets
                CALL =>{
                    let (_,text_data) = data.split_at(self.parse_cursor.unwrap());
                    
                    // Internally updates cursor
                    if let Ok(parsed) = self.parse_textual(text_data){
                        self.data_bytes = Some(parsed.parse::<usize>().unwrap());
                    }else{
                        return ParseResult::IncompleteFrame(data.len());
                    }
                },

                CALLRESP =>{
                    // Unless data is not found
                    let (_,valid_data) = data.split_at(self.parse_cursor.unwrap());
                    if let Some(_) = valid_data.get(2){ 
                        self.error_bytes = Some((*valid_data.get(0).unwrap(),*valid_data.get(1).unwrap()) ); 
                        println!("ERROR BYTES : {:?}",self.error_bytes.unwrap());
                        let(_,text_data) = valid_data.split_at(2); 
                        if let Ok(parsed) = self.parse_textual(text_data){
                            self.data_bytes = Some(parsed.parse::<usize>().expect("Incorrect Number of Bytes"));
                            self.incr_parse_cursor(2); 
                        }else{
                            return ParseResult::IncompleteFrame(data.len());
                        }

                    }else{
                        return ParseResult::IncompleteFrame(data.len())
                    }
                },
                
                // TODO: test 
                REG_SERVICE_ACK =>{
                    let (_,valid_data) = data.split_at(self.parse_cursor.unwrap());
                    if let Some(d) = valid_data.get(4){
                        if *d==b'\n'{
                            let mut valid_data_iter = valid_data.iter().enumerate();
                            let result_type = (*valid_data_iter.next().unwrap().1,*valid_data_iter.next().unwrap().1);
                            let id_start = self.parse_cursor.unwrap() + valid_data_iter.next().unwrap().0;
                            let id_end = self.parse_cursor.unwrap() + valid_data_iter.next().unwrap().0;
                            return ParseResult::Frame(ParsedFrame::with_all(REG_SERVICE_ACK,id_start,id_end,result_type,None));
                        } 
                    }else{
                        return ParseResult::IncompleteFrame(data.len())
                    }
                },
                
                // TODO: test 
                REG_CALLER_ACK =>{
                    let (_,valid_data) = data.split_at(self.parse_cursor.unwrap());
                    if let Some(d) = valid_data.get(6){
                        if *d==b'\n'{
                            let mut valid_data_iter = valid_data.iter().enumerate();
                            let result_type = (*valid_data_iter.next().unwrap().1,*valid_data_iter.next().unwrap().1);
                            let id_start = self.parse_cursor.unwrap() + valid_data_iter.next().unwrap().0; 
                            valid_data_iter.next(); // FIXME DEBUG THIS   
                            valid_data_iter.next(); // FIXME DEBUG THIS // ID: 2 bytes 
                            let id_end = self.parse_cursor.unwrap() + valid_data_iter.next().unwrap().0;        // SERV_ID : 2 bytes
                            return ParseResult::Frame(ParsedFrame::with_all(REG_CALLER_ACK,id_start,id_end,result_type,None));
                        } 

                    }
                },
                
                // TODO: test 
                STOP_SERVICE_ACK =>{
                    let (_,valid_data) = data.split_at(self.parse_cursor.unwrap());
                    if let Some(d) = valid_data.get(2){
                        if *d==b'\n'{
                            let mut valid_data_iter = valid_data.iter().enumerate();
                            let result_type = (*valid_data_iter.next().unwrap().1,*valid_data_iter.next().unwrap().1);
                            return ParseResult::Frame(ParsedFrame::with_all_raw(STOP_SERVICE_ACK,None,Some(result_type),None));
                        } 
                    }else{
                        return ParseResult::IncompleteFrame(data.len())
                    }
                },

                // TODO : test 
                STOP_CALLER_ACK =>{
                    let (_,valid_data) = data.split_at(self.parse_cursor.unwrap());
                    if let Some(d) = valid_data.get(2){
                        if *d==b'\n'{
                            let mut valid_data_iter = valid_data.iter().enumerate();
                            let result_type = (*valid_data_iter.next().unwrap().1,*valid_data_iter.next().unwrap().1);
                            return ParseResult::Frame(ParsedFrame::with_all_raw(STOP_CALLER_ACK,None,Some(result_type),None));
                        } 
                    }else{
                        return ParseResult::IncompleteFrame(data.len())
                    }

                }
                _=> {return ParseResult::ParseError("Incorrect method called");}
            }
        }
        

        // NOTE: These are sent only with CALL and CALLRESP
        // These parser does not have to know or care about the data
        let (_,relevant) = data.split_at(self.parse_cursor.unwrap()+1); // NOTE: Could cause spaghetti code --1 
        // IMPORTANT (jfarhan): The caller must ensure that only the amount of bytes read must be
        // the bounds of the slice 
        if let Some(pos) = relevant.get(self.data_bytes.unwrap()){
            if *pos == protocol_defs::END{
                let start = self.parse_cursor.unwrap()+1;       // NOTE: Could cause spaghetti code --2
                let end = start+self.data_bytes.unwrap(); 
                if let Some(v) = self.error_bytes{
                    return ParseResult::Frame(ParsedFrame::with_all(self.header_type.unwrap(),start,end,v,None))
                }else{
                    //  Ok check 
                    return ParseResult::Frame(ParsedFrame::with_data(self.header_type.unwrap(),start,end))
                }

            }else{
                // NOTE (jfarhan): Return Error?
                return ParseResult::IncompleteFrame(data.len())
            }
        }else{
            return ParseResult::IncompleteFrame(data.len())
        }
    }
}


impl fmt::Debug for ParseResult{
    fn fmt(&self,f:&mut fmt::Formatter<'_> )->fmt::Result{
        match self{
            ParseResult::Debug =>{
                write!(f,"IGNORE FOR DEBUG")
            }
            ParseResult::DebugOk=>{
                write!(f,"IGNORE FOR DEBUG BUT OK")
            }
            ParseResult::Frame(data) =>{
                write!(f,"Frame({:#?})",data)
            }
            ParseResult::IncompleteFrame(size) =>{
                write!(f,"IncompleteFrame:{}",size)
            }
            ParseResult::ParseError(size) =>{
                write!(f,"ParseError : {}",size)
            }
        }
    }
}













#[cfg(test)]
pub mod test{    
    use crate::basic_bbb::protocol_defs::{self,methods::*,errors::*,CR};
    use crate::basic_bbb::parser::*;
    const BD:u8 = 0x69;
    const BUST:u8 = 0x00;
    #[test]
    pub fn test_parser(){
        //Service
        let register_service:[u8;7] = [REG_SERVICE,'T' as u8,'E' as u8,'S' as u8,'T' as u8,CR,CR];
        let register_service_resp:[u8;7] = [REG_SERVICE_ACK,OK,OK_CODE,0x00,0x03,CR,CR];
        
        let register_service_part_a = &register_service[0..3];
        let register_service_part_b = &register_service[3..5];
        let register_service_part_c = &register_service[5..7];


        let register_service_resp_err_ae:&[u8] = &[REG_SERVICE_ACK,ERR,ALREADY_EXISTS_ERROR,0x00,0x03,CR,CR];
        let register_service_resp_err_sd:&[u8] = &[REG_SERVICE_ACK,ERR,SERVICE_DOWN_ERROR,0x00,0x03,CR,CR];
        let register_service_resp_err_if:&[u8] = &[REG_SERVICE_ACK,ERR,SERVICE_DOWN_ERROR,0x00,0x03,CR,CR];
        let register_service_resp_err_ae:&[u8] = &[REG_SERVICE_ACK,ERR,NAME_INVALID_ERROR,0x00,0x03,CR,CR];


        let call_service:&[u8] = &[CALL,'5' as u8,CR,BD,BD,BD,BD,BD,CR,CR];
        let service_response:[u8;11] = [CALLRESP,OK,OK_CODE,'5' as u8,BD,BD,BD,BD,BD,CR,CR];
        let service_response_a = &service_response[0..3];
        let service_response_a = &service_response[3..6];
        let service_response_a = &service_response[6..9];
        let service_response_a = &service_response[9..11];

        
        let service_response:&[u8] = &[CALLRESP,ERR,INCOMPLETE_FRAME_ERROR,'5' as u8,BD,BD,BD,BD,BD,CR,CR];
        let service_response:&[u8] = &[CALLRESP,ERR,SERVICE_DOWN_ERROR,'5' as u8,BD,BD,BD,BD,BD,CR,CR];
        let service_response:&[u8] = &[CALLRESP,ERR,FRAME_PARSE_ERROR,'5' as u8,BD,BD,BD,BD,BD,CR,CR];



        let register_caller:&[u8] = &[REG_CALLER,'T' as u8,'E' as u8,'S' as u8,'T' as u8,CR,CR];
        let register_caller_resp:&[u8] = &[REG_CALLER_ACK,ERR,FRAME_PARSE_ERROR,0x0,0x02,0x0,0x03,CR,CR];
        let register_caller_resp:&[u8] = &[REG_CALLER_ACK,ERR,DOES_NOT_EXIST_ERROR,0x0,0x02,0x0,0x03,CR,CR];
        let register_caller_resp:&[u8] = &[REG_CALLER_ACK,ERR,INCOMPLETE_FRAME_ERROR,0x0,0x02,0x0,0x03,CR,CR];
        let register_caller_resp:&[u8] = &[REG_CALLER_ACK,ERR,NAME_INVALID_ERROR,0x0,0x02,0x0,0x03,CR,CR];
    }


    #[test]
    pub fn test_bbb_parse(){
        let mut underlying_data = Vec::<u8>::with_capacity(4096);
        let mut bbb_parse = crate::basic_bbb::parser::BBBParse::new();
        
        // Test with full data      --SUCCESS
        let register_service:[u8;8] = [protocol_defs::START,REG_SERVICE,'T' as u8,'E' as u8,'S' as u8,'T' as u8,CR,CR];
        let register_caller:[u8;10] = [protocol_defs::START,REG_CALLER,'T' as u8,'E' as u8,'S' as u8,'T' as u8,CR,CR,0,0];
        let callresp:[u8;14]  = [protocol_defs::START,CALLRESP,0x20,0x20,4,CR,0xDE,0xAD,0xBE,0xEF,CR,CR,0,0]; 
        let call:[u8;12]  = [protocol_defs::START,CALL,4,CR,0xDE,0xAD,0xBE,0xEF,CR,CR,0,0]; 
        let stop_service:[u8;4] =[protocol_defs::START,STOP_SERVICE,CR,CR];
        let stop_caller:[u8;4] =[protocol_defs::START,STOP_CALLER,CR,CR]; 
        for m in 1..12{
            let output  = bbb_parse.parse(&callresp[..m]);
            println!("{:#?}",output) ;   
        }
    }
}

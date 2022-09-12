use std::net::TcpStream;
use std::io::Write;
use std::io::Read;
use bbb_parser::BBBParse;
use crate::basic_bbb::message::Container;

pub enum ConnResult{
    Ok(bbb_parser::ParsedFrame),
    Err(&'static str)
}

pub struct ConnHandle{
    read_cursor:usize,
    size:usize,
    data:Container
}

impl ConnHandle{
    pub fn new()->Self{
        Self::default()
    }

    pub fn with_size(size:usize)->Self{
        Self{
            read_cursor:0,
            size,
            data:vec![0;size]
        }
    }

    pub fn consume(self)->Container{
        self.data
    }

    pub fn handle_connection(&mut self,stream:&mut TcpStream)->ConnResult{
        let mut parser = BBBParse::new();
        loop{

            if let Ok(n) = stream.read(&mut self.data[self.read_cursor..]){
                if n==0{
                    // cats and dogs making bread together 
                    // NOTE decide what to do here : return Error ? Or Parse?
                }
                
                self.read_cursor+=n;

                match parser.parse(&self.data[..self.read_cursor]){
                    bbb_parser::ParseResult::Frame(res) => {
                        return ConnResult::Ok(res);
                    },

                    bbb_parser::ParseResult::IncompleteFrame(res) =>{
                        if let Some(v) = res.size(){
                            let size = v + 0xFF;
                    // NOTE Added 0xFF  because it appears to be an upper bound for the
                    // packet parameters excluding data 
                            if size>self.size{
                                self.resize(size);
                            }
                        }
                    },

                    _=>{
                        return ConnResult::Err("Frame Parsing Error");
                        //  For the time being 
                    }
                }

            }else{
                // Organic life no longer exists on earth 
                // and Sentient metal bread rules the galaxy
                panic!("Client Error - Aborting!"); // NOTE (jfarhanm) : Probably not a good idea to panic here
            }
        

        }
        
    }

    pub fn clean(&mut self){
        self.data.clear()
    }
    

    pub fn resize(&mut self,len:usize){
        self.data.resize(len,0)
    }

    pub fn send(data:Container, stream:&mut TcpStream)->std::io::Result<()>{
       stream.write_all(data.as_slice())
    }

}


pub const DEFAULT_SIZE:usize = 64;
impl Default for ConnHandle{
    fn default()->Self{
        Self{
            read_cursor:0,
            size:DEFAULT_SIZE,
            data:vec![0u8;DEFAULT_SIZE]
        }
    }
}


#[cfg(test)]
mod test{
    use crate::basic_bbb::conn::ConnHandle;
    use std::net::TcpStream;
    use std::net::TcpListener;
    #[test]
    fn bbb_conn_tests(){
        let mut listener = TcpListener::bind("localhost:8008").unwrap();
        if let Ok((stream,_)) = listener.accept(){
            handle_connections(stream);
        }

    }

    pub fn handle_connections(mut connection:TcpStream){
        let mut handle = ConnHandle::with_size(64);
        if let crate::basic_bbb::conn::ConnResult::Ok(_) = handle.handle_connection(&mut connection){
            let out  = handle.consume();
        }
    }
}


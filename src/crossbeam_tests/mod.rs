use crossbeam::atomic;
use crossbeam::utils;
use crossbeam::queue;
use std::cell::RefCell;
use std::sync::Arc;


#[derive(Debug)]
pub struct ShareableData{
    name:String,
    car:(u8,u8)
}

type ACdt <'a> = atomic::AtomicCell<&'a queue::ArrayQueue<ShareableData>>;
type Arcdt = Arc<queue::ArrayQueue<ShareableData>>;


pub fn test(){
    let nq= queue::ArrayQueue::<ShareableData>::new(7);
    let new_arc = Arc::new(nq);
    for m in 0..10{
        let new_arc = Arc::clone(&new_arc);
        let n = m;
        std::thread::spawn(move||{
           thread_arc(new_arc,n) 
        });
    }
    
    loop{    
        while let Some(v) = new_arc.as_ref().pop(){
            println!("{:?}",v);
        }
    }

}


pub fn thread_arc(data:Arcdt,index:u8){
    let mut ind = 0;
    loop{
        //std::thread::sleep_ms(5);
        //This is idiotically error prone , but it works !
        if let Ok(v) = data.as_ref().push(ShareableData{name:String::from("Hello"),car:(ind,index)}){
            ind+=1;
        }
    }
}


pub fn thread_p(data: ACdt){
    unsafe{
        (data.as_ptr()).as_ref().unwrap().push(ShareableData{name:String::from("Hello"),car:(10,10)});
    }
}

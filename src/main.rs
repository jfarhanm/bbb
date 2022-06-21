use std::io::Result;
pub fn main()->Result<()>{

    let data:&[u8] = &[9,9];
    let newdata:&[u8] = vec![0u8;10].as_slice();
    Ok(())

}

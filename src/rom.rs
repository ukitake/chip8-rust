use std::fs::File;
use std::io::{prelude::*};
use std::io::{self};

pub(crate) struct Rom {
    pub bytes: Vec<u8>,
}

pub(crate) fn read(file_name: &str) -> io::Result<Rom> {
    let mut file = File::open(&file_name)?;
    
    let mut program_buffer = Vec::new();
    file.read_to_end(&mut program_buffer)?;
    
    return Ok(Rom { bytes: program_buffer });
}

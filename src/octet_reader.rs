pub mod octet_reader {
    use std::{fs::File, io::{BufReader, Read}};

    
pub struct OctetReader {
    reader: BufReader<File>
}

impl OctetReader {

    pub fn new(file: File) -> Self {
        return OctetReader{ reader: BufReader::new(file) }
    }  

    pub fn read_next(&mut self) -> Result<u8, std::io::Error> {
        
        let mut  word_bytes: [u8; 1] = [0];

        return match self.reader.read_exact(&mut word_bytes) {
            Ok(()) => Ok( word_bytes[0]),
            Err(err) => return  Err(err)
        }
    }
    
}

}
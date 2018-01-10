use std::io::{Write, Error as IOError};
use std::string::FromUtf8Error;

pub trait Output {
    fn write(&mut self, seg: &str) -> Result<(), IOError>;
}

pub struct WriteOutput {
    write: Box<Write>
}

impl Output for WriteOutput {
    fn write(&mut self, seg: &str) -> Result<(), IOError> {
        self.write.write(seg.as_bytes()).map(|_| ())
    }
}

impl WriteOutput {
    pub fn new<W>(w: W) -> WriteOutput where W: Write + 'static {
        WriteOutput {
            write: Box::new(w)
        }
    }
}

pub struct StringOutput {
    buf: Vec<u8>
}

impl Output for StringOutput {
    fn write(&mut self, seg: &str) -> Result<(), IOError> {
        for b in seg.as_bytes() {
            self.buf.push(*b);
        }
        
        Ok(())
    }
}

impl StringOutput {
    pub fn new() -> StringOutput {
        StringOutput {
            buf: Vec::with_capacity(8 * 1024)
        }
    }

    pub fn to_string(self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.buf)
    }
}

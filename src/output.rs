use std::io::{Error as IOError, Write};
use std::string::FromUtf8Error;

pub trait Output {
    fn write(&mut self, seg: &str) -> Result<(), IOError>;
}

pub struct WriteOutput<W: Write> {
    write: W,
}

impl<W: Write> Output for WriteOutput<W> {
    fn write(&mut self, seg: &str) -> Result<(), IOError> {
        self.write.write_all(seg.as_bytes())
    }
}

impl<W: Write> WriteOutput<W> {
    pub fn new(write: W) -> WriteOutput<W> {
        WriteOutput { write }
    }
}

pub struct StringOutput {
    buf: Vec<u8>,
}

impl Output for StringOutput {
    fn write(&mut self, seg: &str) -> Result<(), IOError> {
        self.buf.extend_from_slice(seg.as_bytes());
        Ok(())
    }
}

impl StringOutput {
    pub fn new() -> StringOutput {
        StringOutput {
            buf: Vec::with_capacity(8 * 1024),
        }
    }

    pub fn into_string(self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.buf)
    }
}

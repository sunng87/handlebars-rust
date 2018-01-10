use std::io::{Error as IOError, Write};
use std::string::FromUtf8Error;

pub trait Output {
    fn write(&mut self, seg: &str) -> Result<(), IOError>;
}

pub struct WriteOutput<'a, W: 'a + Write> {
    write: &'a mut W,
}

impl<'a, W: 'a + Write> Output for WriteOutput<'a, W> {
    fn write(&mut self, seg: &str) -> Result<(), IOError> {
        self.write.write_all(seg.as_bytes())
    }
}

impl<'a, W: 'a + Write> WriteOutput<'a, W> {
    pub fn new(write: &'a mut W) -> WriteOutput<'a, W>
    where
        W: Write + 'a,
    {
        WriteOutput { write }
    }
}

pub struct StringOutput {
    buf: Vec<u8>,
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
            buf: Vec::with_capacity(8 * 1024),
        }
    }

    pub fn to_string(self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.buf)
    }
}

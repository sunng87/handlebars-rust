pub mod str {
    use std::io::{Result, Write};

    #[derive(Debug)]
    pub struct StringWriter {
        buf: Vec<u8>,
    }

    impl StringWriter {
        pub fn new() -> StringWriter {
            StringWriter {
                buf: Vec::with_capacity(8 * 1024),
            }
        }

        pub fn to_string(self) -> String {
            if let Ok(s) = String::from_utf8(self.buf) {
                s
            } else {
                String::new()
            }
        }
    }

    impl Write for StringWriter {
        fn write(&mut self, buf: &[u8]) -> Result<usize> {
            for b in buf {
                self.buf.push(*b);
            }
            Ok(buf.len())
        }

        fn flush(&mut self) -> Result<()> {
            Ok(())
        }
    }

    #[cfg(test)]
    mod test {
        use std::io::Write;
        use support::str::StringWriter;

        #[test]
        fn test_string_writer() {
            let mut sw = StringWriter::new();

            let _ = sw.write("hello".to_owned().into_bytes().as_ref());
            let _ = sw.write("world".to_owned().into_bytes().as_ref());

            let s = sw.to_string();
            assert_eq!(s, "helloworld".to_string());
        }
    }
}

use std::cell::Ref;
use std::ops::Deref;

pub enum RefWrapper<'a, T: 'a + ?Sized> {
    CellRef(Ref<'a, T>),
    Ref(&'a T),
}

impl<'a, T: 'a + ?Sized> Deref for RefWrapper<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            RefWrapper::CellRef(cell_ref) => cell_ref.deref(),
            RefWrapper::Ref(normal_ref) => normal_ref.deref(),
        }
    }
}

impl<'a, T: 'a + ?Sized> From<Ref<'a, T>> for RefWrapper<'a, T> {
    fn from(cell_ref: Ref<'a, T>) -> RefWrapper<'a, T> {
        RefWrapper::CellRef(cell_ref)
    }
}

impl<'a, T: 'a + ?Sized> From<&'a T> for RefWrapper<'a, T> {
    fn from(normal_ref: &'a T) -> RefWrapper<'a, T> {
        RefWrapper::Ref(normal_ref)
    }
}

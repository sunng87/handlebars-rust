pub mod str {
    use std::io::{Write, Result};

    pub struct StringWriter {
        buf: Vec<u8>,
    }

    impl StringWriter {
        pub fn new() -> StringWriter {
            StringWriter { buf: Vec::with_capacity(8 * 1024) }
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
        use support::str::StringWriter;
        use std::io::Write;

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

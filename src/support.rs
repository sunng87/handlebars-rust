pub mod str {
    use std::io::{Result, Write};

    #[derive(Debug)]
    pub struct StringWriter {
        buf: Vec<u8>,
    }

    impl Default for StringWriter {
        fn default() -> Self {
            Self::new()
        }
    }

    impl StringWriter {
        pub fn new() -> StringWriter {
            StringWriter {
                buf: Vec::with_capacity(8 * 1024),
            }
        }

        pub fn into_string(self) -> String {
            if let Ok(s) = String::from_utf8(self.buf) {
                s
            } else {
                String::new()
            }
        }
    }

    impl Write for StringWriter {
        fn write(&mut self, buf: &[u8]) -> Result<usize> {
            self.buf.extend_from_slice(buf);
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

            let s = sw.into_string();
            assert_eq!(s, "helloworld".to_string());
        }
    }
}

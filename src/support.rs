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

    pub fn escape_html(s: &str) -> String {
        let mut output = String::new();
        for c in s.chars() {
            match c {
                '<' => output.push_str("&lt;"),
                '>' => output.push_str("&gt;"),
                '"' => output.push_str("&quot;"),
                '&' => output.push_str("&amp;"),
                _ => output.push(c),
            }
        }
        output
    }

    #[cfg(test)]
    mod test {
        use crate::support::str::StringWriter;
        use std::io::Write;

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

pub mod str {
    use std::io::{Write, Result};

    pub trait SliceChars {
        fn slice_chars_alt(&self, begin: usize, end: usize) -> &str;
    }

    impl SliceChars for str {
        fn slice_chars_alt(&self, begin: usize, end: usize) -> &str {
            assert!(begin <= end);
            let mut count = 0;
            let mut begin_byte = None;
            let mut end_byte = None;

            // This could be even more efficient by not decoding,
            // only finding the char boundaries
            for (idx, _) in self.char_indices() {
                if count == begin { begin_byte = Some(idx); }
                if count == end { end_byte = Some(idx); break; }
                count += 1;
            }
            if begin_byte.is_none() && count == begin { begin_byte = Some(self.len()) }
            if end_byte.is_none() && count == end { end_byte = Some(self.len()) }

            match (begin_byte, end_byte) {
                (None, _) => panic!("slice_chars: `begin` is beyond end of string"),
                (_, None) => panic!("slice_chars: `end` is beyond end of string"),
                (Some(a), Some(b)) => unsafe { self.slice_unchecked(a, b) }
            }
        }
    }

    pub struct StringWriter {
        buf: Vec<u8>
    }

    impl StringWriter {
        pub fn new() -> StringWriter {
            StringWriter {
                buf: Vec::new()
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

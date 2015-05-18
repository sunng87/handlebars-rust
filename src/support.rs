pub mod str {
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
}

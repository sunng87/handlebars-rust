quick_error! {
    /// Template parsing error
    #[derive(Debug, Clone)]
    pub enum TemplateError {
        UnclosedBraces(line_no: usize, col_no: usize) {
            display("closing braces `}}` expected but EOF reached at line {:?}, column {:?}",
                    line_no, col_no)
            description("closing braces `}}` expected but EOF reached")
        }
        UnexpectedClosingBraces(line_no: usize, col_no: usize) {
            display("can't close braces `}}` at line {:?}, column {:?}",
                    line_no, col_no)
            description("can't close braces `}}` at this location")
        }
        MismatchingClosedHelper(line_no: usize, col_no: usize, open: String, closed: String) {
            display("helper {:?} was opened, but {:?} is closing at line {:?}, column {:?}",
                open, closed, line_no, col_no)
            description("wrong name of closing helper")
        }
        UnclosedHelper(line_no: usize, col_no: usize, name: String) {
            display("helper {:?} was not closed on the end of file at line {:?}, column {:?}",
                    name, line_no, col_no)
            description("some helper was not closed on the end of file")
        }
    }
}

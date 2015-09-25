quick_error! {
    /// Template parsing error
    #[derive(Debug, Clone)]
    pub enum TemplateError {
        UnclosedBraces {
            description("closing braces `}}` expected but eof reached")
        }
        UnexpectedClosingBraces {
            description("can't close braces `}}` at this location")
        }
        MismatchingClosedHelper(open: String, closed: String) {
            display("helper {:?} was opened, but {:?} is closing",
                open, closed)
            description("wrong name of closing helper")
        }
        UnclosedHelper(name: String) {
            display("helper {:?} was not closed on end of file", name)
            description("some helper was not closed on end of file")
        }
    }
}

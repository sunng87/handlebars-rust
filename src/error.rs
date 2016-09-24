use std::io::Error as IOError;

use render::RenderError;

quick_error! {
/// Template parsing error
    #[derive(PartialEq, Debug, Clone)]
    pub enum TemplateError {
        MismatchingClosedHelper(line_no: usize, col_no: usize, open: String, closed: String) {
            display("helper {:?} was opened, but {:?} is closing at line {:?}, column {:?}",
                open, closed, line_no, col_no)
            description("wrong name of closing helper")
        }
        MismatchingClosedDirective(line_no: usize, col_no: usize, open: String, closed: String) {
            display("directive {:?} was opened, but {:?} is closing at line {:?}, column {:?}",
                open, closed, line_no, col_no)
            description("wrong name of closing directive")
        }
        InvalidSyntax (line_no: usize, col_no: usize) {
            display("invalid handlebars syntax at line {:?}, column {:?}", line_no, col_no)
            description("invalid handlebars syntax")
        }
        InvalidParam (param: String) {
            display("invalid parameter {:?}", param)
            description("invalid parameter")
        }
        NestedSubexpression(line_no: usize, col_no: usize) {
            display("nested subexpression at line {:?}, column {:?} is not supported.", line_no, col_no)
            description("nested subexpression is not supported")
        }
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum TemplateFileError {
        TemplateError(err: TemplateError) {
            from()
            cause(err)
        }
        IOError(err: IOError) {
            from()
            cause(err)
        }
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum TemplateRenderError {
        TemplateError(err: TemplateError) {
            from()
            cause(err)
        }
        RenderError(err: RenderError) {
            from()
            cause(err)
        }
        IOError(err: IOError) {
            from()
            cause(err)
        }
    }
}

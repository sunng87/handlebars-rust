use std::io::Error as IOError;
use std::error::Error;
use std::fmt;

use serde_json::error::Error as SerdeError;

use template::Parameter;

/// Error when rendering data on template.
#[derive(Debug)]
pub struct RenderError {
    pub desc: String,
    pub template_name: Option<String>,
    pub line_no: Option<usize>,
    pub column_no: Option<usize>,
    cause: Option<Box<Error + Send + Sync>>,
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match (self.line_no, self.column_no) {
            (Some(line), Some(col)) => write!(
                f,
                "Error rendering \"{}\" line {}, col {}: {}",
                self.template_name
                    .as_ref()
                    .unwrap_or(&"Unnamed template".to_owned(),),
                line,
                col,
                self.desc
            ),
            _ => write!(f, "{}", self.desc),
        }
    }
}

impl Error for RenderError {
    fn description(&self) -> &str {
        &self.desc[..]
    }

    fn cause(&self) -> Option<&Error> {
        self.cause.as_ref().map(|e| &**e as &Error)
    }
}

impl From<IOError> for RenderError {
    fn from(e: IOError) -> RenderError {
        RenderError::with(e)
    }
}

impl From<SerdeError> for RenderError {
    fn from(e: SerdeError) -> RenderError {
        RenderError::with(e)
    }
}

impl RenderError {
    pub fn new<T: AsRef<str>>(desc: T) -> RenderError {
        RenderError {
            desc: desc.as_ref().to_owned(),
            template_name: None,
            line_no: None,
            column_no: None,
            cause: None,
        }
    }

    pub fn with<E>(cause: E) -> RenderError
    where
        E: Error + Send + Sync + 'static,
    {
        let mut e = RenderError::new(cause.description());
        e.cause = Some(Box::new(cause));

        e
    }
}

quick_error! {
/// Template parsing error
    #[derive(PartialEq, Debug, Clone)]
    pub enum TemplateErrorReason {
        MismatchingClosedHelper(open: String, closed: String) {
            display("helper {:?} was opened, but {:?} is closing",
                open, closed)
            description("wrong name of closing helper")
        }
        MismatchingClosedDirective(open: Parameter, closed: Parameter) {
            display("directive {:?} was opened, but {:?} is closing",
                open, closed)
            description("wrong name of closing directive")
        }
        InvalidSyntax {
            display("invalid handlebars syntax.")
            description("invalid handlebars syntax")
        }
        InvalidParam (param: String) {
            display("invalid parameter {:?}", param)
            description("invalid parameter")
        }
        NestedSubexpression {
            display("nested subexpression is not supported")
            description("nested subexpression is not supported")
        }
    }
}

/// Error on parsing template.
#[derive(Debug, PartialEq)]
pub struct TemplateError {
    pub reason: TemplateErrorReason,
    pub template_name: Option<String>,
    pub line_no: Option<usize>,
    pub column_no: Option<usize>,
    segment: Option<String>,
}

impl TemplateError {
    pub fn of(e: TemplateErrorReason) -> TemplateError {
        TemplateError {
            reason: e,
            template_name: None,
            line_no: None,
            column_no: None,
            segment: None,
        }
    }

    pub fn at(mut self, template_str: &str, line_no: usize, column_no: usize) -> TemplateError {
        self.line_no = Some(line_no);
        self.column_no = Some(column_no);
        self.segment = Some(template_segment(template_str, line_no, column_no));
        self
    }

    pub fn in_template(mut self, name: String) -> TemplateError {
        self.template_name = Some(name);
        self
    }
}

impl Error for TemplateError {
    fn description(&self) -> &str {
        self.reason.description()
    }
}

fn template_segment(template_str: &str, line: usize, col: usize) -> String {
    let range = 3;
    let line_start = if line >= range { line - range } else { 0 };
    let line_end = line + range;

    let mut buf = String::new();
    for (line_count, line_content) in template_str.lines().enumerate() {
        if line_count >= line_start && line_count <= line_end {
            buf.push_str(&format!("{:4} | {}\n", line_count, line_content));
            if line_count == line - 1 {
                buf.push_str("     |");
                for c in 0..line_content.len() {
                    if c != col {
                        buf.push_str("-");
                    } else {
                        buf.push_str("^");
                    }
                }
                buf.push_str("\n");
            }
        }
    }

    buf
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match (self.line_no, self.column_no, &self.segment) {
            (Some(line), Some(col), &Some(ref seg)) => write!(
                f,
                "Template error: {}\n    --> Template error in \"{}\":{}:{}\n     |\n{}     |\n     = reason: {}\n",
                self.reason,
                self.template_name
                    .as_ref()
                    .unwrap_or(&"Unnamed template".to_owned()),
                line,
                col,
                seg,
                self.reason
            ),
            _ => write!(f, "{}", self.reason),
        }
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum TemplateFileError {
        TemplateError(err: TemplateError) {
            from()
            cause(err)
            description(err.description())
            display("{}", err)
        }
        IOError(err: IOError, name: String) {
            cause(err)
            description(err.description())
            display("Template \"{}\": {}", name, err)
        }
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum TemplateRenderError {
        TemplateError(err: TemplateError) {
            from()
            cause(err)
            description(err.description())
            display("{}", err)
        }
        RenderError(err: RenderError) {
            from()
            cause(err)
            description(err.description())
            display("{}", err)
        }
        IOError(err: IOError, name: String) {
            cause(err)
            description(err.description())
            display("Template \"{}\": {}", name, err)
        }
    }
}

impl TemplateRenderError {
    pub fn as_render_error(&self) -> Option<&RenderError> {
        if let &TemplateRenderError::RenderError(ref e) = self {
            Some(&e)
        } else {
            None
        }
    }
}

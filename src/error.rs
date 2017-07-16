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
    cause: Option<Box<Error + Send>>,
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match (self.line_no, self.column_no) {
            (Some(line), Some(col)) => {
                write!(
                    f,
                    "Error rendering \"{}\" line {}, col {}: {}",
                    self.template_name.as_ref().unwrap_or(
                        &"Unnamed template".to_owned(),
                    ),
                    line,
                    col,
                    self.desc
                )
            }
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
        E: Error + Send + 'static,
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
    pub column_no: Option<usize>, // template segment
}

impl TemplateError {
    pub fn of(e: TemplateErrorReason) -> TemplateError {
        TemplateError {
            reason: e,
            template_name: None,
            line_no: None,
            column_no: None,
        }
    }

    pub fn at(mut self, line_no: usize, column_no: usize) -> TemplateError {
        self.line_no = Some(line_no);
        self.column_no = Some(column_no);
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

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match (self.line_no, self.column_no) {
            (Some(line), Some(col)) => {
                write!(
                    f,
                    "Template \"{}\" line {}, col {}: {}",
                    self.template_name.as_ref().unwrap_or(
                        &"Unnamed template".to_owned(),
                    ),
                    line,
                    col,
                    self.reason
                )
            }
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

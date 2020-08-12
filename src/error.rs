use std::error::Error;
use std::fmt;
use std::io::Error as IOError;
use std::num::ParseIntError;
use std::string::FromUtf8Error;

use serde_json::error::Error as SerdeError;
#[cfg(feature = "dir_source")]
use walkdir::Error as WalkdirError;

#[cfg(feature = "script_helper")]
use rhai::{EvalAltResult, ParseError};

/// Error when rendering data on template.
#[derive(Debug)]
pub struct RenderError {
    pub desc: String,
    pub template_name: Option<String>,
    pub line_no: Option<usize>,
    pub column_no: Option<usize>,
    cause: Option<Box<dyn Error + Send + Sync + 'static>>,
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
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
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause.as_ref().map(|e| &**e as &(dyn Error + 'static))
    }
}

impl From<IOError> for RenderError {
    fn from(e: IOError) -> RenderError {
        RenderError::from_error("Error on output generation.", e)
    }
}

impl From<SerdeError> for RenderError {
    fn from(e: SerdeError) -> RenderError {
        RenderError::from_error("Error when accessing JSON data.", e)
    }
}

impl From<FromUtf8Error> for RenderError {
    fn from(e: FromUtf8Error) -> RenderError {
        RenderError::from_error("Error on bytes generation.", e)
    }
}

impl From<ParseIntError> for RenderError {
    fn from(e: ParseIntError) -> RenderError {
        RenderError::from_error("Error on accessing array/vector with string index.", e)
    }
}

#[cfg(feature = "script_helper")]
impl From<Box<EvalAltResult>> for RenderError {
    fn from(e: Box<EvalAltResult>) -> RenderError {
        RenderError::from_error("Error on converting data to Rhai dynamic.", e)
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

    pub fn strict_error(path: Option<&String>) -> RenderError {
        let msg = match path {
            Some(path) => format!("Variable {:?} not found in strict mode.", path),
            None => "Value is missing in strict mode".to_owned(),
        };
        RenderError::new(&msg)
    }

    #[deprecated]
    pub fn with<E>(cause: E) -> RenderError
    where
        E: Error + Send + Sync + 'static,
    {
        let mut e = RenderError::new(cause.to_string());
        e.cause = Some(Box::new(cause));

        e
    }

    pub fn from_error<E>(error_kind: &str, cause: E) -> RenderError
    where
        E: Error + Send + Sync + 'static,
    {
        let mut e = RenderError::new(format!("{}: {}", error_kind, cause.to_string()));
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
        }
        MismatchingClosedDecorator(open: String, closed: String) {
            display("decorator {:?} was opened, but {:?} is closing",
                open, closed)
        }
        InvalidSyntax {
            display("invalid handlebars syntax.")
        }
        InvalidParam (param: String) {
            display("invalid parameter {:?}", param)
        }
        NestedSubexpression {
            display("nested subexpression is not supported")
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

impl Error for TemplateError {}

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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match (self.line_no, self.column_no, &self.segment) {
            (Some(line), Some(col), &Some(ref seg)) => writeln!(
                f,
                "Template error: {}\n    --> Template error in \"{}\":{}:{}\n     |\n{}     |\n     = reason: {}",
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
    /// A combined error type for `TemplateError` and `IOError`
    #[derive(Debug)]
    pub enum TemplateFileError {
        TemplateError(err: TemplateError) {
            from()
            source(err)
            display("{}", err)
        }
        IOError(err: IOError, name: String) {
            source(err)
            display("Template \"{}\": {}", name, err)
        }
    }
}

#[cfg(feature = "dir_source")]
impl From<WalkdirError> for TemplateFileError {
    fn from(error: WalkdirError) -> TemplateFileError {
        let path_string: String = error
            .path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        TemplateFileError::IOError(IOError::from(error), path_string)
    }
}

quick_error! {
    /// A combined error type for `TemplateError`, `IOError` and `RenderError`
    #[derive(Debug)]
    pub enum TemplateRenderError {
        TemplateError(err: TemplateError) {
            from()
            source(err)
            display("{}", err)
        }
        RenderError(err: RenderError) {
            from()
            source(err)
            display("{}", err)
        }
        IOError(err: IOError, name: String) {
            source(err)
            display("Template \"{}\": {}", name, err)
        }
    }
}

impl TemplateRenderError {
    pub fn as_render_error(&self) -> Option<&RenderError> {
        if let TemplateRenderError::RenderError(ref e) = *self {
            Some(&e)
        } else {
            None
        }
    }
}

#[cfg(feature = "script_helper")]
quick_error! {
    #[derive(Debug)]
    pub enum ScriptError {
        IOError(err: IOError) {
            from()
            source(err)
        }
        ParseError(err: ParseError) {
            from()
            source(err)
        }
    }
}

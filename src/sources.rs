use crate::error::{TemplateError, TemplateErrorReason};
use crate::template::Template;

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

pub(crate) trait Source {
    type Item;
    type Error;

    fn load(&self) -> Result<Self::Item, Self::Error>;
}

pub(crate) struct FileTemplateSource {
    name: String,
    path: PathBuf,
}

impl FileTemplateSource {
    pub(crate) fn new(path: PathBuf, name: String) -> FileTemplateSource {
        FileTemplateSource { path, name }
    }
}

impl Source for FileTemplateSource {
    type Item = Template;
    type Error = TemplateError;

    fn load(&self) -> Result<Self::Item, Self::Error> {
        let mut reader =
            BufReader::new(File::open(&self.path).map_err(|e| {
                TemplateError::of(TemplateErrorReason::IoError(e, self.name.clone()))
            })?);

        let mut buf = String::new();
        reader
            .read_to_string(&mut buf)
            .map_err(|e| TemplateError::of(TemplateErrorReason::IoError(e, self.name.clone())))?;

        Template::compile_with_name(buf, self.name.clone())
    }
}

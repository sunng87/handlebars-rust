use crate::error::TemplateError;
use crate::template::Template;

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub(crate) trait Source {
    type Item;
    type Error;

    fn load(&mut self) -> Result<Self::Item, Self::Error>;
}

pub(crate) struct FileTemplateSource {
    path: Path,
}

impl FileTemplateSource {
    fn new(path: Path) -> FileTemplateSource {
        FileTemplateSource { path }
    }
}

impl Source for FileTemplateSource {
    type Item = Template;
    type Error = TemplateError;

    fn load(&mut self) -> Result<Self::Item, Self::Error> {
        let mut reader = BufReader::new(
            File::open(tpl_path).map_err(|e| TemplateFileError::IOError(e, name.to_owned()))?,
        );
        let mut buf = String::new();
        tpl_source
            .read_to_string(&mut buf)
            .map_err(|e| TemplateFileError::IOError(e, name.to_owned()))?;
    }
}

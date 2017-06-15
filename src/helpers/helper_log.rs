use helpers::HelperDef;
use registry::Registry;
use context::JsonRender;
use render::{RenderContext, Helper};
use error::RenderError;

#[derive(Clone, Copy)]
pub struct LogHelper;

impl HelperDef for LogHelper {
    fn call(&self, h: &Helper, _: &Registry, _: &mut RenderContext) -> Result<(), RenderError> {
        let param = try!(h.param(0).ok_or_else(|| {
            RenderError::new("Param not found for helper \"log\"")
        }));

        info!(
            "{}: {}",
            param.path().unwrap_or(&"".to_owned()),
            param.value().render()
        );

        Ok(())
    }
}

pub static LOG_HELPER: LogHelper = LogHelper;

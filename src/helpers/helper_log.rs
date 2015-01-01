use helpers::{HelperDef};
use template::{Helper};
use registry::{Registry};
use context::{Context};
use render::{RenderContext, RenderError, render_error, EMPTY};

use log;

#[deriving(Copy)]
pub struct LogHelper;

impl HelperDef for LogHelper {
    fn call(&self, c: &Context, h: &Helper, r: &Registry, rc: &mut RenderContext) -> Result<String, RenderError> {
        let param = h.params().get(0);

        if param.is_none() {
            return Err(render_error("Param not found for helper \"log\""));
        }

        let name = param.unwrap();

        let value = if name.starts_with("@") {
            rc.get_local_var(name)
        } else {
            c.navigate(rc.get_path(), name)
        };

        log!(log::INFO, "{}: {}", name, value);

        Ok(EMPTY.to_string())
    }
}

pub static LOG_HELPER: LogHelper = LogHelper;

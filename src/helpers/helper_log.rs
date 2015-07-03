use helpers::{HelperDef};
use registry::{Registry};
use context::{Context};
use render::{RenderContext, RenderError, render_error, Helper};

#[derive(Clone, Copy)]
pub struct LogHelper;

impl HelperDef for LogHelper {
    fn call(&self, c: &Context, h: &Helper, _: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        let param = h.param(0);

        if param.is_none() {
            return Err(render_error("Param not found for helper \"log\""));
        }

        let name = param.unwrap();

        let value = if name.starts_with("@") {
            rc.get_local_var(name)
        } else {
            c.navigate(rc.get_path(), name)
        };

        info!("{}: {}", name, value);

        Ok(())
    }
}

pub static LOG_HELPER: LogHelper = LogHelper;

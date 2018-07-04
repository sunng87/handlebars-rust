use context::Context;
use error::RenderError;
use helpers::{HelperDef, HelperResult};
use output::Output;
use registry::Registry;
use render::{Helper, RenderContext};
use value::JsonRender;

#[derive(Clone, Copy)]
pub struct LogHelper;

impl HelperDef for LogHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper,
        _: &Registry,
        _: &Context,
        _: &mut RenderContext,
        _: &mut Output,
    ) -> HelperResult {
        let param = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"log\""))?;

        info!(
            "{}: {}",
            param.path().unwrap_or(&"".to_owned()),
            param.value().render()
        );

        Ok(())
    }
}

pub static LOG_HELPER: LogHelper = LogHelper;

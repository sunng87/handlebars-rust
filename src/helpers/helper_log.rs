use context::Context;
#[cfg(not(feature = "no_logging"))]
use error::RenderError;
use helpers::{HelperDef, HelperResult};
use output::Output;
use registry::Registry;
use render::{Helper, RenderContext};
#[cfg(not(feature = "no_logging"))]
use value::JsonRender;

#[derive(Clone, Copy)]
pub struct LogHelper;

#[cfg(not(feature = "no_logging"))]
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
        let level = h.hash_get("level")
            .and_then(|v| v.value().as_str())
            .unwrap_or("info");

        match level {
            "trace" => trace!("{}: {}",
                              param.path().unwrap_or(&"".to_owned()),
                              param.value().render()),
            "debug" => debug!("{}: {}",
                            param.path().unwrap_or(&"".to_owned()),
                            param.value().render()),
            "info" => info!("{}: {}",
                            param.path().unwrap_or(&"".to_owned()),
                            param.value().render()),
            "warn" => warn!("{}: {}",
                            param.path().unwrap_or(&"".to_owned()),
                            param.value().render()),
            "error" => error!("{}: {}",
                              param.path().unwrap_or(&"".to_owned()),
                              param.value().render()),
            _ => {}
        };
        Ok(())
    }
}

#[cfg(feature = "no_logging")]
impl HelperDef for LogHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        _: &Helper,
        _: &Registry,
        _: &Context,
        _: &mut RenderContext,
        _: &mut Output,
    ) -> HelperResult {
        Ok(())
    }
}

pub static LOG_HELPER: LogHelper = LogHelper;

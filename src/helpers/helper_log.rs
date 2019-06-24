use crate::context::Context;
#[cfg(not(feature = "no_logging"))]
use crate::error::RenderError;
use crate::helpers::{HelperDef, HelperResult};
use crate::output::Output;
use crate::registry::Registry;
use crate::render::{Helper, RenderContext};
#[cfg(not(feature = "no_logging"))]
use crate::value::JsonRender;
#[cfg(not(feature = "no_logging"))]
use log::Level;
#[cfg(not(feature = "no_logging"))]
use std::str::FromStr;

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
        _: &mut dyn Output,
    ) -> HelperResult {
        let param = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"log\""))?;
        let level = h
            .hash_get("level")
            .and_then(|v| v.value().as_str())
            .unwrap_or("info");

        if let Ok(log_level) = Level::from_str(level) {
            log!(
                log_level,
                "{}: {}",
                param.path().unwrap_or(&"".to_owned()),
                param.value().render()
            )
        } else {
            return Err(RenderError::new(&format!(
                "Unsupported logging level {}",
                level
            )));
        }
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

#![allow(dead_code, unused_variables)]
#![feature(unboxed_closures, phase)]
#![experimental]

extern crate serialize;
extern crate regex;
#[phase(plugin)]
extern crate regex_macros;

pub use self::template::{Template, Helper};
pub use self::registry::{Registry};
pub use self::render::{Renderable, RenderError, RenderContext};
pub use self::helpers::{HelperDef};
pub use self::context::{Context, JsonRender, JsonTruthy};

mod template;
mod registry;
mod render;
mod helpers;
mod context;

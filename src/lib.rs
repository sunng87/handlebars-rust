#![allow(dead_code, unused_variables)]
#![feature(unboxed_closures, phase)]
#![experimental]

extern crate serialize;
extern crate regex;
#[phase(plugin)]
extern crate regex_macros;

pub mod template;
pub mod registry;
pub mod render;
pub mod helpers;
pub mod context;

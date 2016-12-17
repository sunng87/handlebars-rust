//! # Handlebars
//! Handlebars is a modern and extensible templating solution originally created in the JavaScript world. It's used by many popular frameworks like [Ember.js](http://emberjs.com) and Chaplin. It's also ported to some other platforms such as [Java](https://github.com/jknack/handlebars.java).
//!
//! And this is handlebars Rust implementation, designed for server-side page generation. It's a general-purpose library so you use it for any kind of text generation.
//!
//! ## Why (this) Handlebars?
//!
//! Handlebars is a real-world templating system that you can use to build
//! your application without pain.
//!
//! ### Features
//!
//! #### Isolation of Rust and HTML
//!
//! This library doesn't attempt to use some macro magic to allow you to
//! write your template within your rust code. I admit that it's fun to do
//! that but it doesn't fit real-world use case.
//!
//! #### Limited but essential control structure built-in
//!
//! Only essential control directive `if` and `each` were built-in. This
//! prevents you to put too much application logic into your template.
//!
//! #### Extensible helper system
//!
//! You can write your own helper with Rust! It can be a block helper or
//! inline helper. Put you logic into the helper and don't repeat
//! yourself.
//!
//! A helper can be as a simple as a Rust function like:
//!
//! ```ignore
//! fn hex_helper (h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
//!     let param = h.param(0).unwrap();
//!     let rendered = format!("{:x}", param.value().render());
//!     try!(rc.writer.write(rendered.into_bytes().as_ref()));
//!     Ok(())
//! }
//!
//! /// register the helper
//! handlebars.register_helper("hex", Box::new(hex_helper));
//! ```
//!
//! #### Template inheritance
//!
//! Every time I look into a templating system, I will investigate its
//! support for [template
//! inheritance](https://docs.djangoproject.com/en/1.9/ref/templates/language/#template-inh
//! eritance).
//!
//! Template include is not enough. In most case you will need a skeleton
//! of page as parent (header, footer, etc.), and embed you page into this
//! parent.
//!
//! You can find a real example for template inheritance in
//! `examples/partials.rs`, and templates used by this file.
//!
//! ### Limitations
//!
//! * This implementation is **not fully compatible** with the original
//!   javascript version
//! * As a static typed language, it's a little verbose to use handlebars
//! * You will have to make your data `ToJson`-able, so we can render
//!   it. If you are on nightly channel, we have [a syntax
//!   extension](https://github.com/sunng87/tojson_macros) to generate
//!   default `ToJson` implementation for you. If you use
//!   [serde](https://github.com/serde-rs/serde), you can enable
//!   `serde_type` feature of handlebars-rust and add `#[Serialize]` for
//!   your types.
//!
//! ## Usage
//!
//! ### Template Creation and Registration
//!
//! Templates are created from String and registered to `Handlebars` with a name.
//!
//! ```
//!
//! extern crate handlebars;
//!
//! use handlebars::Handlebars;
//!
//! fn main() {
//!   let mut handlebars = Handlebars::new();
//!   let source = "hello {{world}}";
//!
//!   //compile returns an Option, we use unwrap() to deref it directly here
//!   handlebars.register_template_string("helloworld", source.to_string())
//!           .ok().unwrap();
//! }
//! ```
//!
//! On registeration, the template is parsed and cached in the registry. So further
//! usage will benifite from the one-time work. Also features like include, inheritance
//! that involves template reference requires you to register those template first so
//! the registry can find it.
//!
//! If you template is small or just to expirement, you can use `template_render` APIs
//! without registeration.
//!
//! ### Rendering Something
//!
//! I should say that rendering is a little tricky. Since handlebars is originally a JavaScript templating framework. It supports dynamic features like duck-typing, truthy/falsey values. But for a static language like Rust, this is a little difficult. As a solution, I'm using the `serialize::json::Json` internally for data rendering, which seems good by far.
//!
//! That means, if you want to render something, you have to ensure that it implements the `rustc_serialize::json::ToJson` trait. Luckily, most built-in types already have trait. However, if you want to render your custom struct, you need to implement this trait manually, or use [tojson_macros](https://github.com/sunng87/tojson_macros) to generate default `ToJson` implementation.
//!
//! You can use default `render` function to render a template into `String`. From 0.9, there's `renderw` to render text into anything of `std::io::Write`.
//!
//! From 0.13, we also support serde types by a feature flag `serde_type`.
//!
//! ```ignore
//! extern crate rustc_serialize;
//! extern crate handlebars;
//!
//! use rustc_serialize::json::{Json, ToJson};
//! use std::collections::BTreeMap;
//!
//! use handlebars::Handlebars;
//!
//! struct Person {
//!   name: String,
//!   age: i16,
//! }
//!
//! impl ToJson for Person {
//!   fn to_json(&self) -> Json {
//!     let mut m: BTreeMap<String, Json> = BTreeMap::new();
//!     m.insert("name".to_string(), self.name.to_json());
//!     m.insert("age".to_string(), self.age.to_json());
//!     m.to_json()
//!   }
//! }
//!
//! fn main() {
//!   let source = "Hello, {{name}}";
//!
//!   let mut handlebars = Handlebars::new();
//!   handlebars.register_template_string("hello", source.to_string())
//!           .ok().unwrap();
//!
//!   let data = Person {
//!       name: "Ning Sun".to_string(),
//!       age: 27
//!   };
//!   let result = handlebars.render("hello", &data);
//! }
//! ```
//!
//! Or if you don't need the template to be cached or referenced by other ones, you can
//! simply render it without registering.
//!
//! ```ignore
//! fn main() {
//!   let source = "Hello, {{name}}";
//!
//!   let mut handlebars = Handlebars::new();
//!
//!   let data = Person {
//!       name: "Ning Sun".to_string(),
//!       age: 27
//!   };
//!   let result = handlebars.template_render("Hello, {{name}}", &data);
//! }
//! ```
//!
//!
//! #### Escaping
//!
//! As per the handlebars spec, output using `{{expression}}` is escaped by default (to be precise, the characters `&"<>` are replaced by their respective html / xml entities). However, since the use cases of a rust template engine are probably a bit more diverse than those of a JavaScript one, this implementation allows the user to supply a custom escape function to be used instead. For more information see the `EscapeFn` type and `Handlebars::register_escape_fn()` method.
//!
//! ### Custom Helper
//!
//! Handlebars is nothing without helpers. You can also create your own helpers with rust. Helpers in handlebars-rust are custom struct implements the `HelperDef` trait, concretely, the `call` function. For your convenience, most of stateless helpers can be implemented as bare functions.
//!
//! ```
//!
//! extern crate handlebars;
//!
//! use std::io::Write;
//! use handlebars::{Handlebars, HelperDef, RenderError, RenderContext, Helper, Context, JsonRender};
//!
//! // implement by a structure impls HelperDef
//! #[derive(Clone, Copy)]
//! struct SimpleHelper;
//!
//! impl HelperDef for SimpleHelper {
//!   fn call(&self, h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
//!     let param = h.param(0).unwrap();
//!
//!     try!(rc.writer.write("Ny helper dumps: ".as_bytes()));
//!     try!(rc.writer.write(param.value().render().into_bytes().as_ref()));
//!     Ok(())
//!   }
//! }
//!
//! // implement via bare function
//! fn another_simple_helper (h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
//!     let param = h.param(0).unwrap();
//!
//!     try!(rc.writer.write("My second helper dumps: ".as_bytes()));
//!     try!(rc.writer.write(param.value().render().into_bytes().as_ref()));
//!     Ok(())
//! }
//!
//!
//! fn main() {
//!   let mut handlebars = Handlebars::new();
//!   handlebars.register_helper("simple-helper", Box::new(SimpleHelper));
//!   handlebars.register_helper("another-simple-helper", Box::new(another_simple_helper));
//!   // via closure
//!   handlebars.register_helper("closure-helper",
//!       Box::new(|h: &Helper, r: &Handlebars, rc: &mut RenderContext| -> Result<(), RenderError>{
//!         try!(rc.writer.write("...".as_bytes()));
//!         Ok(())
//!       }));
//!
//!   //...
//! }
//! ```
//!
//! #### Arguments of HelpDef
//!
//! You can get data from the `Helper` argument about the template information:
//!
//! * `name()` for the helper name. This is known to you for most situation but if you are defining `helperMissing` or `blockHelperMissing`, this is important.
//! * `params()` is a vector of String as params in helper, like `{{#somehelper param1 param2 param3}}`.
//! * `hash()` is a map of String key and Json value, defined in helper as `{{@somehelper a=1 b="2" c=true}}`.
//! * `template()` gives you the nested template of block helper.
//! * `inverse()` gives you the inversed template of it, inversed template is the template behind `{{else}}`.
//!
//! You can learn more about helpers by looking into source code of built-in helpers.
//!
//! #### Built-in Helpers
//!
//! * `{{#raw}} ... {{/raw}}` escape handlebars expression within the block
//! * `{{#if ...}} ... {{else}} ... {{/if}}` if-else block
//! * `{{#unless ...}} ... {{else}} .. {{/unless}}` if-not-else block
//! * `{{#each ...}} ... {{/each}}` iterates over an array or object. Handlebar-rust doesn't support mustach iteration syntax so use this instead.
//! * `{{#with ...}} ... {{/with}}` change current context. Similar to {{#each}}, used for replace corresponding mustach syntax.
//! * `{{lookup ... ...}}` get value from array by `@index` or `@key`
//! * `{{#partial ...}} ... {{/partial}}` template reuse, used to replace block with same name
//! * `{{#block ...}} ... {{/block}}` template reuse, used to be replaced by partial with same name, with default content if partial not found.
//! * `{{> ...}}` include template with name
//! * `{{log ...}}` log value with rust logger, default level: INFO. Currently you cannot change the level.
//!

#![allow(dead_code)]
#![recursion_limit = "200"]

#[macro_use]
extern crate log;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate pest;
#[macro_use]
extern crate lazy_static;

extern crate regex;

#[cfg(test)]
#[macro_use]
extern crate maplit;

#[cfg(all(feature = "rustc_ser_type", not(feature = "serde_type")))]
extern crate rustc_serialize as serialize;

#[cfg(feature = "serde_type")]
extern crate serde_json;

pub use self::template::Template;
pub use self::error::{TemplateError, TemplateFileError, TemplateRenderError};
pub use self::registry::{EscapeFn, no_escape, html_escape, Registry as Handlebars};
pub use self::render::{Renderable, RenderError, RenderContext, Helper, ContextJson};
pub use self::helpers::HelperDef;
pub use self::context::{Context, JsonRender, JsonTruthy};

mod grammar;
mod template;
mod error;
mod registry;
mod render;
mod helpers;
mod context;
mod support;
mod directives;
#[cfg(feature="partial4")]
mod partial;

//! # Handlebars
//!
//! [Handlebars](http://handlebarsjs.com/) is a modern and extensible templating solution originally created in the JavaScript world. It's used by many popular frameworks like [Ember.js](http://emberjs.com) and Chaplin. It's also ported to some other platforms such as [Java](https://github.com/jknack/handlebars.java).
//!
//! And this is handlebars Rust implementation, designed for general purpose text generation.
//!
//! ## Quick Start
//!
//! ```
//! use std::collections::BTreeMap;
//! use handlebars::Handlebars;
//!
//! fn main() {
//!   // create the handlebars registry
//!   let mut handlebars = Handlebars::new();
//!
//!   // register the template. The template string will be verified and compiled.
//!   let source = "hello {{world}}";
//!   assert!(handlebars.register_template_string("t1", source).is_ok());
//!
//!   // Prepare some data.
//!   //
//!   // The data type should implements `rustc_serialize::json::ToJson` or
//!   // `serde::Serialize`
//!   let mut data = BTreeMap::new();
//!   data.insert("world".to_string(), "世界!".to_string());
//!   assert_eq!(handlebars.render("t1", &data).unwrap(), "hello 世界!");
//! }
//! ```
//!
//! In this example, we created a template registry and registered a template named `t1`.
//! Then we rendered a `BTreeMap` with an entry of key `world`, the result is just what
//! we expected.
//!
//! I recommend you to walk through handlebars.js' [intro page](http://handlebarsjs.com)
//! if you are not quite familiar with the template language itself.
//!
//! ## Rational: Why (this) Handlebars?
//!
//! Handlebars is a real-world templating system that you can use to build
//! your application without pain.
//!
//! ### Features
//!
//! #### Isolation of Rust and HTML
//!
//! This library doesn't attempt to use some macro magic to allow you to
//! write your template within your rust code. I admit that it's fun (and feel cool) to do
//! that but it doesn't fit real-world use case in my opinion.
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
//! #### Compatibility with JavaScript version
//!
//! This implementation is **not fully compatible** with the original javascript version.
//!
//! First of all, mustache block is not supported. I suggest you to use `#if` and `#each` for
//! same functionality.
//!
//! There are some other minor features missing:
//!
//! * Chained else [#12](https://github.com/sunng87/handlebars-rust/issues/12)
//!
//! Feel free to fire an issue on [github](https://github.com/sunng87/handlebars-rust/issues) if
//! you find missing features.
//!
//! #### Static typed
//!
//! As a static typed language, it's a little verbose to use handlebars.
//! You will have to make your data `ToJson`-able, so we can render
//! it. If you are on nightly channel, we have [a syntax
//! extension](https://github.com/sunng87/tojson_macros) to generate
//! default `ToJson` implementation for you. If you prefer
//! [serde](https://github.com/serde-rs/serde), you can enable `serde_type` feature
//! of handlebars-rust and add `#[Serialize]` for your types.
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
//!   assert!(handlebars.register_template_string("t1", source).is_ok())
//! }
//! ```
//!
//! On registeration, the template is parsed, compiled and cached in the registry. So further
//! usage will benifite from the one-time work. Also features like include, inheritance
//! that involves template reference requires you to register those template first with
//! a name so the registry can find it.
//!
//! If you template is small or just to expirement, you can use `template_render` API
//! without registration.
//!
//! ```
//! use handlebars::Handlebars;
//! use std::collections::BTreeMap;
//!
//! fn main() {
//!   let mut handlebars = Handlebars::new();
//!   let source = "hello {{world}}";
//!
//!   let mut data = BTreeMap::new();
//!   data.insert("world".to_string(), "世界!".to_string());
//!   assert_eq!(handlebars.template_render(source, &data).unwrap(),"hello 世界!".to_owned());
//! }
//! ```
//!
//! ### Rendering Something
//!
//! Since handlebars is originally based on JavaScript type system. It supports dynamic features like duck-typing, truthy/falsey values. But for a static language like Rust, this is a little difficult. As a solution, we are using the `serialize::json::Json` internally for data rendering.
//!
//! That means, if you want to render something, you have to ensure the data type implements the `rustc_serialize::json::ToJson` trait (or serde's `Serialize`). Luckily, most rust internal types already have that trait. However, if you want to render your custom struct, you need to implement it manually, or use [tojson_macros](https://github.com/sunng87/tojson_macros) to generate default `ToJson` implementation.
//!
//! You can use default `render` function to render a template into `String`. From 0.9, there's `renderw` to render text into anything of `std::io::Write`.
//!
//! ```ignore
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
//!   assert!(handlebars.register_template_string("hello", source).is_ok());
//!
//!
//!   let data = Person {
//!       name: "Ning Sun".to_string(),
//!       age: 27
//!   };
//!   assert_eq!(handlebars.render("hello", &data).unwrap(), "Hello, Ning Sun".to_owned());
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
//!   assert_eq!(handlebars.template_render("Hello, {{name}}", &data).unwrap(),
//!       "Hello, Ning Sun".to_owned());
//! }
//! ```
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
//!     try!(rc.writer.write("1st helper: ".as_bytes()));
//!     try!(rc.writer.write(param.value().render().into_bytes().as_ref()));
//!     Ok(())
//!   }
//! }
//!
//! // implement via bare function
//! fn another_simple_helper (h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
//!     let param = h.param(0).unwrap();
//!
//!     try!(rc.writer.write("2nd helper: ".as_bytes()));
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
//!           let param = h.param(0).unwrap();
//!
//!           try!(rc.writer.write("3rd helper: ".as_bytes()));
//!           try!(rc.writer.write(param.value().render().into_bytes().as_ref()));
//!           Ok(())
//!       }));
//!
//!   let tpl = "{{simple-helper 1}}\n{{another-simple-helper 2}}\n{{closure-helper 3}}";
//!   assert_eq!(handlebars.template_render(tpl, &()).unwrap(),
//!       "1st helper: 1\n2nd helper: 2\n3rd helper: 3".to_owned());
//! }
//! ```
//! Data available to helper can be found in [Helper](struct.Helper.html). And there are more
//! examples in [HelperDef](trait.HelperDef.html) page.
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
//! * `{{> ...}}` include template with name
//! * `{{log ...}}` log value with rust logger, default level: INFO. Currently you cannot change the level.
//!
//! ### Template inheritance
//!
//! Handlebarsjs partial system is fully supported in this implementation.
//! Check [example](https://github.com/sunng87/handlebars-rust/blob/master/examples/partials.rs#L49) for detail.
//!
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

#[cfg(test)]
#[macro_use]
extern crate maplit;
#[cfg(test)]
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
extern crate serde;

extern crate regex;
extern crate serde_json;

pub use self::template::Template;
pub use self::error::{TemplateError, TemplateFileError, TemplateRenderError};
pub use self::registry::{EscapeFn, no_escape, html_escape, Registry as Handlebars};
pub use self::render::{Renderable, Evaluable, RenderError, RenderContext, Helper, ContextJson,
                       Directive as Decorator};
pub use self::helpers::HelperDef;
pub use self::directives::DirectiveDef as DecoratorDef;
pub use self::context::{Context, JsonRender, to_json};

mod grammar;
mod template;
mod error;
mod registry;
mod render;
mod helpers;
mod context;
mod support;
mod directives;
#[cfg(not(feature="partial_legacy"))]
mod partial;

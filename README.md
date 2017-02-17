handlebars-rust
===============

Rust templating with [Handlebars templating language](https://handlebarsjs.com).

[![Build Status](https://travis-ci.org/sunng87/handlebars-rust.svg?branch=master)](https://travis-ci.org/sunng87/handlebars-rust)
[![](http://meritbadge.herokuapp.com/handlebars)](https://crates.io/crates/handlebars)
[![](https://img.shields.io/crates/d/handlebars.svg)](https://crates.io/crates/handlebars)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Gitter](https://img.shields.io/gitter/room/sunng87/handlebars-rust.svg?maxAge=2592000)](https://gitter.im/sunng87/handlebars-rust)
[![Docs](https://docs.rs/handlebars/badge.svg)](https://docs.rs/handlebars/)

## Getting Started

If you are not familiar with [handlebars language
syntax](https://handlebarsjs.com), it is recommended to walk through
their introduction first.

Check `render` example in the source tree. The example shows you how
to:

* Create a `Handlebars` registry and register the template from files;
* Create a custom Helper with closure or struct implementing
 `HelperDef`, and register it;
* Define and prepare some data;
* Render it;

Run `cargo run --example render` to see results.
(or `RUST_LOG=handlebars=info cargo run --example render` for logging
output).

Checkout `examples/` for more concrete demos of current API.

From 0.13, you can use either `rustc_serialize` or `serde` for your
data type. By default we use `ToJson` from `rustc_serialize` to
convert your data into handlebars internal types. If you use `serde`
framework in your project, you can enable `serde_type` feature of this
crate and we will use `Serialize` from `serde` to convert.

## Documents

[Rust
doc](http://sunng87.github.io/handlebars-rust/handlebars/index.html).

## Changelog

Change log is available in the source tree named as `CHANGELOG.md`.

## Why (this) Handlebars?

Handlebars is a real-world templating system that you can use to build
your application without pain.

### Features

#### Isolation of Rust and HTML

This library doesn't attempt to use some macro magic to allow you to
write your template within your rust code. I admit that it's fun to do
that but it doesn't fit real-world use case.

#### Limited but essential control structure built-in

Only essential control directive `if` and `each` were built-in. This
prevents you to put too much application logic into your template.

#### Extensible helper system

You can write your own helper with Rust! It can be a block helper or
inline helper. Put you logic into the helper and don't repeat
yourself.

A helper can be as a simple as a Rust function like:

```rust
fn hex_helper (h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
    // just for example, add error check for unwrap
    let param = h.param(0).unwrap().value();
    let rendered = format!("0x{:x}", param.as_u64().unwrap());
    try!(rc.writer.write(rendered.into_bytes().as_ref()));
    Ok(())
}

/// register the helper
handlebars.register_helper("hex", Box::new(hex_helper));
```

And using it in your template:

```handlebars
{{hex my_value}}
```

#### Template inheritance

Every time I look into a templating system, I will investigate its
support for [template
inheritance](https://docs.djangoproject.com/en/1.9/ref/templates/language/#template-inheritance).

Template include is not sufficient for template reuse. In most case
you will need a skeleton of page as parent (header, footer, etc.), and
embed you page into this parent.

You can find a real example for template inheritance in
`examples/partials.rs`, and templates used by this file.

From 0.23 we support Handlebars 4.0 partial syntax by
default. Original partial syntax via `block`, `partial` helpers are
still supported via feature flag `partial_legacy`. Examples can be
find in `examples/partials.rs`.

#### WebAssembly compatible

You can use this handlebars implementation in your rust project that
compiles to WebAssembly. Checkout my fork of
[todomvc](https://github.com/sunng87/rust-todomvc) demo.

### Limitations

* This implementation is **not fully compatible** with the original
  javascript version. Specifically, mustache list iteration and null
  check doesn't work. But you can use `#each` and `#if` for same
  behavior.
* You will need to make your data `ToJson`-able, so we can render
  it. If you were on nightly channel, we have [a syntax
  extension](https://github.com/sunng87/tojson_macros) to generate
  default `ToJson` implementation for you. If you use
  [serde](https://github.com/serde-rs/serde), you can enable
  `serde_type` feature of handlebars-rust and add `#derive[Serialize]`
  for your types.

### Handlebars-js features supported in Handlebars-rust

* Expression / Block Helpers
* Built-in helpers
  * each
  * if
  * with
  * lookup
  * log
* Custom helper
* Parameter and hashes for helper, block params
* Partials, include, template inheritance
* Omitting whitespace with `~`
* Subexpression `{{(foo bar)}}`
* Json expression `a.b.[0]` and `a.b.[c]`
* RawHelper syntax `{{{{raw-helper}}}}...{{{{/raw-helper}}}}`
* Decorator, implemented in Rust way

### JavaScript implementation features we don't have

* Mustache block (use `if`/`each` instead)
* Chained else

Feel free to report an issue if you find something broken. We aren't
going to implement all features of handlebars-js, but we should have a
workaround for cases we don't support.

## Handlebars for Web Frameworks

* Iron: [handlebars-iron](https://github.com/sunng87/handlebars-iron)
* Rocket: [rocket/contrib](https://api.rocket.rs/rocket_contrib/struct.Template.html)

## Using handlebars-rust?

Add your project to our
[adopters](https://github.com/sunng87/handlebars-rust/wiki/adopters).

## License

This library (handlebars-rust) is open sourced under MIT License.

## Contact

[Ning Sun](https://github.com/sunng87) (sunng@about.me)

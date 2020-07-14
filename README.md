handlebars-rust
===============

[Handlebars templating language](https://handlebarsjs.com) implemented
in Rust and for Rust.

Handlebars-rust is the template engine that renders the official Rust website
[rust-lang.org](https://www.rust-lang.org), [its
book](https://doc.rust-lang.org/book/).

[![Build Status](https://travis-ci.org/sunng87/handlebars-rust.svg?branch=master)](https://travis-ci.org/sunng87/handlebars-rust)
[![](https://meritbadge.herokuapp.com/handlebars)](https://crates.io/crates/handlebars)
[![](https://img.shields.io/crates/d/handlebars.svg)](https://crates.io/crates/handlebars)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Docs](https://docs.rs/handlebars/badge.svg)](https://docs.rs/crate/handlebars/)
[![Donate](https://img.shields.io/badge/donate-liberapay-yellow.svg)](https://liberapay.com/Sunng/donate)
[![Donate](https://img.shields.io/badge/donate-buymeacoffee-yellow.svg)](https://www.buymeacoffee.com/Sunng)

## Getting Started

### Quick Start

```rust
extern crate handlebars;
#[macro_use]
extern crate serde_json;

use handlebars::Handlebars;

fn main() -> Result<(), Box<dyn Error>> {
    let mut reg = Handlebars::new();
    // render without register
    println!(
        "{}",
        reg.render_template("Hello {{name}}", &json!({"name": "foo"}))?
    );

    // register template using given name
    reg.register_template_string("tpl_1", "Good afternoon, {{name}}")?;
    println!("{}", reg.render("tpl_1", &json!({"name": "foo"}))?);
    Ok(())
}
```

### Code Example

If you are not familiar with [handlebars language
syntax](https://handlebarsjs.com), it is recommended to walk through
their introduction first.

Check the `render` example in the source tree. The example shows you how
to:

* Create a `Handlebars` registry and register the template from files;
* Create a custom Helper with closure or struct implementing
 `HelperDef`, and register it;
* Define and prepare some data;
* Render it;

Run `cargo run --example render` to see results
(or `RUST_LOG=handlebars=info cargo run --example render` for logging
output).

Checkout `examples/` for more concrete demos of the current API.


## Minimum Rust Version Policy

Handlebars will track Rust nightly and stable channel. When dropping
support for previous stable versions, I will bump **minor** version
and clarify in CHANGELOG.

### Rust compatibility table

| Handlebars version range | Minimum Rust version |
| --- | --- |
| ~3.0.0 | 1.32 |
| ~2.0.0 | 1.32 |
| ~1.1.0 | 1.30 |
| ~1.0.0 | 1.23 |

## Document

[Rust doc](https://docs.rs/crate/handlebars/).

## Changelog

Changelog is available in the source tree named as `CHANGELOG.md`.

## Contributor Guide

Any contribution to this library is welcomed. To get started into
development, I have several [Help
Wanted](https://github.com/sunng87/handlebars-rust/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22)
issues, with the difficulty level labeled. When running into any problem,
feel free to contact me on github.

I'm always looking for maintainers to work together on this library,
let me know (via email or anywhere in the issue tracker) if you
want to join.

## Donations

I'm now accepting donations on [liberapay](https://liberapay.com/Sunng/donate)
and [buymeacoffee](https://www.buymeacoffee.com/Sunng) if you find my
work helpful and want to keep it going.

[![buymeacoffee](https://www.buymeacoffee.com/assets/img/guidelines/download-assets-3.svg)](https://www.buymeacoffee.com/Sunng)

## Why (this) Handlebars?

Handlebars is a real-world templating system that you can use to build
your application without pain.

### Features

#### Isolation of Rust and HTML

This library doesn't attempt to use some macro magic to allow you to
write your template within your rust code. I admit that it's fun to do
that but it doesn't fit real-world use cases.

#### Limited but essential control structures built-in

Only essential control directives `if` and `each` are built-in. This
prevents you from putting too much application logic into your template.

#### Extensible helper system

You can write your own helper with Rust! It can be a block helper or
inline helper. Put your logic into the helper and don't repeat
yourself.

A helper can be as a simple as a Rust function like:

```rust
handlebars_helper!(hex: |v: i64| format!("0x{:x}", v));

/// register the helper
handlebars.register_helper("hex", Box::new(hex));
```

And using it in your template:

```handlebars
{{hex 16}}
```

With `script_helper` feature flag enabled, you can also create helpers
using [rhai](https://github.com/jonathandturner/rhai) script, just like JavaScript
for handlebars-js. This feature was in early stage. Its API was limited at the
moment, and can change in future.

#### Template inheritance

Every time I look into a templating system, I will investigate its
support for [template
inheritance](https://docs.djangoproject.com/en/1.9/ref/templates/language/#template-inheritance).

Template include is not sufficient for template reuse. In most cases
you will need a skeleton of page as parent (header, footer, etc.), and
embed your page into this parent.

You can find a real example of template inheritance in
`examples/partials.rs` and templates used by this file.

#### WebAssembly compatible

Handlebars 3.0 can be used in WebAssembly projects.

## Related Projects

### Web frameworks

* Iron: [handlebars-iron](https://github.com/sunng87/handlebars-iron)
* Rocket: [rocket/contrib](https://api.rocket.rs/v0.4/rocket_contrib/templates/index.html)
* Warp: [handlebars
  example](https://github.com/seanmonstar/warp/blob/master/examples/handlebars_template.rs)
* Tower-web: [Built-in](https://github.com/carllerche/tower-web)
* Actix: [handlebars
  example](https://github.com/actix/examples/blob/master/template_handlebars/src/main.rs)
* Tide: [tide-handlebars](https://github.com/No9/tide-handlebars)

### Adopters

The
[adopters](https://github.com/sunng87/handlebars-rust/wiki/Adopters)
page lists projects that uses handlebars for part of their
functionalities.

### Extensions

The
[extensions](https://github.com/sunng87/handlebars-rust/wiki/Extensions)
page has libraries that provide additional helpers, decorators and
outputs to handlebars-rust, and you can use in your own projects.

## License

This library (handlebars-rust) is open sourced under the MIT License.

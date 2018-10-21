handlebars-rust
===============

Rust templating with [Handlebars templating language](https://handlebarsjs.com).

[![Build Status](https://travis-ci.org/sunng87/handlebars-rust.svg?branch=master)](https://travis-ci.org/sunng87/handlebars-rust)
[![](http://meritbadge.herokuapp.com/handlebars)](https://crates.io/crates/handlebars)
[![](https://img.shields.io/crates/d/handlebars.svg)](https://crates.io/crates/handlebars)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Docs](https://docs.rs/handlebars/badge.svg)](https://docs.rs/crate/handlebars/)
[![Donate](https://img.shields.io/badge/donate-liberapay-yellow.svg)](https://liberapay.com/Sunng/donate)

## Getting Started

### Quick Start

```rust
extern crate handlebars;
#[macro_use]
extern crate serde_json;

use handlebars::Handlebars;

fn main() -> Result<(), Box<Error>> {
    let mut reg = Handlebars::new();
    // render without register
    println!(
        "{}",
        reg.render_template("Hello {{name}}", &json!({"name": "foo"}))?
    );

    // register template using given name
    reg.register_template_string("tpl_1", "Good afternoon, {{name}}")?;
    println!("{}", reg.render("tpl_1", &json!({"name": "foo"}))?);
}
```

### Code Example

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

## Document

[Rust doc](https://docs.rs/crate/handlebars/).

## Changelog

Change log is available in the source tree named as `CHANGELOG.md`.

## Contributor Guide

Any contribution to this library is welcomed. To get started into
development, I have several [Help
Wanted](https://github.com/sunng87/handlebars-rust/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22)
issue, with difficult level labeled. When running into any problem,
feel free to contact me on github.

I'm always looking for maintainers to work together on this library,
also let me know (via email or anywhere in the issue tracker) if you
want to join.

## Donation

I'm now accepting donation on [liberapay](https://liberapay.com/Sunng/donate),
if you find my work helpful and want to keep it going.

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
handlebars_helper!(hex: |v: i64| format!("0x{:x}", v));

/// register the helper
handlebars.register_helper("hex", Box::new(hex));
```

And using it in your template:

```handlebars
{{hex 16}}
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

#### WebAssembly compatible

Handlebars 1.0 can be used in WebAssembly projects with directory
source feature disabled. Adding handlebars to your project like this:

```
handlebars = { version = "1", features = ["no_dir_source"], default-features = false }
```

#### Strict mode

Handlebars, the language designed to work with JavaScript, has no
strict restriction on accessing non-existed fields or index. It
generates empty string for such case. However, in Rust we want a
little bit strict sometime.

By enabling `strict_mode` on handlebars:

```rust
handlebars.set_strict_mode(true);
```

You will get a `RenderError` when accessing field that not exists.

### Limitations

* This implementation is **not fully compatible** with the original
  javascript version. Specifically, mustache list iteration and null
  check doesn't work. But you can use `#each` and `#if` for same
  behavior.
* You will need to make your data `Serializable` on serde. We don't
  actually serialize data into JSON string or similar. However, we use
  JSON data type system in template render process.

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
* Warp: [handlebars
  example](https://github.com/seanmonstar/warp/blob/master/examples/handlebars_template.rs)
* Tower-web: [Built-in](https://github.com/carllerche/tower-web)

## Using handlebars-rust?

Add your project to our
[adopters](https://github.com/sunng87/handlebars-rust/wiki/adopters).

## License

This library (handlebars-rust) is open sourced under MIT License.

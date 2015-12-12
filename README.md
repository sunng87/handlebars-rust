handlebars-rust
===============

![4721045137_a05105da92_z](https://cloud.githubusercontent.com/assets/221942/10570641/aa0d31c4-7668-11e5-8ede-dca92bfd7299.jpg)

([Photo ref](https://www.flickr.com/photos/jg33/4721045137))

Rust templating with Handlebars.

[![Build Status](https://travis-ci.org/sunng87/handlebars-rust.svg?branch=master)](https://travis-ci.org/sunng87/handlebars-rust)
[![](http://meritbadge.herokuapp.com/handlebars)](https://crates.io/crates/handlebars)
[![](https://img.shields.io/crates/d/handlebars.svg)](https://crates.io/crates/handlebars)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

Thanks to [@blaenk](https://github.com/blaenk)'s patch,
handlebars-rust now compiles on nightly, beta and stable (1.4.0+) channel.

## Documents

[Rust doc](http://sunng87.github.io/handlebars-rust/handlebars/index.html).

## Usage

Check examples in the source. The example shows you how to:

* Create a `Handlebars` and register the template from files
* Create a custom Helper with closure or struct implementing `HelperDef`, and register it
* Render something

Run `cargo run --example render` to see results.
(or `RUST_LOG=handlebars=info cargo run --example render` for logging output).

Checkout `examples/` for more concrete demos of current API.

## Why Handlebars?

For information about handlebars, you will go to [handlebars.js](http://handlebarsjs.com).

It's my favorite templating tool by far. I used it in
[Delicious.com](https://delicious.com) as javascript-side template in
2013. Also I maintained a Clojure wrapper for Handlebars.java,
[hbs](http://github.com/sunng87/hbs). And you may notice the
close relationship between Ember.js and Rust community, handlebars is
the default templating language of Ember.js framework, which powers
[crates.io](http://crates.io).

Reasons I prefer Handlebars:

* Never ruin your Rust with HTML
* Never ruin your HTML with Rust
* Templating without a reuse mechanism is shit
* Templating without customization is nothing but shit

Handlebars provides:

* Separation of Rust and HTML
* A few control structures makes templating easier
* Few control structures stops you to put logic into templates
* Template reuse with include(`>`), `partial` and `block`
* Customize template behavior with **helper**s

Limitations:

* As a static typed language, it's a little verbose to use handlebars
* You will have to make your data `ToJson`-able, so we can render
it. If you are on nightly channel, we have [a syntax extension](https://github.com/sunng87/tojson_macros) to generate default `ToJson` implementation for you.

## Handlebars-js features supported in Handlebars-rust

* Expression / Block Helpers
* Built-in helpers
* Customizing helper
* Parameter and hashes for helper
* Partials, include
* Omitting whitespace with `~`
* Subexpression `{{(foo bar)}}`
* Json expression `a.b.[0]` and `a.b.[c]`

Feel free to report an issue if you find something broken. We aren't
going to implement all features of handlebars-js, but we should have a
workaround for cases we don't support.

## Handlebars for Iron

I have started another project
[handlebars-iron](https://github.com/sunng87/handlebars-iron) for
the Iron web framework.

## Break Change Log

No doubt that we will try our best to keep API compatible in each
release. But sometime we have to bring in break changes to improve the
design when worthy.

* 0.10.0: `renderw` now accepts `&Context` instead of `ToJson` to avoid
unnecessary clone in batch rendering. [90274e7](https://github.com/sunng87/handlebars-rust/commit/90274e75feeffa1d509abde62a8c70ac2c0a4e76)

## License

MIT, of course.

## Contact

[Ning Sun](https://github.com/sunng87) (sunng@about.me)

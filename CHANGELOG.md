# Change Log

## [Unreleased](https://github.com/sunng87/handlebars-rust/compare/3.2.1...Unreleased) - ReleaseDate

* [Added] Added two new APIs to reuse `Context` for rendering [#352]
* [Changed] Update rhai to 0.17 [#354]

## [3.2.1](https://github.com/sunng87/handlebars-rust/compare/3.2.0...3.2.1) - 2020-06-28

* [Fixed] block context leak introduced in 3.2.0, #346 [#349]

## [3.2.0](https://github.com/sunng87/handlebars-rust/compare/3.1.0...3.2.0) - 2020-06-28

* [Added] API to register an pre-processed template [#331]
* [Added] Helper macro now has support for named argument and helepr hash [#338]
* [Added] Added support for `$` expression that is part of mustache.js [#339]
* [Changed] Update rhai to 0.15 [#330]
* [Fixed] else block for `each` [#344]

## [3.1.0](https://github.com/sunng87/handlebars-rust/compare/3.0.1...3.1.0) - 2020-06-01

* [Added] All new rhai script helper
* [Added] multiple parameter support for log helper
* [Fixed] helper lookup priority
* [Changed] `Send` and `Sync` are not required for RenderContext local helper [#319]
* [Fixed] partial block when using path as name [#321]

## [3.0.1](https://github.com/sunng87/handlebars-rust/compare/3.0.0...3.0.1) - 2020-01-25

* [Fixed] Slash in partial path causing syntax error  #313

## [3.0.0](https://github.com/sunng87/handlebars-rust/compare/2.0.3...3.0.0) - 2020-01-24

* [Changed] Added lifetime specifier to `Handlebars` structure allowing helper definition to have non-static borrowed data #282
* [Changed] Removed hashbrown dependency #279
* [Changed] Features has been reorganized. `dir_source` were turned off by default. #289
* [Changed] Refactored `RenderContext` API to improve performance up to 5x over `2.0`
* [Added] Add new `BlockContext` API for helper developer to store block scope state #307
* [Fixed] `RenderError` should be `Send` and `Sync` #304

## [2.0.4](https://github.com/sunng87/handlebars-rust/compare/2.0.3...2.0.4) - 2020-01-06

* [Fixed] `RenderError` should be `Send` and `Sync` #304

## [2.0.3](https://github.com/sunng87/handlebars-rust/compare/2.0.2...2.0.3) - 2020-01-04

* [Fixed] deprecated warnings on rust 1.42 nightly, due to changes in
  `Error` trait

## [2.0.2](https://github.com/sunng87/handlebars-rust/compare/2.0.1...2.0.2) - 2019-09-06

* [Changed] Extended `eq` and `ne` helper for all json types #287
* [Changed] Removed `regex` and `lazy_static` crate to optimize dependency tree

## [2.0.1](https://github.com/sunng87/handlebars-rust/compare/2.0.0...2.0.1) - 2019-07-12
* [Changed] Fixed issue with block context #275
* [Changed] Added support for array index in block context #276
* [Changed] Deprecated RenderContext `concat_path`
* [Changed] Update hashbrown to 0.5.0

## [2.0.0](https://github.com/sunng87/handlebars-rust/compare/2.0.0-beta3...2.0.0) - 2019-07-02
* [Changed] Fixed more dyn trait warnings
* [Changed] #80 Fixed support for zero-param helper
* [Changed] Changed minimum Rust version to 1.32 as required by
  getrandom crate

## [2.0.0-beta.3](https://github.com/sunng87/handlebars-rust/compare/2.0.0-beta1...2.0.0-beta.3) - 2019-06-24

* [Changed] Block parameter revamp, fixed cases for #260 and #264
* [Changed] #265 Fixed block parameter order in `each` helper
* [Changed] #266 Accept any JSON value in boolean helpers
* [Changed] `RenderContext` API update, `evaluate_absolute` removed,
  use `@root` instead

## [2.0.0-beta.1](https://github.com/sunng87/handlebars-rust/compare/1.1.0...2.0.0-beta.1) - 2019-03-16

* [Changed] Everything changed in yanked 1.2.0
* [Changed] With Pest updated to 2.1, our minimal rust version is set
  to 1.31
* [Changed] Using hashbrown `HashMap` internally and externally,
  performance improvement up to 10%
* [Changed] strict mode also apply to return value of helper expression

## [1.2.0](https://github.com/sunng87/handlebars-rust/compare/1.1.0...1.2.0) - 2018-12-15

*This release is yanked.*

* [Changed] Using rust 2018 edition
* [Changed] Improve strict mode and only raise error when accessing
  missing fields in expression
* [Changed] Improved `get_helper` and `get_decorator` return type

## [1.1.0](https://github.com/sunng87/handlebars-rust/compare/1.0.5...1.1.0) - 2018-10-24

* [Added] New option `includeZero` for `if` helper
* [Added] New option `level` for `log` helper
* [Changed] Updated Pest to 2.0, moving minimal Rust version to 1.30

## [1.0.5](https://github.com/sunng87/handlebars-rust/compare/1.0.4...1.0.5) - 2018-10-04

* [Changed] Added feature `no_logging` for using handlebars in a
  logging provider.

## [1.0.4](https://github.com/sunng87/handlebars-rust/compare/1.0.3...1.0.4) - 2018-09-21

* [Changed] Fixed build on wasm
* [Changed] Added support for single-quote Json string literal

## [1.0.3](https://github.com/sunng87/handlebars-rust/compare/1.0.2...1.0.3) - 2018-08-29

* [Changed] Fixed build on Rust 1.23.0

## [1.0.2](https://github.com/sunng87/handlebars-rust/compare/1.0.1...1.0.2) - 2018-08-27

* [Changed] Update minimal dependency versions

## [1.0.1](https://github.com/sunng87/handlebars-rust/compare/1.0.0...1.0.1) - 2018-08-16

* [Changed] Added hidden/temp file filter to directory register

## [1.0.0](https://github.com/sunng87/handlebars-rust/compare/0.32.4...1.0.0) - 2018-07-18

* [Changed] Helper API finalized and new output API
* [Changed] New internal value API, reduced clone cost
* [Added] Helper macro
* [Added] New built-in helpers: `gt`, `lt` and some more
* [Added] Register template folder

## [0.32.4](https://github.com/sunng87/handlebars-rust/compare/0.32.3...0.32.4) - 2018-05-23

* [Changed] Keep compatibility with pre-1.26 rust by removing `impl
  Trait` on parameters

## [0.32.3](https://github.com/sunng87/handlebars-rust/compare/0.32.2...0.32.3) - 2018-05-21

* [Changed] Fixed escape syntax

## [0.32.2](https://github.com/sunng87/handlebars-rust/compare/0.32.1...0.32.2) - 2018-05-09

* [Changed] Fixed issue with processing handlebars comment

## [0.32.1](https://github.com/sunng87/handlebars-rust/compare/0.32.0...0.32.1) - 2018-05-02

* [Changed] Regex 1.0

## [0.32.0](https://github.com/sunng87/handlebars-rust/compare/0.30.1...0.32.0) - 2018-02-16

* [Added] Strict mode that raises `RenderError` on accessing
  non-existed field or array index.

## [0.31.0](https://github.com/sunng87/handlebars-rust/compare/0.30.1...0.31.0) - 2018-02-09
* [Changed] Fixed handlebars comment support, added html comment output
* [Changed] Removed some wasted string clones

## [0.30.1](https://github.com/sunng87/handlebars-rust/compare/0.30.0...0.30.1) - 2018-01-31
* [Changed] Added `Debug` for public types

## [0.30.0](https://github.com/sunng87/handlebars-rust/compare/0.30.0-beta.5...0.30.0) - 2018-01-21
* [Changed] Use pest 1.0

## [0.30.0-beta.5](https://github.com/sunng87/handlebars-rust/compare/0.30.0-beta.4...0.30.0-beta.5) - 2018-01-19

* [Changed] Improve `TemplateError` display. Now includes a segment of
  template string.
* [Changed] Updated `lazy_static` to 1.0
* [Changed] Renamed some render functions names.

## [0.30.0-beta.4](https://github.com/sunng87/handlebars-rust/compare/0.30.0-beta.3...0.30.0-beta.4) - 2017-11-20
* [Changed] Added `Sync` to the nested error of `RenderError`

## [0.30.0-beta.3](https://github.com/sunng87/handlebars-rust/compare/0.30.0-beta.2...0.30.0-beta.3) - 2017-11-16
* [Changed] Fixed issue `template_render` methods doesn't respect `source_map` setting

## [0.30.0-beta.2](https://github.com/sunng87/handlebars-rust/compare/0.30.0-beta.1...0.30.0-beta.2) - 2017-10-07
* [Changed] Fixed parsing keywords like `as`

## [0.30.0-beta.1](https://github.com/sunng87/handlebars-rust/compare/0.29.1...0.30.0-beta.1) - 2017-10-03

* [Changed] Upgrade pest to 1.0
* [Changed] Fixed template parsing issue when parameter starts with "as"
* [Changed] Added new HelperDef function to return JSON value
* [Changed] Added support for @root

## [0.29.1](https://github.com/sunng87/handlebars-rust/compare/0.29.0...0.29.1) - 2017-09-01

* [Changed] Remove `debug!` logging from render to avoid conflict when
  using handlebars as logging backend

## [0.29.0](https://github.com/sunng87/handlebars-rust/compare/0.28.3...0.29.0) - 2017-08-23

* [Changed] Align JSON path with original JavaScript implementation

## [0.28.3](https://github.com/sunng87/handlebars-rust/compare/0.28.2...0.28.3) - 2017-08-02

* [Changed] fixed support for escape, again

## [0.28.2](https://github.com/sunng87/handlebars-rust/compare/0.28.1...0.28.2) - 2017-08-01

* [Changed] Fixed support for escape `\\{{`. [#170](https://github.com/sunng87/handlebars-rust/issues/170)

## [0.28.1](https://github.com/sunng87/handlebars-rust/compare/0.28.0...0.28.1) - 2017-07-16

* [Changed] Mark `RenderError` with `Send` trait

## [0.28.0](https://github.com/sunng87/handlebars-rust/compare/0.27.0...0.28.0) - 2017-07-15

* [Changed] Fixed performance issue discussed in [#166](https://github.com/sunng87/handlebars-rust/issues/166)
* [Added] Added error cause `RenderError`

## [0.27.0](https://github.com/sunng87/handlebars-rust/compare/0.26.2...0.27.0) - 2017-06-03

* [Changed] `partial_legacy` is dropped
* [Changed] `context.navigate` now returns a `Result<&Json,RenderError>`. Error is raised when
  given path cannot be not parsed.
* [Changed] removed `context::extend` because it's like to ruin your context outside the helper.
* [Changed] `RenderContext` now owns `Context`, you can host a new Context for particular block
  helper.
* [Changed] Added some convenience functions to `RenderContext`. However, `RenderContext` may
  still change in future release.

## [0.26.1](https://github.com/sunng87/handlebars-rust/compare/0.25.3...0.26.1) - 2017-04-23

* [Changed] Updated to Serde 1.0
* [Changed] Dropped rustc_serialize, serde is now the default type system

## [0.25.3](https://github.com/sunng87/handlebars-rust/compare/0.25.2...0.25.3) - 2017-04-19

* [Changed] Fixed path up [#147](https://github.com/sunng87/handlebars-rust/issues/147)
* [Changed] Fixed duplicated template inclusion [#146](https://github.com/sunng87/handlebars-rust/issues/146)

## [0.25.2](https://github.com/sunng87/handlebars-rust/compare/0.25.1...0.25.2) - 2017-03-22

* [Changed] Fixed bug when including two partials with same name [#143](https://github.com/sunng87/handlebars-rust/issues/143)

## [0.25.1](https://github.com/sunng87/handlebars-rust/compare/0.25.0...0.25.1) - 2017-02-21

* [Added] Added support for braces escaping`\{{var}}`.

## [0.25.0](https://github.com/sunng87/handlebars-rust/compare/0.24.2...0.25.0) - 2017-01-28

* [Changed] Updated serde family to 0.9.x
* [Added] Added `to_json` function to convert data to `Json` or `Value`

## [0.24.2](https://github.com/sunng87/handlebars-rust/compare/0.24.1...0.24.2) - 2017-01-28

* [Added] Added support for `{{> @partial-block}}`

## [0.24.1](https://github.com/sunng87/handlebars-rust/compare/0.24.0...0.24.1) - 2016-12-30

* [Changed] Updated `regex` crate to 0.2, fixed WebAssembly support
* [Changed] Fixed error reporting in partial.

## [0.24.0](https://github.com/sunng87/handlebars-rust/compare/0.23.0...0.24.0) - 2016-12-30

* [Added] Decorator support: change context data and helpers during rendering
* [Changed] (**Breaking**) Helper trait changed, `Context` parameter no longer
  available, use `render_context.context()` instead.
* [Changed] (**Breaking**) Refactored Handlebars APIs, `Template` and
  `Context` are no longer exposed in public API.
* [Changed] Docs updated.

## [0.23.0](https://github.com/sunng87/handlebars-rust/compare/0.22.0...0.23.0) - 2016-12-12

* [Changed] `partial4` is now default. Use `partial_legacy` for previous version of template inheritance.
* [Changed] Corrected subexpression behavior. Subexpression result is treated as string.
* [Changed] Improved performance for render: better escape function and string writer buffer.

## [0.22.0](https://github.com/sunng87/handlebars-rust/compare/0.21.1...0.22.0) - 2016-10-29

* [Changed] Improved error reporting. Fixed display for several error
  types.
* [Changed] Dropped regex and lazystatic as dependency.
* [Changed] Examples refined.

## [0.21.1](https://github.com/sunng87/handlebars-rust/compare/0.21.0...0.21.1) - 2016-10-09

* [Changed] Fixed
  [#106](https://github.com/sunng87/handlebars-rust/issue/106), when
  property name contains `this`, it doesn't work

## [0.21.0](https://github.com/sunng87/handlebars-rust/compare/0.20.5...0.21.0) - 2016-09-27

* [Added] Block params support
  [#101](https://github.com/sunng87/handlebars-rust/pull/101)
* [Added] New partial syntax [#103](https://github.com/sunng87/handlebars-rust/pull/103)
* [Changed] Rewrite path parser, better support for `../`
  [#105](https://github.com/sunng87/handlebars-rust/pull/105)

## [0.20.5](https://github.com/sunng87/handlebars-rust/compare/0.20.5...0.20.4) - 2016-08-27

* [Changed] Fixed issue for using [] in expression
  [#100](https://github.com/sunng87/handlebars-rust/issue/100)

## [0.20.4](https://github.com/sunng87/handlebars-rust/compare/0.20.4...0.20.3) - 2016-08-27

* [Changed] Fixed error message for partials
  [#98](https://github.com/sunng87/handlebars-rust/issue/98)
* [Added] Added support for `else` in `each` block
  [#99](https://github.com/sunng87/handlebars-rust/issue/99)

## [0.20.3](https://github.com/sunng87/handlebars-rust/compare/0.20.3...0.20.2) - 2016-08-14

* [Changed] Fixed `with` used inside `each` block [#97](https://github.com/sunng87/handlebars-rust/pull/97)

## [0.20.2](https://github.com/sunng87/handlebars-rust/compare/0.20.2...0.20.0) - 2016-08-07

* [Changed] Allowed dash character in reference
  [#94](https://github.com/sunng87/handlebars-rust/pull/94)
* [Changed] Fixed path error in nested each helpers [#95](https://github.com/sunng87/handlebars-rust/pull/95)

## [0.20.0](https://github.com/sunng87/handlebars-rust/compare/0.20.0...0.19.1) - 2016-07-31

* [Changed] Updated serde to 0.8

## [0.19.1](https://github.com/sunng87/handlebars-rust/compare/0.19.1...0.19.0) - 2016-07-26

* [Changed] Fixed `../` path visitor bug in nested `#each`
  [#93](https://github.com/sunng87/handlebars-rust/issues/93)
* [Changed] Rollback 0.19.0 change for `#if`

## [0.19.0] - 2016-07-24

* [Changed] changed `&Path` to `AsRef<Path>`
* [Changed] Fixed "../" path visitor in `#each` and `#if`.
* [Added] `set_local_path_root` and `get_local_path_root` for
  `RenderContext`.

## [0.18.2] - 2016-07-11

* [Changed] Disable `rustc_type` when `serde_type` enabled.

## [0.18.1] - 2016-07-04

* [Changed] Allow `-` char in reference.

## [0.18.0] - 2016-06-25

* [Changed] Rewrite template parser with pest.

## [0.17.0] - 2016-06-05

* [Added] JSON literals as helper param or hash, and subexpression
  return value.
* [Added] RenderError now reports template name, line and column
  number. Enabled by default. This behavior can be disabled via
  `registry.source_map_enable(false)` on production.
* [Changed] Helper API **break change**: `param(..)` and `hash(...)`
  now returns a  `ContextJson` as value which contains path as well as
  parsed Json value. No need to call `ctx.navigate(...)` any more.
* [Removed] `to_string` of `Template` and `TemplateElement` which is
  unnecessary and contains issue

## [0.16.1] - 2016-05-15

* [Removed] `num` crate dependency which is unnecessary

## [0.16.0] - 2016-03-18

* [Added] new APIs to render template string/files without
  registering to Registry
* [Added] new handlebars raw helper syntax

## [0.15.0] - 2016-03-01

* [Changed] update serde libraries to 0.7.x

## [0.14.0] - 2016-02-08

* [Added] new API: `register_template_file`

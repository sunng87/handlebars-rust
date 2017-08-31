# Change Log

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

# Change Log

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

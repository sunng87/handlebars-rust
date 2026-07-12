//! Ergonomic test helpers for handlebars.
//!
//! Available to this crate's own `#[cfg(test)]` modules unconditionally, and
//! to downstream users (for testing their own helpers/templates) when the
//! `testing` cargo feature is enabled.
//!
//! The helpers collapse the repetitive `Registry::new()` / `render_template` /
//! `.unwrap()` / `assert_eq!` boilerplate and, more importantly, produce
//! self-describing failure messages that include the template and the data.
//!
//! ```ignore
//! use handlebars::Handlebars;
//! use handlebars::testing::TestHandlebars;
//! use serde_json::json;
//!
//! // inline string template
//! let hbs = Handlebars::new();
//! hbs.assert_render_template("hello {{name}}", &json!({"name": "world"}), "hello world");
//!
//! // registered template, rendered by name
//! let mut hbs = Handlebars::new();
//! hbs.register("p", "{{this}}!");
//! hbs.assert_render("p", &json!("hi"), "hi!");
//! ```

use crate::error::RenderError;
use crate::registry::Registry;
use serde::Serialize;

/// Extension trait that adds render-test assertions to a [`Registry`].
///
/// On failure these methods panic with the template and the data so the cause
/// is obvious without re-running under a debugger.
pub trait TestHandlebars {
    /// Register a template string, panicking with a descriptive message on a
    /// compile error.
    ///
    /// Replaces the common `assert!(hbs.register_template_string(..).is_ok())`
    /// idiom, which hides compile errors.
    fn register(&mut self, name: &str, template: &str);

    /// Render an inline string template against `data` and assert the output
    /// equals `expected`.
    fn assert_render_template<T: Serialize>(&self, template: &str, data: &T, expected: &str);

    /// Render a previously-registered template by `name` and assert the output
    /// equals `expected`.
    fn assert_render<T: Serialize>(&self, name: &str, data: &T, expected: &str);

    /// Assert that rendering an inline template succeeds (the output is
    /// discarded). Useful for suites that toggle registry options such as
    /// strict mode between cases.
    fn assert_render_template_ok<T: Serialize>(&self, template: &str, data: &T);

    /// Assert that rendering an inline template fails. If `msg` is `Some(_)`,
    /// additionally require the rendered error string to contain it.
    ///
    /// The error is returned so the caller can inspect it further
    /// (e.g. `reason()`, `line_no`).
    fn assert_render_template_err<T: Serialize>(
        &self,
        template: &str,
        data: &T,
        msg: Option<&str>,
    ) -> RenderError;

    /// Assert that rendering a registered template by `name` fails. If `msg` is
    /// `Some(_)`, additionally require the error string to contain it. Returns
    /// the error for further inspection.
    fn assert_render_err<T: Serialize>(
        &self,
        name: &str,
        data: &T,
        msg: Option<&str>,
    ) -> RenderError;
}

impl<'reg> TestHandlebars for Registry<'reg> {
    fn register(&mut self, name: &str, template: &str) {
        if let Err(e) = self.register_template_string(name, template) {
            panic!("failed to register template {name:?} ({template:?}):\n{e}");
        }
    }

    fn assert_render_template<T: Serialize>(&self, template: &str, data: &T, expected: &str) {
        let actual = self.render_template(template, data).unwrap_or_else(|e| {
            panic!("render_template failed:\n  template: {template:?}\n  error: {e}")
        });
        assert_eq!(
            actual,
            expected,
            "\nrender_template mismatch\n  template: {template:?}\n  data: {}\n",
            data_to_string(data)
        );
    }

    fn assert_render<T: Serialize>(&self, name: &str, data: &T, expected: &str) {
        let actual = self
            .render(name, data)
            .unwrap_or_else(|e| panic!("render({name:?}) failed:\n  error: {e}"));
        assert_eq!(
            actual,
            expected,
            "\nrender({name:?}) mismatch\n  data: {}\n",
            data_to_string(data)
        );
    }

    fn assert_render_template_ok<T: Serialize>(&self, template: &str, data: &T) {
        if let Err(e) = self.render_template(template, data) {
            panic!(
                "expected render_template to succeed, but it errored:\n  template: {template:?}\n  error: {e}"
            );
        }
    }

    fn assert_render_template_err<T: Serialize>(
        &self,
        template: &str,
        data: &T,
        msg: Option<&str>,
    ) -> RenderError {
        match self.render_template(template, data) {
            Ok(actual) => panic!(
                "expected render_template to fail, but it produced:\n  template: {template:?}\n  output: {actual:?}"
            ),
            Err(e) => {
                if let Some(needle) = msg {
                    let s = e.to_string();
                    assert!(
                        s.contains(needle),
                        "render_template failed as expected, but the error did not contain \
                         {needle:?}:\n  {s}"
                    );
                }
                e
            }
        }
    }

    fn assert_render_err<T: Serialize>(
        &self,
        name: &str,
        data: &T,
        msg: Option<&str>,
    ) -> RenderError {
        match self.render(name, data) {
            Ok(actual) => {
                panic!("expected render({name:?}) to fail, but it produced:\n  output: {actual:?}")
            }
            Err(e) => {
                if let Some(needle) = msg {
                    let s = e.to_string();
                    assert!(
                        s.contains(needle),
                        "render({name:?}) failed as expected, but the error did not contain \
                         {needle:?}:\n  {s}"
                    );
                }
                e
            }
        }
    }
}

fn data_to_string<T: Serialize>(data: &T) -> String {
    serde_json::to_string(data).unwrap_or_else(|_| "<non-serializable>".to_owned())
}

use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

use serde::Serialize;

use crate::context::Context;
use crate::decorators::{self, DecoratorDef};
#[cfg(feature = "script_helper")]
use crate::error::ScriptError;
use crate::error::{RenderError, TemplateError, TemplateFileError, TemplateRenderError};
use crate::helpers::{self, HelperDef};
use crate::output::{Output, StringOutput, WriteOutput};
use crate::render::{RenderContext, Renderable};
use crate::support::str::{self, StringWriter};
use crate::template::Template;

#[cfg(feature = "dir_source")]
use walkdir::{DirEntry, WalkDir};

#[cfg(feature = "script_helper")]
use rhai::Engine;

#[cfg(feature = "script_helper")]
use crate::helpers::scripting::ScriptHelper;

/// This type represents an *escape fn*, that is a function whose purpose it is
/// to escape potentially problematic characters in a string.
///
/// An *escape fn* is represented as a `Box` to avoid unnecessary type
/// parameters (and because traits cannot be aliased using `type`).
pub type EscapeFn = Box<dyn Fn(&str) -> String + Send + Sync>;

/// The default *escape fn* replaces the characters `&"<>`
/// with the equivalent html / xml entities.
pub fn html_escape(data: &str) -> String {
    str::escape_html(data)
}

/// `EscapeFn` that does not change anything. Useful when using in a non-html
/// environment.
pub fn no_escape(data: &str) -> String {
    data.to_owned()
}

/// The single entry point of your Handlebars templates
///
/// It maintains compiled templates and registered helpers.
pub struct Registry<'reg> {
    templates: HashMap<String, Template>,
    helpers: HashMap<String, Box<dyn HelperDef + Send + Sync + 'reg>>,
    decorators: HashMap<String, Box<dyn DecoratorDef + Send + Sync + 'reg>>,
    escape_fn: EscapeFn,
    source_map: bool,
    strict_mode: bool,
    #[cfg(feature = "script_helper")]
    pub(crate) engine: Engine,
}

impl<'reg> Debug for Registry<'reg> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("Handlebars")
            .field("templates", &self.templates)
            .field("helpers", &self.helpers.keys())
            .field("decorators", &self.decorators.keys())
            .field("source_map", &self.source_map)
            .finish()
    }
}

impl<'reg> Default for Registry<'reg> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "dir_source")]
fn filter_file(entry: &DirEntry, suffix: &str) -> bool {
    let path = entry.path();

    // ignore hidden files, emacs buffers and files with wrong suffix
    !path.is_file()
        || path
            .file_name()
            .map(|s| {
                let ds = s.to_string_lossy();
                ds.starts_with('.') || ds.starts_with('#') || !ds.ends_with(suffix)
            })
            .unwrap_or(true)
}

#[cfg(feature = "script_helper")]
fn rhai_engine() -> Engine {
    Engine::new()
}

impl<'reg> Registry<'reg> {
    pub fn new() -> Registry<'reg> {
        let r = Registry {
            templates: HashMap::new(),
            helpers: HashMap::new(),
            decorators: HashMap::new(),
            escape_fn: Box::new(html_escape),
            source_map: true,
            strict_mode: false,
            #[cfg(feature = "script_helper")]
            engine: rhai_engine(),
        };

        r.setup_builtins()
    }

    fn setup_builtins(mut self) -> Registry<'reg> {
        self.register_helper("if", Box::new(helpers::IF_HELPER));
        self.register_helper("unless", Box::new(helpers::UNLESS_HELPER));
        self.register_helper("each", Box::new(helpers::EACH_HELPER));
        self.register_helper("with", Box::new(helpers::WITH_HELPER));
        self.register_helper("lookup", Box::new(helpers::LOOKUP_HELPER));
        self.register_helper("raw", Box::new(helpers::RAW_HELPER));
        self.register_helper("log", Box::new(helpers::LOG_HELPER));

        self.register_helper("eq", Box::new(helpers::helper_boolean::eq));
        self.register_helper("ne", Box::new(helpers::helper_boolean::ne));
        self.register_helper("gt", Box::new(helpers::helper_boolean::gt));
        self.register_helper("gte", Box::new(helpers::helper_boolean::gte));
        self.register_helper("lt", Box::new(helpers::helper_boolean::lt));
        self.register_helper("lte", Box::new(helpers::helper_boolean::lte));
        self.register_helper("and", Box::new(helpers::helper_boolean::and));
        self.register_helper("or", Box::new(helpers::helper_boolean::or));
        self.register_helper("not", Box::new(helpers::helper_boolean::not));

        self.register_decorator("inline", Box::new(decorators::INLINE_DECORATOR));
        self
    }

    /// Enable handlebars template source map
    ///
    /// Source map provides line/col reporting on error. It uses slightly
    /// more memory to maintain the data.
    ///
    /// Default is true.
    pub fn source_map_enabled(&mut self, enable: bool) {
        self.source_map = enable;
    }

    /// Enable handlebars strict mode
    ///
    /// By default, handlebars renders empty string for value that
    /// undefined or never exists. Since rust is a static type
    /// language, we offer strict mode in handlebars-rust.  In strict
    /// mode, if you were to render a value that doesn't exist, a
    /// `RenderError` will be raised.
    pub fn set_strict_mode(&mut self, enable: bool) {
        self.strict_mode = enable;
    }

    /// Return strict mode state, default is false.
    ///
    /// By default, handlebars renders empty string for value that
    /// undefined or never exists. Since rust is a static type
    /// language, we offer strict mode in handlebars-rust.  In strict
    /// mode, if you were access a value that doesn't exist, a
    /// `RenderError` will be raised.
    pub fn strict_mode(&self) -> bool {
        self.strict_mode
    }

    /// Register a `Template`
    ///
    /// This is infallible since the template has already been parsed and
    /// insert cannot fail. If there is an existing template with this name it
    /// will be overwritten.
    pub fn register_template(&mut self, name: &str, tpl: Template) {
        self.templates.insert(name.to_string(), tpl);
    }

    /// Register a template string
    ///
    /// Returns `TemplateError` if there is syntax error on parsing the template.
    pub fn register_template_string<S>(
        &mut self,
        name: &str,
        tpl_str: S,
    ) -> Result<(), TemplateError>
    where
        S: AsRef<str>,
    {
        let template = Template::compile_with_name(tpl_str, name.to_owned(), self.source_map)?;
        self.register_template(name, template);
        Ok(())
    }

    /// Register a partial string
    ///
    /// A named partial will be added to the registry. It will overwrite template with
    /// same name. Currently a registered partial is just identical to a template.
    pub fn register_partial<S>(&mut self, name: &str, partial_str: S) -> Result<(), TemplateError>
    where
        S: AsRef<str>,
    {
        self.register_template_string(name, partial_str)
    }

    /// Register a template from a path
    pub fn register_template_file<P>(
        &mut self,
        name: &str,
        tpl_path: P,
    ) -> Result<(), TemplateFileError>
    where
        P: AsRef<Path>,
    {
        let mut reader = BufReader::new(
            File::open(tpl_path).map_err(|e| TemplateFileError::IOError(e, name.to_owned()))?,
        );
        self.register_template_source(name, &mut reader)
    }

    /// Register templates from a directory
    ///
    /// * `tpl_extension`: the template file extension
    /// * `dir_path`: the path of directory
    ///
    /// Hidden files and tempfile (starts with `#`) will be ignored. All registered
    /// will use their relative name as template name. For example, when `dir_path` is
    /// `templates/` and `tpl_extension` is `.hbs`, the file
    /// `templates/some/path/file.hbs` will be registered as `some/path/file`.
    ///
    /// This method is not available by default.
    /// You will need to enable the `dir_source` feature to use it.
    #[cfg(feature = "dir_source")]
    pub fn register_templates_directory<P>(
        &mut self,
        tpl_extension: &'static str,
        dir_path: P,
    ) -> Result<(), TemplateFileError>
    where
        P: AsRef<Path>,
    {
        let dir_path = dir_path.as_ref();

        let prefix_len = if dir_path.to_string_lossy().ends_with('/') {
            dir_path.to_string_lossy().len()
        } else {
            dir_path.to_string_lossy().len() + 1
        };

        let walker = WalkDir::new(dir_path);
        let dir_iter = walker
            .min_depth(1)
            .into_iter()
            .filter(|e| e.is_ok() && !filter_file(e.as_ref().unwrap(), tpl_extension));

        for entry in dir_iter {
            let entry = entry?;

            let tpl_path = entry.path();
            let tpl_file_path = entry.path().to_string_lossy();

            let tpl_name = &tpl_file_path[prefix_len..tpl_file_path.len() - tpl_extension.len()];
            let tpl_canonical_name = tpl_name.replace("\\", "/");
            self.register_template_file(&tpl_canonical_name, &tpl_path)?;
        }

        Ok(())
    }

    /// Register a template from `std::io::Read` source
    pub fn register_template_source<R>(
        &mut self,
        name: &str,
        tpl_source: &mut R,
    ) -> Result<(), TemplateFileError>
    where
        R: Read,
    {
        let mut buf = String::new();
        tpl_source
            .read_to_string(&mut buf)
            .map_err(|e| TemplateFileError::IOError(e, name.to_owned()))?;
        self.register_template_string(name, buf)?;
        Ok(())
    }

    /// Remove a template from the registry
    pub fn unregister_template(&mut self, name: &str) {
        self.templates.remove(name);
    }

    /// Register a helper
    pub fn register_helper(
        &mut self,
        name: &str,
        def: Box<dyn HelperDef + Send + Sync + 'reg>,
    ) -> Option<Box<dyn HelperDef + Send + Sync + 'reg>> {
        self.helpers.insert(name.to_string(), def)
    }

    /// Register a [rhai](https://docs.rs/rhai/) script as handlebars helper
    ///
    /// Currently only simple helpers are supported. You can do computation or
    /// string formatting with rhai script.
    ///
    /// Helper parameters and hash are available in rhai script as array `params`
    /// and map `hash`. Example script:
    ///
    /// ```handlebars
    /// {{percent 0.34 label="%"}}
    /// ```
    ///
    /// ```rhai
    /// // percent.rhai
    /// let value = params[0];
    /// let label = hash["label"];
    ///
    /// (value * 100).to_string() + label
    /// ```
    ///
    ///
    #[cfg(feature = "script_helper")]
    pub fn register_script_helper(
        &mut self,
        name: &str,
        script: String,
    ) -> Result<Option<Box<dyn HelperDef + Send + Sync + 'reg>>, ScriptError> {
        let compiled = self.engine.compile(&script)?;
        let script_helper = ScriptHelper { script: compiled };
        Ok(self
            .helpers
            .insert(name.to_string(), Box::new(script_helper)))
    }

    /// Register a [rhai](https://docs.rs/rhai/) script from file
    #[cfg(feature = "script_helper")]
    pub fn register_script_helper_file<P>(
        &mut self,
        name: &str,
        script_path: P,
    ) -> Result<Option<Box<dyn HelperDef + Send + Sync + 'reg>>, ScriptError>
    where
        P: AsRef<Path>,
    {
        let mut script = String::new();
        {
            let mut file = File::open(script_path)?;
            file.read_to_string(&mut script)?;
        }

        self.register_script_helper(name, script)
    }

    /// Register a decorator
    pub fn register_decorator(
        &mut self,
        name: &str,
        def: Box<dyn DecoratorDef + Send + Sync + 'reg>,
    ) -> Option<Box<dyn DecoratorDef + Send + Sync + 'reg>> {
        self.decorators.insert(name.to_string(), def)
    }

    /// Register a new *escape fn* to be used from now on by this registry.
    pub fn register_escape_fn<F: 'static + Fn(&str) -> String + Send + Sync>(
        &mut self,
        escape_fn: F,
    ) {
        self.escape_fn = Box::new(escape_fn);
    }

    /// Restore the default *escape fn*.
    pub fn unregister_escape_fn(&mut self) {
        self.escape_fn = Box::new(html_escape);
    }

    /// Get a reference to the current *escape fn*.
    pub fn get_escape_fn(&self) -> &dyn Fn(&str) -> String {
        &*self.escape_fn
    }

    /// Return `true` if a template is registered for the given name
    pub fn has_template(&self, name: &str) -> bool {
        self.get_template(name).is_some()
    }

    /// Return a registered template,
    pub fn get_template(&self, name: &str) -> Option<&Template> {
        self.templates.get(name)
    }

    /// Return a registered helper
    pub fn get_helper(&self, name: &str) -> Option<&(dyn HelperDef + Send + Sync + 'reg)> {
        self.helpers.get(name).map(|v| v.as_ref())
    }

    #[inline]
    pub(crate) fn has_helper(&self, name: &str) -> bool {
        self.helpers.contains_key(name)
    }

    /// Return a registered decorator
    pub fn get_decorator(&self, name: &str) -> Option<&(dyn DecoratorDef + Send + Sync + 'reg)> {
        self.decorators.get(name).map(|v| v.as_ref())
    }

    /// Return all templates registered
    pub fn get_templates(&self) -> &HashMap<String, Template> {
        &self.templates
    }

    /// Unregister all templates
    pub fn clear_templates(&mut self) {
        self.templates.clear();
    }

    fn render_to_output<O>(
        &self,
        name: &str,
        ctx: &Context,
        output: &mut O,
    ) -> Result<(), RenderError>
    where
        O: Output,
    {
        self.get_template(name)
            .ok_or_else(|| RenderError::new(format!("Template not found: {}", name)))
            .and_then(|t| {
                let mut render_context = RenderContext::new(t.name.as_ref());
                t.render(self, &ctx, &mut render_context, output)
            })
    }

    /// Render a registered template with some data into a string
    ///
    /// * `name` is the template name you registered previously
    /// * `data` is the data that implements `serde::Serialize`
    ///
    /// Returns rendered string or a struct with error information
    pub fn render<T>(&self, name: &str, data: &T) -> Result<String, RenderError>
    where
        T: Serialize,
    {
        let mut output = StringOutput::new();
        let ctx = Context::wraps(&data)?;
        self.render_to_output(name, &ctx, &mut output)?;
        output.into_string().map_err(RenderError::from)
    }

    /// Render a registered template with reused context
    pub fn render_with_context(&self, name: &str, ctx: &Context) -> Result<String, RenderError> {
        let mut output = StringOutput::new();
        self.render_to_output(name, ctx, &mut output)?;
        output.into_string().map_err(RenderError::from)
    }

    /// Render a registered template and write some data to the `std::io::Write`
    pub fn render_to_write<T, W>(&self, name: &str, data: &T, writer: W) -> Result<(), RenderError>
    where
        T: Serialize,
        W: Write,
    {
        let mut output = WriteOutput::new(writer);
        let ctx = Context::wraps(data)?;
        self.render_to_output(name, &ctx, &mut output)
    }

    /// Render a template string using current registry without registering it
    pub fn render_template<T>(
        &self,
        template_string: &str,
        data: &T,
    ) -> Result<String, TemplateRenderError>
    where
        T: Serialize,
    {
        let mut writer = StringWriter::new();
        self.render_template_to_write(template_string, data, &mut writer)?;
        Ok(writer.into_string())
    }

    /// Render a template string using reused context data
    pub fn render_template_with_context(
        &self,
        template_string: &str,
        ctx: &Context,
    ) -> Result<String, TemplateRenderError> {
        let tpl = Template::compile2(template_string, self.source_map)?;

        let mut out = StringOutput::new();
        {
            let mut render_context = RenderContext::new(None);
            tpl.render(self, &ctx, &mut render_context, &mut out)?;
        }

        out.into_string()
            .map_err(|e| TemplateRenderError::from(RenderError::from(e)))
    }

    /// Render a template string using current registry without registering it
    pub fn render_template_to_write<T, W>(
        &self,
        template_string: &str,
        data: &T,
        writer: W,
    ) -> Result<(), TemplateRenderError>
    where
        T: Serialize,
        W: Write,
    {
        let tpl = Template::compile2(template_string, self.source_map)?;
        let ctx = Context::wraps(data)?;
        let mut render_context = RenderContext::new(None);
        let mut out = WriteOutput::new(writer);
        tpl.render(self, &ctx, &mut render_context, &mut out)
            .map_err(TemplateRenderError::from)
    }

    /// Render a template source using current registry without registering it
    pub fn render_template_source_to_write<T, R, W>(
        &self,
        template_source: &mut R,
        data: &T,
        writer: W,
    ) -> Result<(), TemplateRenderError>
    where
        T: Serialize,
        W: Write,
        R: Read,
    {
        let mut tpl_str = String::new();
        template_source
            .read_to_string(&mut tpl_str)
            .map_err(|e| TemplateRenderError::IOError(e, "Unnamed template source".to_owned()))?;
        self.render_template_to_write(&tpl_str, data, writer)
    }
}

#[cfg(test)]
mod test {
    use crate::context::Context;
    use crate::error::RenderError;
    use crate::helpers::HelperDef;
    use crate::output::Output;
    use crate::registry::Registry;
    use crate::render::{Helper, RenderContext, Renderable};
    use crate::support::str::StringWriter;
    use crate::template::Template;
    #[cfg(feature = "dir_source")]
    use std::fs::{DirBuilder, File};
    #[cfg(feature = "dir_source")]
    use std::io::Write;
    #[cfg(feature = "dir_source")]
    use tempfile::tempdir;

    #[derive(Clone, Copy)]
    struct DummyHelper;

    impl HelperDef for DummyHelper {
        fn call<'reg: 'rc, 'rc>(
            &self,
            h: &Helper<'reg, 'rc>,
            r: &'reg Registry<'reg>,
            ctx: &'rc Context,
            rc: &mut RenderContext<'reg, 'rc>,
            out: &mut dyn Output,
        ) -> Result<(), RenderError> {
            h.template().unwrap().render(r, ctx, rc, out)
        }
    }

    static DUMMY_HELPER: DummyHelper = DummyHelper;

    #[test]
    fn test_registry_operations() {
        let mut r = Registry::new();

        assert!(r.register_template_string("index", "<h1></h1>").is_ok());

        let tpl = Template::compile("<h2></h2>").unwrap();
        r.register_template("index2", tpl);

        assert_eq!(r.templates.len(), 2);

        r.unregister_template("index");
        assert_eq!(r.templates.len(), 1);

        r.clear_templates();
        assert_eq!(r.templates.len(), 0);

        r.register_helper("dummy", Box::new(DUMMY_HELPER));

        // built-in helpers plus 1
        let num_helpers = 7;
        let num_boolean_helpers = 9; // stuff like gt and lte
        let num_custom_helpers = 1; // dummy from above
        assert_eq!(
            r.helpers.len(),
            num_helpers + num_boolean_helpers + num_custom_helpers
        );
    }

    #[test]
    #[cfg(feature = "dir_source")]
    fn test_register_templates_directory() {
        let mut r = Registry::new();
        {
            let dir = tempdir().unwrap();

            assert_eq!(r.templates.len(), 0);

            let file1_path = dir.path().join("t1.hbs");
            let mut file1: File = File::create(&file1_path).unwrap();
            writeln!(file1, "<h1>Hello {{world}}!</h1>").unwrap();

            let file2_path = dir.path().join("t2.hbs");
            let mut file2: File = File::create(&file2_path).unwrap();
            writeln!(file2, "<h1>Hola {{world}}!</h1>").unwrap();

            let file3_path = dir.path().join("t3.hbs");
            let mut file3: File = File::create(&file3_path).unwrap();
            writeln!(file3, "<h1>Hallo {{world}}!</h1>").unwrap();

            let file4_path = dir.path().join(".t4.hbs");
            let mut file4: File = File::create(&file4_path).unwrap();
            writeln!(file4, "<h1>Hallo {{world}}!</h1>").unwrap();

            r.register_templates_directory(".hbs", dir.path()).unwrap();

            assert_eq!(r.templates.len(), 3);
            assert_eq!(r.templates.contains_key("t1"), true);
            assert_eq!(r.templates.contains_key("t2"), true);
            assert_eq!(r.templates.contains_key("t3"), true);
            assert_eq!(r.templates.contains_key("t4"), false);

            drop(file1);
            drop(file2);
            drop(file3);

            dir.close().unwrap();
        }

        {
            let dir = tempdir().unwrap();

            let file1_path = dir.path().join("t4.hbs");
            let mut file1: File = File::create(&file1_path).unwrap();
            writeln!(file1, "<h1>Hello {{world}}!</h1>").unwrap();

            let file2_path = dir.path().join("t5.erb");
            let mut file2: File = File::create(&file2_path).unwrap();
            writeln!(file2, "<h1>Hello {{% world %}}!</h1>").unwrap();

            let file3_path = dir.path().join("t6.html");
            let mut file3: File = File::create(&file3_path).unwrap();
            writeln!(file3, "<h1>Hello world!</h1>").unwrap();

            r.register_templates_directory(".hbs", dir.path()).unwrap();

            assert_eq!(r.templates.len(), 4);
            assert_eq!(r.templates.contains_key("t4"), true);

            drop(file1);
            drop(file2);
            drop(file3);

            dir.close().unwrap();
        }

        {
            let dir = tempdir().unwrap();

            let _ = DirBuilder::new().create(dir.path().join("french")).unwrap();
            let _ = DirBuilder::new()
                .create(dir.path().join("portugese"))
                .unwrap();
            let _ = DirBuilder::new()
                .create(dir.path().join("italian"))
                .unwrap();

            let file1_path = dir.path().join("french/t7.hbs");
            let mut file1: File = File::create(&file1_path).unwrap();
            writeln!(file1, "<h1>Bonjour {{world}}!</h1>").unwrap();

            let file2_path = dir.path().join("portugese/t8.hbs");
            let mut file2: File = File::create(&file2_path).unwrap();
            writeln!(file2, "<h1>Ola {{world}}!</h1>").unwrap();

            let file3_path = dir.path().join("italian/t9.hbs");
            let mut file3: File = File::create(&file3_path).unwrap();
            writeln!(file3, "<h1>Ciao {{world}}!</h1>").unwrap();

            r.register_templates_directory(".hbs", dir.path()).unwrap();

            assert_eq!(r.templates.len(), 7);
            assert_eq!(r.templates.contains_key("french/t7"), true);
            assert_eq!(r.templates.contains_key("portugese/t8"), true);
            assert_eq!(r.templates.contains_key("italian/t9"), true);

            drop(file1);
            drop(file2);
            drop(file3);

            dir.close().unwrap();
        }
    }

    #[test]
    fn test_render_to_write() {
        let mut r = Registry::new();

        assert!(r.register_template_string("index", "<h1></h1>").is_ok());

        let mut sw = StringWriter::new();
        {
            r.render_to_write("index", &(), &mut sw).ok().unwrap();
        }

        assert_eq!("<h1></h1>".to_string(), sw.into_string());
    }

    #[test]
    fn test_escape_fn() {
        let mut r = Registry::new();

        let input = String::from("\"<>&");

        r.register_template_string("test", String::from("{{this}}"))
            .unwrap();

        assert_eq!("&quot;&lt;&gt;&amp;", r.render("test", &input).unwrap());

        r.register_escape_fn(|s| s.into());

        assert_eq!("\"<>&", r.render("test", &input).unwrap());

        r.unregister_escape_fn();

        assert_eq!("&quot;&lt;&gt;&amp;", r.render("test", &input).unwrap());
    }

    #[test]
    fn test_escape() {
        let r = Registry::new();
        let data = json!({"hello": "world"});

        assert_eq!(
            "{{hello}}",
            r.render_template(r"\{{hello}}", &data).unwrap()
        );

        assert_eq!(
            " {{hello}}",
            r.render_template(r" \{{hello}}", &data).unwrap()
        );

        assert_eq!(r"\world", r.render_template(r"\\{{hello}}", &data).unwrap());
    }

    #[test]
    fn test_strict_mode() {
        let mut r = Registry::new();
        assert!(!r.strict_mode());

        r.set_strict_mode(true);
        assert!(r.strict_mode());

        let data = json!({
            "the_only_key": "the_only_value"
        });

        assert!(r
            .render_template("accessing the_only_key {{the_only_key}}", &data)
            .is_ok());
        assert!(r
            .render_template("accessing non-exists key {{the_key_never_exists}}", &data)
            .is_err());

        let render_error = r
            .render_template("accessing non-exists key {{the_key_never_exists}}", &data)
            .unwrap_err();
        assert_eq!(
            render_error.as_render_error().unwrap().column_no.unwrap(),
            26
        );

        let data2 = json!([1, 2, 3]);
        assert!(r
            .render_template("accessing valid array index {{this.[2]}}", &data2)
            .is_ok());
        assert!(r
            .render_template("accessing invalid array index {{this.[3]}}", &data2)
            .is_err());
        let render_error2 = r
            .render_template("accessing invalid array index {{this.[3]}}", &data2)
            .unwrap_err();
        assert_eq!(
            render_error2.as_render_error().unwrap().column_no.unwrap(),
            31
        );
    }

    use crate::json::value::ScopedJson;
    struct GenMissingHelper;
    impl HelperDef for GenMissingHelper {
        fn call_inner<'reg: 'rc, 'rc>(
            &self,
            _: &Helper<'reg, 'rc>,
            _: &'reg Registry<'reg>,
            _: &'rc Context,
            _: &mut RenderContext<'reg, 'rc>,
        ) -> Result<Option<ScopedJson<'reg, 'rc>>, RenderError> {
            Ok(Some(ScopedJson::Missing))
        }
    }

    #[test]
    fn test_strict_mode_in_helper() {
        let mut r = Registry::new();
        r.set_strict_mode(true);

        r.register_helper(
            "check_missing",
            Box::new(
                |h: &Helper<'_, '_>,
                 _: &Registry<'_>,
                 _: &Context,
                 _: &mut RenderContext<'_, '_>,
                 _: &mut dyn Output|
                 -> Result<(), RenderError> {
                    let value = h.param(0).unwrap();
                    assert!(value.is_value_missing());
                    Ok(())
                },
            ),
        );

        r.register_helper("generate_missing_value", Box::new(GenMissingHelper));

        let data = json!({
            "the_key_we_have": "the_value_we_have"
        });
        assert!(r
            .render_template("accessing non-exists key {{the_key_we_dont_have}}", &data)
            .is_err());
        assert!(r
            .render_template(
                "accessing non-exists key from helper {{check_missing the_key_we_dont_have}}",
                &data
            )
            .is_ok());
        assert!(r
            .render_template(
                "accessing helper that generates missing value {{generate_missing_value}}",
                &data
            )
            .is_err());
    }

    #[test]
    fn test_html_expression() {
        let reg = Registry::new();
        assert_eq!(
            reg.render_template("{{{ a }}}", &json!({"a": "<b>bold</b>"}))
                .unwrap(),
            "<b>bold</b>"
        );
        assert_eq!(
            reg.render_template("{{ &a }}", &json!({"a": "<b>bold</b>"}))
                .unwrap(),
            "<b>bold</b>"
        );
    }

    #[test]
    fn test_render_context() {
        let mut reg = Registry::new();

        let data = json!([0, 1, 2, 3]);

        assert_eq!(
            "0123",
            reg.render_template_with_context(
                "{{#each this}}{{this}}{{/each}}",
                &Context::wraps(&data).unwrap()
            )
            .unwrap()
        );

        reg.register_template_string("t0", "{{#each this}}{{this}}{{/each}}")
            .unwrap();
        assert_eq!(
            "0123",
            reg.render_with_context("t0", &Context::wraps(&data).unwrap())
                .unwrap()
        );
    }
}

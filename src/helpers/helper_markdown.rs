use helpers::{HelperDef};
use registry::{Registry};
use context::{Context, JsonRender};
use render::{Renderable, RenderContext, RenderError, render_error, Helper};
extern crate pulldown_cmark;

use self::pulldown_cmark::Parser;
use self::pulldown_cmark::{Options, OPTION_ENABLE_TABLES, OPTION_ENABLE_FOOTNOTES};
use self::pulldown_cmark::html;


#[derive(Clone, Copy)]
pub struct MarkdownHelper;



pub fn render_html(text: String) -> String {
    let mut opts = Options::empty();
    opts.insert(OPTION_ENABLE_TABLES);
    opts.insert(OPTION_ENABLE_FOOTNOTES);
    let mut s = String::with_capacity(text.len() * 3 / 2);
    let p = Parser::new_ext(&*text, opts);
    html::push_html(&mut s, p);
    s
}


impl HelperDef for MarkdownHelper {
    fn call(&self, c: &Context, h: &Helper, _: &Registry, rc: &mut RenderContext) -> Result<(), RenderError> {
        let markdown_text_var = try!(h.param(0).ok_or_else(|| render_error("Param not found for helper \"markdown\"")));
        let markdown_text = c.navigate(rc.get_path(), &markdown_text_var).render(); 
        let html_string = render_html(markdown_text);
        try!(rc.writer.write(html_string.into_bytes().as_ref()));
        Ok(())
    }
}

pub static MARKDOWN_HELPER: MarkdownHelper = MarkdownHelper;

#[cfg(test)]
mod test {
    use template::{Template};
    use registry::{Registry};

    use std::collections::BTreeMap;

    #[test]
    fn test_markdown() {
        let t0 = Template::compile("{{markdown x}}".to_string()).ok().unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", t0);

        let mut m :BTreeMap<String, String> = BTreeMap::new();
        m.insert("x".into(), "# wow\n\n## second wow".into());

        let r0 = handlebars.render("t0", &m);
        assert_eq!(r0.ok().unwrap(), "<h1>wow</h1>\n<h2>second wow</h2>\n".to_string());
    }
}

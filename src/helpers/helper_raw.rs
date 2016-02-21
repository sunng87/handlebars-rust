#[cfg(test)]
mod test {
    use template::{Template};
    use registry::{Registry};

    #[test]
    fn test_raw_helper () {
        let t = Template::compile("a{{#raw}}{{content}}{{else}}hello{{/raw}}".to_string()).ok().unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", t);

        let r = handlebars.render("t0", &());
        assert_eq!(r.ok().unwrap(), "a{{content}}{{else}}hello");
    }
    
    static TEMPLATE: &'static str = r#"a
{{#raw}}
    {{content}}
        Hi!
    {{else}}
        hello
{{/raw}}
"#;

    static RESULT: &'static str = r#"a

    {{content}}
        Hi!
    {{else}}
        hello

"#;
    
    #[test]
    fn test_raw_helper_2 () {
        let t = Template::compile(TEMPLATE.to_string()).ok().unwrap();

        let mut handlebars = Registry::new();
        handlebars.register_template("t1", t);

        let r = handlebars.render("t1", &());
        assert_eq!(r.ok().unwrap(), RESULT);
    }
   
    #[test]
    fn test_raw_helper_3 () {
        let t = match Template::compile("a{{#raw}}{{{{content}}{{else}}hello{{/raw}}".to_string()) {
            Ok(t) => t,
            Err(e) => {
                panic!("{}", e);
            }
        };

        let mut handlebars = Registry::new();
        handlebars.register_template("t0", t);

        let r = handlebars.render("t0", &());
        assert_eq!(r.ok().unwrap(), "a{{{{content}}{{else}}hello");
    }
}

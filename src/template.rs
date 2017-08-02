use std::slice::Iter;
use std::iter::Peekable;
use std::convert::From;
use std::collections::{BTreeMap, VecDeque};
use pest::prelude::*;

use serde_json::value::Value as Json;
use std::str::FromStr;

use grammar::{Rdp, Rule};

use error::{TemplateError, TemplateErrorReason};

use self::TemplateElement::*;

#[derive(PartialEq, Clone, Debug)]
pub struct TemplateMapping(pub usize, pub usize);

/// A handlebars template
#[derive(PartialEq, Clone, Debug)]
pub struct Template {
    pub name: Option<String>,
    pub elements: Vec<TemplateElement>,
    pub mapping: Option<Vec<TemplateMapping>>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Subexpression {
    pub name: String,
    pub params: Vec<Parameter>,
    pub hash: BTreeMap<String, Parameter>,
}

impl Subexpression {
    pub fn is_helper(&self) -> bool {
        !(self.params.is_empty() && self.hash.is_empty())
    }

    pub fn as_template(&self) -> Template {
        let mut t = Template::new(false);
        let el = if self.is_helper() {
            HelperExpression(HelperTemplate::from(self))
        } else {
            Expression(Parameter::Name(self.name.clone()))
        };
        t.elements.push(el);
        t
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum BlockParam {
    Single(Parameter),
    Pair((Parameter, Parameter)),
}

#[derive(PartialEq, Clone, Debug)]
pub struct ExpressionSpec {
    pub name: Parameter,
    pub params: Vec<Parameter>,
    pub hash: BTreeMap<String, Parameter>,
    pub block_param: Option<BlockParam>,
    pub omit_pre_ws: bool,
    pub omit_pro_ws: bool,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Parameter {
    Name(String),
    Literal(Json),
    Subexpression(Subexpression),
}

#[derive(PartialEq, Clone, Debug)]
pub struct HelperTemplate {
    pub name: String,
    pub params: Vec<Parameter>,
    pub hash: BTreeMap<String, Parameter>,
    pub block_param: Option<BlockParam>,
    pub template: Option<Template>,
    pub inverse: Option<Template>,
    pub block: bool,
}

impl<'a> From<&'a Subexpression> for HelperTemplate {
    fn from(s: &Subexpression) -> HelperTemplate {
        HelperTemplate {
            name: s.name.clone(),
            params: s.params.clone(),
            hash: s.hash.clone(),
            block_param: None,
            template: None,
            inverse: None,
            block: false,
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Directive {
    pub name: Parameter,
    pub params: Vec<Parameter>,
    pub hash: BTreeMap<String, Parameter>,
    pub template: Option<Template>,
}

impl Parameter {
    pub fn as_name(self) -> Option<String> {
        if let Parameter::Name(n) = self {
            Some(n)
        } else {
            None
        }
    }

    pub fn parse(s: &str) -> Result<Parameter, TemplateError> {
        let mut parser = Rdp::new(StringInput::new(s));
        if !parser.parameter() {
            return Err(TemplateError::of(
                TemplateErrorReason::InvalidParam(s.to_owned()),
            ));
        }

        let mut it = parser.queue().iter().peekable();
        Template::parse_param(s, &mut it, s.len() - 1)
    }
}

impl Template {
    pub fn new(mapping: bool) -> Template {
        Template {
            elements: Vec::new(),
            name: None,
            mapping: if mapping { Some(Vec::new()) } else { None },
        }
    }

    fn unescape_tags(txt: &str) -> String {
        txt.replace(r"\\{{", "{{")
    }

    fn push_element(&mut self, e: TemplateElement, line: usize, col: usize) {
        self.elements.push(e);
        if let Some(ref mut maps) = self.mapping {
            maps.push(TemplateMapping(line, col));
        }
    }

    pub fn compile<S: AsRef<str>>(source: S) -> Result<Template, TemplateError> {
        Template::compile2(source, false)
    }

    #[inline]
    fn parse_subexpression<'a>(
        source: &'a str,
        it: &mut Peekable<Iter<Token<Rule>>>,
        limit: usize,
    ) -> Result<Parameter, TemplateError> {
        let espec = try!(Template::parse_expression(source, it.by_ref(), limit));
        if let Parameter::Name(name) = espec.name {
            Ok(Parameter::Subexpression(Subexpression {
                name: name,
                params: espec.params,
                hash: espec.hash,
            }))
        } else {
            // line/col no
            Err(TemplateError::of(TemplateErrorReason::NestedSubexpression))
        }
    }

    #[inline]
    fn parse_name<'a>(
        source: &'a str,
        it: &mut Peekable<Iter<Token<Rule>>>,
        _: usize,
    ) -> Result<Parameter, TemplateError> {
        let name_node = it.next().unwrap();
        match name_node.rule {
            Rule::identifier |
            Rule::reference |
            Rule::invert_tag_item => {
                Ok(Parameter::Name(
                    source[name_node.start..name_node.end].to_owned(),
                ))
            }
            Rule::subexpression => {
                Template::parse_subexpression(source, it.by_ref(), name_node.end)
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    fn parse_param<'a>(
        source: &'a str,
        it: &mut Peekable<Iter<Token<Rule>>>,
        _: usize,
    ) -> Result<Parameter, TemplateError> {
        let mut param = it.next().unwrap();
        if param.rule == Rule::param {
            param = it.next().unwrap();
        }
        let result = match param.rule {
            Rule::reference => Parameter::Name(source[param.start..param.end].to_owned()),
            Rule::literal => {
                let s = &source[param.start..param.end];
                if let Ok(json) = Json::from_str(s) {
                    Parameter::Literal(json)
                } else {
                    Parameter::Name(s.to_owned())
                }
            }
            Rule::subexpression => {
                try!(Template::parse_subexpression(
                    source,
                    it.by_ref(),
                    param.end,
                ))
            }
            _ => unreachable!(),
        };

        loop {
            if let Some(ref n) = it.peek() {
                if n.end > param.end {
                    break;
                }
            } else {
                break;
            }

            it.next();
        }

        Ok(result)
    }

    #[inline]
    fn parse_hash<'a>(
        source: &'a str,
        it: &mut Peekable<Iter<Token<Rule>>>,
        limit: usize,
    ) -> Result<(String, Parameter), TemplateError> {
        let name = it.next().unwrap();
        // identifier
        let key = source[name.start..name.end].to_owned();

        let value = try!(Template::parse_param(source, it.by_ref(), limit));
        Ok((key, value))
    }

    #[inline]
    fn parse_block_param<'a>(
        source: &'a str,
        it: &mut Peekable<Iter<Token<Rule>>>,
        limit: usize,
    ) -> Result<BlockParam, TemplateError> {
        let p1_name = it.next().unwrap();
        // identifier
        let p1 = source[p1_name.start..p1_name.end].to_owned();

        let p2 = it.peek().and_then(|p2_name| if p2_name.end <= limit {
            Some(source[p2_name.start..p2_name.end].to_owned())
        } else {
            None
        });

        if p2.is_some() {
            it.next();
            Ok(BlockParam::Pair(
                (Parameter::Name(p1), Parameter::Name(p2.unwrap())),
            ))
        } else {
            Ok(BlockParam::Single(Parameter::Name(p1)))
        }
    }

    #[inline]
    fn parse_expression<'a>(
        source: &'a str,
        it: &mut Peekable<Iter<Token<Rule>>>,
        limit: usize,
    ) -> Result<ExpressionSpec, TemplateError> {
        let mut params: Vec<Parameter> = Vec::new();
        let mut hashes: BTreeMap<String, Parameter> = BTreeMap::new();
        let mut omit_pre_ws = false;
        let mut omit_pro_ws = false;
        let mut block_param = None;

        if it.peek().unwrap().rule == Rule::pre_whitespace_omitter {
            omit_pre_ws = true;
            it.next();
        }

        let name = try!(Template::parse_name(source, it.by_ref(), limit));

        loop {
            let rule;
            let end;
            if let Some(ref token) = it.peek() {
                if token.end < limit {
                    rule = token.rule;
                    end = token.end;
                } else {
                    break;
                }
            } else {
                break;
            }

            it.next();

            match rule {
                Rule::param => {
                    params.push(try!(Template::parse_param(source, it.by_ref(), end)));
                }
                Rule::hash => {
                    let (key, value) = try!(Template::parse_hash(source, it.by_ref(), end));
                    hashes.insert(key, value);
                }
                Rule::block_param => {
                    block_param = Some(try!(Template::parse_block_param(source, it.by_ref(), end)));
                }
                Rule::pro_whitespace_omitter => {
                    omit_pro_ws = true;
                }
                _ => {}
            }
        }
        Ok(ExpressionSpec {
            name: name,
            params: params,
            hash: hashes,
            block_param: block_param,
            omit_pre_ws: omit_pre_ws,
            omit_pro_ws: omit_pro_ws,
        })
    }

    #[inline]
    fn remove_previous_whitespace(template_stack: &mut VecDeque<Template>) {
        let mut t = template_stack.front_mut().unwrap();
        if let Some(el) = t.elements.pop() {
            if let RawString(ref text) = el {
                t.elements.push(RawString(text.trim_right().to_owned()));
            } else {
                t.elements.push(el);
            }
        }
    }

    pub fn compile2<S: AsRef<str>>(source: S, mapping: bool) -> Result<Template, TemplateError> {
        let source = source.as_ref();
        let mut helper_stack: VecDeque<HelperTemplate> = VecDeque::new();
        let mut directive_stack: VecDeque<Directive> = VecDeque::new();
        let mut template_stack: VecDeque<Template> = VecDeque::new();

        let mut omit_pro_ws = false;

        let input = StringInput::new(source);
        let mut parser = Rdp::new(input);

        if !parser.handlebars() {
            let (_, pos) = parser.expected();
            let (line_no, col_no) = parser.input().line_col(pos);
            return Err(TemplateError::of(TemplateErrorReason::InvalidSyntax).at(
                line_no,
                col_no,
            ));
        }

        let mut it = parser.queue().iter().peekable();
        let mut prev_end = 0;
        loop {
            if let Some(ref token) = it.next() {

                if token.rule != Rule::template {
                    if token.start != prev_end && !omit_pro_ws && token.rule != Rule::raw_text &&
                        token.rule != Rule::raw_block_text
                    {
                        let (line_no, col_no) = parser.input().line_col(prev_end);
                        if token.rule == Rule::raw_block_end {
                            let text = &source[prev_end..token.start];
                            let mut t = Template::new(mapping);
                            t.push_element(
                                RawString(Template::unescape_tags(text)),
                                line_no,
                                col_no,
                            );
                            template_stack.push_front(t);
                        } else {
                            let text = &source[prev_end..token.start];
                            let mut t = template_stack.front_mut().unwrap();
                            t.push_element(
                                RawString(Template::unescape_tags(text)),
                                line_no,
                                col_no,
                            );
                        }
                    }
                }

                let (line_no, col_no) = parser.input().line_col(token.start);
                match token.rule {
                    Rule::template => {
                        template_stack.push_front(Template::new(mapping));
                    }
                    Rule::raw_text => {
                        let mut text = &source[prev_end..token.end];
                        if omit_pro_ws {
                            text = text.trim_left();
                        }
                        let mut t = template_stack.front_mut().unwrap();
                        t.push_element(RawString(Template::unescape_tags(text)), line_no, col_no);
                    }
                    Rule::helper_block_start |
                    Rule::raw_block_start |
                    Rule::directive_block_start |
                    Rule::partial_block_start => {
                        let exp = try!(Template::parse_expression(source, it.by_ref(), token.end));

                        match token.rule {
                            Rule::helper_block_start |
                            Rule::raw_block_start => {
                                let helper_template = HelperTemplate {
                                    name: exp.name.as_name().unwrap(),
                                    params: exp.params,
                                    hash: exp.hash,
                                    block_param: exp.block_param,
                                    block: true,
                                    template: None,
                                    inverse: None,
                                };
                                helper_stack.push_front(helper_template);
                            }
                            Rule::directive_block_start |
                            Rule::partial_block_start => {
                                let directive = Directive {
                                    name: exp.name,
                                    params: exp.params,
                                    hash: exp.hash,
                                    template: None,
                                };
                                directive_stack.push_front(directive);
                            }
                            _ => unreachable!(),
                        }

                        if exp.omit_pre_ws {
                            Template::remove_previous_whitespace(&mut template_stack);
                        }
                        omit_pro_ws = exp.omit_pro_ws;

                        let mut t = template_stack.front_mut().unwrap();
                        if let Some(ref mut maps) = t.mapping {
                            maps.push(TemplateMapping(line_no, col_no));
                        }
                    }
                    Rule::invert_tag => {
                        // hack: invert_tag structure is similar to ExpressionSpec, so I
                        // use it here to represent the data
                        let exp = try!(Template::parse_expression(source, it.by_ref(), token.end));

                        if exp.omit_pre_ws {
                            Template::remove_previous_whitespace(&mut template_stack);
                        }
                        omit_pro_ws = exp.omit_pro_ws;

                        let t = template_stack.pop_front().unwrap();
                        let mut h = helper_stack.front_mut().unwrap();
                        h.template = Some(t);
                    }
                    Rule::raw_block_text => {
                        let mut text = &source[prev_end..token.end];
                        if omit_pro_ws {
                            text = text.trim_left();
                        }
                        let mut t = Template::new(mapping);
                        t.push_element(RawString(Template::unescape_tags(text)), line_no, col_no);
                        template_stack.push_front(t);
                    }
                    Rule::expression |
                    Rule::html_expression |
                    Rule::helper_expression |
                    Rule::directive_expression |
                    Rule::partial_expression |
                    Rule::helper_block_end |
                    Rule::raw_block_end |
                    Rule::directive_block_end |
                    Rule::partial_block_end => {
                        let exp = try!(Template::parse_expression(source, it.by_ref(), token.end));
                        if exp.omit_pre_ws {
                            Template::remove_previous_whitespace(&mut template_stack);
                        }

                        omit_pro_ws = exp.omit_pro_ws;

                        match token.rule {
                            Rule::expression => {
                                let el = Expression(exp.name);
                                let mut t = template_stack.front_mut().unwrap();
                                t.push_element(el, line_no, col_no);
                            }
                            Rule::html_expression => {
                                let el = HTMLExpression(exp.name);
                                let mut t = template_stack.front_mut().unwrap();
                                t.push_element(el, line_no, col_no);
                            }
                            Rule::helper_expression => {
                                let helper_template = HelperTemplate {
                                    name: exp.name.as_name().unwrap(),
                                    params: exp.params,
                                    hash: exp.hash,
                                    block_param: exp.block_param,
                                    block: false,
                                    template: None,
                                    inverse: None,
                                };
                                let el = HelperExpression(helper_template);
                                let mut t = template_stack.front_mut().unwrap();
                                t.push_element(el, line_no, col_no);
                            }
                            Rule::directive_expression |
                            Rule::partial_expression => {
                                let directive = Directive {
                                    name: exp.name,
                                    params: exp.params,
                                    hash: exp.hash,
                                    template: None,
                                };
                                let el = if token.rule == Rule::directive_expression {
                                    DirectiveExpression(directive)
                                } else {
                                    PartialExpression(directive)
                                };
                                let mut t = template_stack.front_mut().unwrap();
                                t.push_element(el, line_no, col_no);
                            }
                            Rule::helper_block_end |
                            Rule::raw_block_end => {
                                let mut h = helper_stack.pop_front().unwrap();
                                let close_tag_name = exp.name.as_name().unwrap();
                                if h.name == close_tag_name {
                                    let prev_t = template_stack.pop_front().unwrap();
                                    if h.template.is_some() {
                                        h.inverse = Some(prev_t);
                                    } else {
                                        h.template = Some(prev_t);
                                    }
                                    let t = template_stack.front_mut().unwrap();
                                    t.elements.push(HelperBlock(h));
                                } else {
                                    return Err(
                                        TemplateError::of(
                                            TemplateErrorReason::MismatchingClosedHelper(
                                                h.name,
                                                close_tag_name,
                                            ),
                                        ).at(line_no, col_no),
                                    );
                                }
                            }
                            Rule::directive_block_end |
                            Rule::partial_block_end => {
                                let mut d = directive_stack.pop_front().unwrap();
                                let close_tag_name = exp.name;
                                if d.name == close_tag_name {
                                    let prev_t = template_stack.pop_front().unwrap();
                                    d.template = Some(prev_t);
                                    let t = template_stack.front_mut().unwrap();
                                    if token.rule == Rule::directive_block_end {
                                        t.elements.push(DirectiveBlock(d));
                                    } else {
                                        t.elements.push(PartialBlock(d));
                                    }
                                } else {
                                    return Err(
                                        TemplateError::of(
                                            TemplateErrorReason::MismatchingClosedDirective(
                                                d.name,
                                                close_tag_name,
                                            ),
                                        ).at(line_no, col_no),
                                    );
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                    Rule::hbs_comment => {
                        let text = parser.input().slice(token.start + 3, token.end - 2);
                        let mut t = template_stack.front_mut().unwrap();
                        t.push_element(Comment(text.to_owned()), line_no, col_no);
                    }
                    _ => {}
                }

                if token.rule != Rule::template {
                    prev_end = token.end;
                }
            } else {
                if prev_end < source.len() {
                    let text = &source[prev_end..source.len()];
                    let (line_no, col_no) = parser.input().line_col(prev_end);
                    let mut t = template_stack.front_mut().unwrap();
                    t.push_element(RawString(text.to_owned()), line_no, col_no);
                }
                return Ok(template_stack.pop_front().unwrap());
            }
        }
    }

    pub fn compile_with_name<S: AsRef<str>>(
        source: S,
        name: String,
        mapping: bool,
    ) -> Result<Template, TemplateError> {
        match Template::compile2(source, mapping) {
            Ok(mut t) => {
                t.name = Some(name);
                Ok(t)
            }
            Err(e) => Err(e.in_template(name)),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum TemplateElement {
    RawString(String),
    Expression(Parameter),
    HTMLExpression(Parameter),
    HelperExpression(HelperTemplate),
    HelperBlock(HelperTemplate),
    DirectiveExpression(Directive),
    DirectiveBlock(Directive),
    PartialExpression(Directive),
    PartialBlock(Directive),
    Comment(String),
}

#[test]
fn test_parse_escaped_tag_raw_string() {
    let source = r"foo \\{{bar}}";
    let t = Template::compile(source.to_string()).ok().unwrap();
    assert_eq!(t.elements.len(), 1);
    assert_eq!(
        *t.elements.get(0).unwrap(),
        RawString("foo {{bar}}".to_string())
    );
}

#[test]
fn test_parse_escaped_block_raw_string() {
    let source = r"\\{{{{foo}}}} bar";
    let t = Template::compile(source.to_string()).ok().unwrap();
    assert_eq!(t.elements.len(), 1);
    assert_eq!(
        *t.elements.get(0).unwrap(),
        RawString("{{{{foo}}}} bar".to_string())
    );
}

#[test]
fn test_parse_template() {
    let source = "<h1>{{title}} 你好</h1> {{{content}}}
{{#if date}}<p>good</p>{{else}}<p>bad</p>{{/if}}<img>{{foo bar}}中文你好
{{#unless true}}kitkat{{^}}lollipop{{/unless}}";
    let t = Template::compile(source.to_string()).ok().unwrap();

    assert_eq!(t.elements.len(), 10);

    assert_eq!(*t.elements.get(0).unwrap(), RawString("<h1>".to_string()));
    assert_eq!(
        *t.elements.get(1).unwrap(),
        Expression(Parameter::Name("title".to_string()))
    );

    assert_eq!(
        *t.elements.get(3).unwrap(),
        HTMLExpression(Parameter::Name("content".to_string()))
    );

    match *t.elements.get(5).unwrap() {
        HelperBlock(ref h) => {
            assert_eq!(h.name, "if".to_string());
            assert_eq!(h.params.len(), 1);
            assert_eq!(h.template.as_ref().unwrap().elements.len(), 1);
        }
        _ => {
            panic!("Helper expected here.");
        }
    };

    match *t.elements.get(7).unwrap() {
        HelperExpression(ref h) => {
            assert_eq!(h.name, "foo".to_string());
            assert_eq!(h.params.len(), 1);
            assert_eq!(*(h.params.get(0).unwrap()), Parameter::Name("bar".into()));
        }
        _ => {
            panic!("Helper expression here");
        }
    };

    match *t.elements.get(9).unwrap() {
        HelperBlock(ref h) => {
            assert_eq!(h.name, "unless".to_string());
            assert_eq!(h.params.len(), 1);
            assert_eq!(h.inverse.as_ref().unwrap().elements.len(), 1);
        }
        _ => {
            panic!("Helper expression here");
        }
    };

}

#[test]
fn test_parse_error() {
    let source = "{{#ifequals name compare=\"hello\"}}\nhello\n\t{{else}}\ngood";

    let t = Template::compile(source.to_string());

    assert_eq!(
        t.unwrap_err(),
        TemplateError::of(TemplateErrorReason::InvalidSyntax).at(4, 5)
    );
}

#[test]
fn test_subexpression() {
    let source = "{{foo (bar)}}{{foo (bar baz)}} hello {{#if (baz bar) then=(bar)}}world{{/if}}";
    let t = Template::compile(source.to_string()).ok().unwrap();

    assert_eq!(t.elements.len(), 4);
    match *t.elements.get(0).unwrap() {
        HelperExpression(ref h) => {
            assert_eq!(h.name, "foo".to_owned());
            assert_eq!(h.params.len(), 1);
            if let &Parameter::Subexpression(ref t) = h.params.get(0).unwrap() {
                assert_eq!(t.name, "bar".to_owned());
            } else {
                panic!("Subexpression expected");
            }
        }
        _ => {
            panic!("Helper expression expected");
        }
    };

    match *t.elements.get(1).unwrap() {
        HelperExpression(ref h) => {
            assert_eq!(h.name, "foo".to_string());
            assert_eq!(h.params.len(), 1);
            if let &Parameter::Subexpression(ref t) = h.params.get(0).unwrap() {
                assert_eq!(t.name, "bar".to_owned());
                if let Some(&Parameter::Name(ref n)) = t.params.get(0) {
                    assert_eq!(n, "baz");
                } else {
                    panic!("non-empty param expected ");
                }
            } else {
                panic!("Subexpression expected");
            }
        }
        _ => {
            panic!("Helper expression expected");
        }
    };

    match *t.elements.get(3).unwrap() {
        HelperBlock(ref h) => {
            assert_eq!(h.name, "if".to_string());
            assert_eq!(h.params.len(), 1);
            assert_eq!(h.hash.len(), 1);

            if let &Parameter::Subexpression(ref t) = h.params.get(0).unwrap() {
                assert_eq!(t.name, "baz".to_owned());
                if let Some(&Parameter::Name(ref n)) = t.params.get(0) {
                    assert_eq!(n, "bar");
                } else {
                    panic!("non-empty param expected ");
                }

            } else {
                panic!("Subexpression expected (baz bar)");
            }

            if let &Parameter::Subexpression(ref t) = h.hash.get("then").unwrap() {
                assert_eq!(t.name, "bar".to_owned());
            } else {
                panic!("Subexpression expected (bar)");
            }
        }
        _ => {
            panic!("HelperBlock expected");
        }
    }
}

#[test]
fn test_white_space_omitter() {
    let source = "hello~     {{~world~}} \n  !{{~#if true}}else{{/if~}}".to_string();
    let t = Template::compile(source).ok().unwrap();

    assert_eq!(t.elements.len(), 4);

    assert_eq!(t.elements[0], RawString("hello~".to_string()));
    assert_eq!(t.elements[1], Expression(Parameter::Name("world".into())));
    assert_eq!(t.elements[2], RawString("!".to_string()));

    let t2 = Template::compile("{{#if true}}1  {{~ else ~}} 2 {{~/if}}".to_string())
        .ok()
        .unwrap();
    assert_eq!(t2.elements.len(), 1);
    match t2.elements[0] {
        HelperBlock(ref h) => {
            assert_eq!(
                h.template.as_ref().unwrap().elements[0],
                RawString("1".to_string())
            );
            assert_eq!(
                h.inverse.as_ref().unwrap().elements[0],
                RawString("2".to_string())
            );
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_unclosed_expression() {
    let sources = ["{{invalid", "{{{invalid", "{{invalid}", "{{!hello"];
    for s in sources.iter() {
        let result = Template::compile(s.to_owned());
        if let Err(e) = result {
            match e.reason {
                TemplateErrorReason::InvalidSyntax => {}
                _ => {
                    panic!("Unexpected error type {}", e);
                }
            }
        } else {
            panic!("Undetected error");
        }
    }
}

#[test]
fn test_raw_helper() {
    let source = "hello{{{{raw}}}}good{{night}}{{{{/raw}}}}world";
    match Template::compile(source.to_owned()) {
        Ok(t) => {
            assert_eq!(t.elements.len(), 3);
            assert_eq!(t.elements[0], RawString("hello".to_owned()));
            assert_eq!(t.elements[2], RawString("world".to_owned()));
            match t.elements[1] {
                HelperBlock(ref h) => {
                    assert_eq!(h.name, "raw".to_owned());
                    if let Some(ref ht) = h.template {
                        assert_eq!(ht.elements.len(), 1);
                        assert_eq!(
                            *ht.elements.get(0).unwrap(),
                            RawString("good{{night}}".to_owned())
                        );
                    } else {
                        panic!("helper template not found");
                    }
                }
                _ => {
                    panic!("Unexpected element type");
                }
            }
        }
        Err(e) => {
            panic!("{}", e);
        }

    }
}

#[test]
#[cfg(all(feature = "rustc_ser_type", not(feature = "serde_type")))]
fn test_literal_parameter_parser() {
    match Template::compile("{{hello 1 name=\"value\" valid=false ref=someref}}") {
        Ok(t) => {
            if let HelperExpression(ref ht) = t.elements[0] {
                assert_eq!(ht.params[0], Parameter::Literal(Json::U64(1)));
                assert_eq!(
                    ht.hash["name"],
                    Parameter::Literal(Json::String("value".to_owned()))
                );
                assert_eq!(ht.hash["valid"], Parameter::Literal(Json::Boolean(false)));
                assert_eq!(ht.hash["ref"], Parameter::Name("someref".to_owned()));
            }
        }
        Err(e) => panic!("{}", e),
    }
}

#[test]
#[cfg(serde_type)]
fn test_literal_parameter_parser() {
    match Template::compile("{{hello 1 name=\"value\" valid=false ref=someref}}") {
        Ok(t) => {
            if let HelperExpression(ref ht) = t.elements[0] {
                assert_eq!(ht.params[0], Parameter::Literal(Json::U64(1)));
                assert_eq!(
                    ht.hash["name"],
                    Parameter::Literal(Json::String("value".to_owned()))
                );
                assert_eq!(ht.hash["valid"], Parameter::Literal(Json::Bool(false)));
                assert_eq!(ht.hash["ref"], Parameter::Name("someref".to_owned()));
            }
        }
        Err(e) => panic!("{}", e),
    }
}

#[test]
fn test_template_mapping() {
    match Template::compile2("hello\n  {{~world}}\n{{#if nice}}\n\thello\n{{/if}}", true) {
        Ok(t) => {
            if let Some(ref mapping) = t.mapping {
                assert_eq!(mapping.len(), t.elements.len());
                assert_eq!(mapping[0], TemplateMapping(1, 1));
                assert_eq!(mapping[1], TemplateMapping(2, 3));
                assert_eq!(mapping[3], TemplateMapping(3, 1));
            } else {
                panic!("should contains mapping");
            }
        }
        Err(e) => panic!("{}", e),
    }
}

#[test]
fn test_whitespace_elements() {
    let c = Template::compile(
        "  {{elem}}\n\t{{#if true}} \
                               {{/if}}\n{{{{raw}}}} {{{{/raw}}}}\n{{{{raw}}}}{{{{/raw}}}}\n",
    );
    assert_eq!(c.ok().unwrap().elements.len(), 9);
}

#[test]
fn test_block_param() {
    match Template::compile("{{#each people as |person|}}{{person}}{{/each}}") {
        Ok(t) => {
            if let HelperBlock(ref ht) = t.elements[0] {
                if let Some(BlockParam::Single(Parameter::Name(ref n))) = ht.block_param {
                    assert_eq!(n, "person");
                } else {
                    panic!("block param expected.")
                }
            } else {
                panic!("Helper block expected");
            }
        }
        Err(e) => panic!("{}", e),
    }

    match Template::compile("{{#each people as |key val|}}{{person}}{{/each}}") {
        Ok(t) => {
            if let HelperBlock(ref ht) = t.elements[0] {
                if let Some(BlockParam::Pair((Parameter::Name(ref n1),
                                              Parameter::Name(ref n2)))) = ht.block_param
                {
                    assert_eq!(n1, "key");
                    assert_eq!(n2, "val");
                } else {
                    panic!("helper block param expected.");
                }
            } else {
                panic!("Helper block expected");
            }
        }
        Err(e) => panic!("{}", e),
    }
}

#[test]
fn test_directive() {
    match Template::compile("hello {{* ssh}} world") {
        Err(e) => panic!("{}", e),
        Ok(t) => {
            if let DirectiveExpression(ref de) = t.elements[1] {
                assert_eq!(de.name, Parameter::Name("ssh".to_owned()));
                assert_eq!(de.template, None);
            }
        }
    }

    match Template::compile("hello {{> ssh}} world") {
        Err(e) => panic!("{}", e),
        Ok(t) => {
            if let PartialExpression(ref de) = t.elements[1] {
                assert_eq!(de.name, Parameter::Name("ssh".to_owned()));
                assert_eq!(de.template, None);
            }
        }
    }

    match Template::compile(
        "{{#*inline \"hello\"}}expand to hello{{/inline}}{{> hello}}",
    ) {
        Err(e) => panic!("{}", e),
        Ok(t) => {
            if let DirectiveBlock(ref db) = t.elements[0] {
                assert_eq!(db.name, Parameter::Name("inline".to_owned()));
                assert_eq!(
                    db.params[0],
                    Parameter::Literal(Json::String("hello".to_owned()))
                );
                assert_eq!(
                    db.template.as_ref().unwrap().elements[0],
                    TemplateElement::RawString("expand to hello".to_owned())
                );
            }
        }
    }

    match Template::compile(
        "{{#> layout \"hello\"}}expand to hello{{/layout}}{{> hello}}",
    ) {
        Err(e) => panic!("{}", e),
        Ok(t) => {
            if let PartialBlock(ref db) = t.elements[0] {
                assert_eq!(db.name, Parameter::Name("layout".to_owned()));
                assert_eq!(
                    db.params[0],
                    Parameter::Literal(Json::String("hello".to_owned()))
                );
                assert_eq!(
                    db.template.as_ref().unwrap().elements[0],
                    TemplateElement::RawString("expand to hello".to_owned())
                );
            }
        }
    }
}

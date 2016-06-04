use std::ops::BitOr;
use std::str::Chars;
// use std::fmt;
use std::collections::{BTreeMap, VecDeque};
use std::string::ToString;
use regex::Regex;
use itertools::PutBackN;

#[cfg(feature = "rustc_ser_type")]
use serialize::json::Json;
#[cfg(feature = "serde_type")]
use serde_json::value::Value as Json;
#[cfg(feature = "serde_type")]
use std::str::FromStr;

use TemplateError;
use TemplateError::*;

use self::TemplateElement::{RawString, Expression, HelperExpression, HTMLExpression, HelperBlock,
                            Comment};

#[derive(PartialEq, Clone, Debug)]
pub struct TemplateMapping(pub usize, pub usize);

#[derive(PartialEq, Clone, Debug)]
pub struct Template {
    pub name: Option<String>,
    pub elements: Vec<TemplateElement>,
    pub mapping: Option<Vec<TemplateMapping>>,
}

#[derive(PartialEq, Debug)]
enum ParserState {
    Text,
    HtmlExpression,
    Comment,
    HelperStart,
    HelperEnd,
    Expression,
    RawHelperStart,
    RawHelperEnd,
    RawText,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Parameter {
    Name(String),
    Literal(Json),
    Subexpression(Template),
}

#[derive(PartialEq, Clone, Debug)]
pub struct HelperTemplate {
    pub name: String,
    pub params: Vec<Parameter>,
    pub hash: BTreeMap<String, Parameter>,
    pub template: Option<Template>,
    pub inverse: Option<Template>,
    pub block: bool,
}

fn find_tokens(source: &str) -> Vec<String> {
    let tokenizer = Regex::new(r"[^\s\(\)]+|\([^\)]*\)").unwrap();

    let mut hash_key: Option<&str> = None;
    let mut results: Vec<String> = vec![];
    tokenizer.captures_iter(&source)
             .map(|c| c.at(0).unwrap())
             .fold(&mut results, |r, item| {
                 match hash_key {
                     Some(k) => {
                         r.push(format!("{}{}", k, item));
                         hash_key = None
                     }
                     None => {
                         if item.ends_with("=") {
                             hash_key = Some(item);
                         } else {
                             r.push(item.to_string());
                         }
                     }
                 }
                 r
             });
    results
}

impl HelperTemplate {
    pub fn parse<S: AsRef<str>>(source: S,
                                block: bool,
                                line_no: usize,
                                col_no: usize)
                                -> Result<HelperTemplate, TemplateError> {
        let source = source.as_ref();
        let tokens_vec = find_tokens(&source);
        let mut tokens = tokens_vec.iter();

        let name = tokens.next();
        match name {
            Some(n) => {
                let mut params: Vec<Parameter> = Vec::new();
                let mut hash: BTreeMap<String, Parameter> = BTreeMap::new();

                for t in tokens {
                    if t.contains('=') {
                        let kv = t.split('=').collect::<Vec<&str>>();
                        let value = try!(Parameter::parse(kv.get(1).unwrap().to_string()));
                        hash.insert(kv.get(0).unwrap().to_string(), value);
                    } else {
                        let value = try!(Parameter::parse(t.to_string()));
                        params.push(value);
                    }
                }

                Ok(HelperTemplate{
                    name: n.to_string(),
                    params: params,
                    hash: hash,
                    template: Option::None,
                    inverse: Option::None,
                    block: block
                })
            },
            None =>
                // As far as I can see this is bare "{{" at the end of file.
                Err(TemplateError::UnclosedBraces(line_no, col_no))
        }
    }
}

impl Parameter {
    pub fn parse<S: AsRef<str>>(source: S) -> Result<Parameter, TemplateError> {
        let source = source.as_ref();
        // move this to static scope when regex! is stable
        let subexpr_regex = Regex::new(r"\(([^\)]+)\)").unwrap();

        if let Some(caps) = subexpr_regex.captures(&source) {
            let parameter = caps.at(1).unwrap();

            let mut temp = String::with_capacity(source.len());
            temp.push_str("{{");
            temp.push_str(parameter);
            temp.push_str("}}");

            let sub_template = try!(Template::compile(temp));
            Ok(Parameter::Subexpression(sub_template))
        } else {
            if let Ok(json) = Json::from_str(source) {
                Ok(Parameter::Literal(json))
            } else {
                Ok(Parameter::Name(source.to_owned()))
            }
        }
    }
}

#[derive(PartialEq)]
enum WhiteSpaceOmit {
    Left = 0x01,
    Right = 0x10,
    Both = 0x11,
    None = 0x00,
}

impl From<u8> for WhiteSpaceOmit {
    fn from(n: u8) -> WhiteSpaceOmit {
        match n {
            0x01 => WhiteSpaceOmit::Left,
            0x10 => WhiteSpaceOmit::Right,
            0x11 => WhiteSpaceOmit::Both,
            0x00 => WhiteSpaceOmit::None,
            _ => WhiteSpaceOmit::None,
        }
    }
}

impl BitOr<WhiteSpaceOmit> for WhiteSpaceOmit {
    type Output = WhiteSpaceOmit;

    fn bitor(self, right: WhiteSpaceOmit) -> WhiteSpaceOmit {
        WhiteSpaceOmit::from((self as u8) | (right as u8))
    }
}


fn process_whitespace(buf: &str, wso: &mut WhiteSpaceOmit) -> String {
    let result = match *wso {
        WhiteSpaceOmit::Left => buf.trim_left().to_string(),
        WhiteSpaceOmit::Right => buf.trim_right().to_string(),
        WhiteSpaceOmit::Both => buf.trim().to_string(),
        WhiteSpaceOmit::None => buf.to_string(),
    };
    *wso = WhiteSpaceOmit::None;
    result
}

fn peek_chars(it: &mut PutBackN<Chars>, n: usize) -> Option<String> {
    let mut tmp = String::new();

    for _ in 0..n {
        if let Some(c) = it.next() {
            tmp.push(c);
        }
    }

    for i in tmp.chars().rev() {
        it.put_back(i);
    }

    Some(tmp)
}

fn iter_skip<I: Iterator>(it: &mut I, n: usize) {
    for _ in 0..n {
        it.next();
    }
}

impl Template {
    pub fn new(mapping: bool) -> Template {
        Template {
            elements: Vec::new(),
            name: None,
            mapping: if mapping {
                Some(vec![TemplateMapping(1, 1)])
            } else {
                None
            },
        }
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

    pub fn compile2<S: AsRef<str>>(source: S, mapping: bool) -> Result<Template, TemplateError> {
        let source = source.as_ref();
        let mut helper_stack: VecDeque<HelperTemplate> = VecDeque::new();
        let mut template_stack: VecDeque<Template> = VecDeque::new();
        template_stack.push_front(Template::new(mapping));

        let mut buffer: String = String::new();
        let mut state = ParserState::Text;

        let mut line_no: usize = 1;
        let mut col_no: usize = 0;
        let mut ws_omitter = WhiteSpaceOmit::None;
        let mut it = PutBackN::new(source.chars());

        loop {
            if let Some(c) = it.next() {
                if c == '\n' {
                    line_no = line_no + 1;
                    col_no = 0;
                } else {
                    col_no = col_no + 1;
                }

                match c {
                    // interested characters, peek more chars
                    '{' | '~' | '}' => {
                        it.put_back(c);

                        // raw helper
                        if let Some(slice) = peek_chars(&mut it, 5) {
                            if slice == "{{{{/" {
                                iter_skip(&mut it, 5);
                                // TODO: remove dup code
                                if !buffer.is_empty() {
                                    let mut t = template_stack.front_mut().unwrap();
                                    let buf_clone = process_whitespace(&buffer, &mut ws_omitter);
                                    t.push_element(RawString(buf_clone), line_no, col_no);
                                    buffer.clear();
                                }
                                let t = template_stack.pop_front().unwrap();
                                let h = helper_stack.front_mut().unwrap();
                                h.template = Some(t);
                                state = ParserState::RawHelperEnd;
                                continue;
                            }
                        }

                        if let Some(slice) = peek_chars(&mut it, 4) {
                            if slice == "{{{{" {
                                iter_skip(&mut it, 4);
                                // TODO: remove dup code
                                if !buffer.is_empty() {
                                    let mut t = template_stack.front_mut().unwrap();
                                    let buf_clone = process_whitespace(&buffer, &mut ws_omitter);
                                    t.push_element(RawString(buf_clone), line_no, col_no);
                                    buffer.clear();
                                }
                                state = ParserState::RawHelperStart;
                                continue;
                            } else if slice == "}}}}" {
                                match state {
                                    ParserState::RawHelperStart => {
                                        iter_skip(&mut it, 4);
                                        let helper = try!(HelperTemplate::parse(buffer.clone(),
                                                                                true,
                                                                                line_no,
                                                                                col_no));
                                        helper_stack.push_front(helper);
                                        template_stack.push_front(Template::new(mapping));

                                        buffer.clear();
                                        state = ParserState::RawText;
                                        continue;
                                    }

                                    ParserState::RawHelperEnd => {
                                        iter_skip(&mut it, 4);
                                        let name = buffer.trim_matches(' ').to_string();
                                        if name == helper_stack.front().unwrap().name {
                                            let h = helper_stack.pop_front().unwrap();
                                            let mut t = template_stack.front_mut().unwrap();
                                            t.push_element(HelperBlock(h), line_no, col_no);
                                            buffer.clear();
                                            state = ParserState::Text;
                                            continue;
                                        } else {
                                            return Err(MismatchingClosedHelper(
                                                line_no, col_no,
                                                helper_stack.front().unwrap().name.clone(),
                                                name));
                                        }
                                    }

                                    _ => {}
                                }
                            }
                        }

                        // within a raw helper, any character will be treated as raw string
                        if state == ParserState::RawText {
                            iter_skip(&mut it, 1);
                            buffer.push(c);
                            continue;
                        }

                        if let Some(mut slice) = peek_chars(&mut it, 3) {
                            if slice == "{{~" {
                                ws_omitter = ws_omitter | WhiteSpaceOmit::Right;
                                // read another char and remove ~
                                slice = peek_chars(&mut it, 4).unwrap();
                                slice.remove(2);
                                iter_skip(&mut it, 1);
                            }
                            if slice == "~}}" {
                                ws_omitter = ws_omitter | WhiteSpaceOmit::Left;
                                iter_skip(&mut it, 1);
                                slice = peek_chars(&mut it, 3).unwrap();
                            }
                            state = match slice.as_ref() {
                                "{{{" | "{{!" | "{{#" | "{{/" => {
                                    iter_skip(&mut it, 3);
                                    if !buffer.is_empty() {
                                        let mut t = template_stack.front_mut().unwrap();
                                        let buf_clone = process_whitespace(&buffer,
                                                                           &mut ws_omitter);
                                        t.push_element(RawString(buf_clone), line_no, col_no);
                                        buffer.clear();
                                    }
                                    match slice.as_ref() {
                                        "{{{" => ParserState::HtmlExpression,
                                        "{{!" => ParserState::Comment,
                                        "{{#" => ParserState::HelperStart,
                                        "{{/" => {
                                            let t = template_stack.pop_front().unwrap();
                                            let h = helper_stack.front_mut().unwrap();
                                            if h.template.is_some() {
                                                h.inverse = Some(t);
                                            } else {
                                                h.template = Some(t);
                                            }
                                            ParserState::HelperEnd
                                        }
                                        _ => unreachable!(),  // because of check above
                                    }
                                }
                                "}}}" => {
                                    iter_skip(&mut it, 3);
                                    let mut t = template_stack.front_mut().unwrap();
                                    t.push_element(HTMLExpression(try!(Parameter::parse(buffer.clone().trim_matches(' ').to_string()))), line_no, col_no);
                                    buffer.clear();
                                    ParserState::Text
                                }
                                _ => {
                                    match if slice.len() > 2 {
                                        &slice[0..2]
                                    } else {
                                        slice.as_ref()
                                    } {
                                        "{{" => {
                                            iter_skip(&mut it, 2);
                                            if !buffer.is_empty() {
                                                let mut t = template_stack.front_mut().unwrap();
                                                let buf_clone = process_whitespace(&buffer,
                                                                                   &mut ws_omitter);

                                                t.push_element(RawString(buf_clone),
                                                               line_no,
                                                               col_no);
                                                buffer.clear();
                                            }
                                            ParserState::Expression
                                        }
                                        "}}" => {
                                            iter_skip(&mut it, 2);
                                            match state {
                                                ParserState::Expression => {
                                                    if !buffer.is_empty() {
                                                        // {{else}} or {{^}} within a helper block
                                                        if buffer.trim() == "else" ||
                                                           buffer.trim() == "^" {
                                                            buffer.clear(); // drop else
                                                            let t = template_stack.pop_front()
                                                                                  .unwrap();
                                                            let h = helper_stack.front_mut()
                                                                                .unwrap();
                                                            h.template = Some(t);
                                                            template_stack.push_front(Template::new(mapping));
                                                            ParserState::Text
                                                        } else {
                                                            if find_tokens(&buffer).len() > 1 {
                                                                // inline helper
                                                                let helper = try!(HelperTemplate::parse(buffer.clone(), false, line_no, col_no));
                                                                let mut t =
                                                                    template_stack.front_mut()
                                                                                  .unwrap();
                                                                t.push_element(HelperExpression(helper), line_no, col_no);
                                                                buffer.clear();
                                                                ParserState::Text
                                                            } else {
                                                                let mut t =
                                                                    template_stack.front_mut()
                                                                                  .unwrap();
                                                                t.push_element(Expression(
                                                                    try!(Parameter::parse(buffer.clone().trim_matches(' ').to_string()))), line_no, col_no);
                                                                buffer.clear();
                                                                ParserState::Text
                                                            }
                                                        }
                                                    } else {
                                                        return Err(UnclosedBraces(line_no, col_no));
                                                    }
                                                }
                                                ParserState::Comment => {
                                                    let mut t = template_stack.front_mut().unwrap();
                                                    t.push_element(Comment(buffer.clone()),
                                                                   line_no,
                                                                   col_no);
                                                    buffer.clear();
                                                    ParserState::Text
                                                }
                                                ParserState::HelperStart => {
                                                    let helper =
                                                        try!(HelperTemplate::parse(buffer.clone(),
                                                                                   true,
                                                                                   line_no,
                                                                                   col_no));
                                                    helper_stack.push_front(helper);
                                                    template_stack.push_front(Template::new(mapping));

                                                    buffer.clear();
                                                    ParserState::Text
                                                }
                                                ParserState::HelperEnd => {
                                                    let name = buffer.trim_matches(' ').to_string();
                                                    if name == helper_stack.front().unwrap().name {
                                                        let h = helper_stack.pop_front().unwrap();
                                                        let mut t = template_stack.front_mut()
                                                                                  .unwrap();
                                                        t.push_element(HelperBlock(h),
                                                                       line_no,
                                                                       col_no);
                                                        buffer.clear();
                                                        ParserState::Text
                                                    } else {
                                                        return Err(MismatchingClosedHelper(
                                                            line_no, col_no,
                                                            helper_stack.front().unwrap().name.clone(),
                                                            name));
                                                    }
                                                }
                                                _ => {
                                                    return Err(UnexpectedClosingBraces(line_no,
                                                                                       col_no))
                                                }
                                            }
                                        }
                                        _ => {
                                            iter_skip(&mut it, 1);
                                            buffer.push(c);
                                            state
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        buffer.push(c);
                    }
                }
            } else {
                break;
            }
        }

        if !buffer.is_empty() {
            let mut t = template_stack.front_mut().unwrap();
            let buf_clone = process_whitespace(&buffer, &mut ws_omitter);
            t.push_element(RawString(buf_clone), line_no, col_no);
        }

        if !helper_stack.is_empty() {
            return Err(UnclosedHelper(line_no, col_no, helper_stack.front().unwrap().name.clone()));
        }

        if state != ParserState::Text {
            return Err(UnclosedExpression(line_no, col_no));
        }

        let mut t = template_stack.pop_front().unwrap();
        if let Some(ref mut mapping) = t.mapping {
            mapping.pop();
        }

        return Ok(t);
    }

    pub fn compile_with_name<S: AsRef<str>>(source: S,
                                            name: String)
                                            -> Result<Template, TemplateError> {
        let mut t = try!(Template::compile(source));
        t.name = Some(name);
        Ok(t)
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum TemplateElement {
    RawString(String),
    Expression(Parameter),
    HTMLExpression(Parameter),
    HelperExpression(HelperTemplate),
    HelperBlock(HelperTemplate),
    Comment(String),
}

#[test]
fn test_parse_helper_start_tag() {
    let source = "if not name compare=1".to_string();
    let h = HelperTemplate::parse(source, true, 0, 0).ok().unwrap();

    assert_eq!(h.name, "if".to_string());
    assert_eq!(h.params,
               vec![Parameter::Name("not".into()), Parameter::Name("name".into())]);

    let key = "compare".to_string();
    let value = h.hash.get(&key).unwrap();
    assert_eq!(*value, Parameter::Literal(Json::U64(1)));
}

#[test]
fn test_parse_template() {
    let source = "<h1>{{title}} 你好</h1> {{{content}}}
{{#if date}}<p>good</p>{{else}}<p>bad</p>{{/if}}<img>{{foo bar}}中文你好
{{#unless true}}kitkat{{^}}lollipop{{/unless}}";
    let t = Template::compile(source.to_string()).ok().unwrap();

    assert_eq!(t.elements.len(), 10);

    assert_eq!(*t.elements.get(0).unwrap(), RawString("<h1>".to_string()));
    assert_eq!(*t.elements.get(1).unwrap(),
               Expression(Parameter::Name("title".to_string())));

    assert_eq!(*t.elements.get(3).unwrap(),
               HTMLExpression(Parameter::Name("content".to_string())));

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

    assert_eq!(format!("{}", t.unwrap_err()),
               r#"helper "ifequals" was not closed on the end of file at line 4, column 4"#);
}

#[test]
fn test_subexpression() {
    let source = "{{foo (bar)}}{{foo (bar baz)}} hello {{#if (baz bar) then=(bar)}}world{{/if}}";
    let t = Template::compile(source.to_string()).ok().unwrap();

    assert_eq!(t.elements.len(), 4);
    match *t.elements.get(0).unwrap() {
        HelperExpression(ref h) => {
            assert_eq!(h.name, "foo".to_string());
            assert_eq!(h.params.len(), 1);
            if let &Parameter::Subexpression(ref t) = h.params.get(0).unwrap() {
                assert_eq!(*t.elements.get(0).unwrap(),
                           Expression(Parameter::Name("bar".to_string())));
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
                assert_eq!(*t.elements.get(0).unwrap(),
                           HelperExpression(HelperTemplate {
                               name: "bar".to_owned(),
                               params: vec![Parameter::Name("baz".to_owned())],
                               hash: BTreeMap::new(),
                               template: None,
                               inverse: None,
                               block: false,
                           }));
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
                assert_eq!(*t.elements.get(0).unwrap(),
                           HelperExpression(HelperTemplate {
                               name: "baz".to_owned(),
                               params: vec![Parameter::Name("bar".to_owned())],
                               hash: BTreeMap::new(),
                               template: None,
                               inverse: None,
                               block: false,
                           }));
            } else {
                panic!("Subexpression expected (baz bar)");
            }

            if let &Parameter::Subexpression(ref t) = h.hash.get("then").unwrap() {
                assert_eq!(*t.elements.get(0).unwrap(),
                           Expression(Parameter::Name("bar".to_string())));
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
}

#[test]
fn test_find_tokens() {
    let source: String = "hello   good (nice) (hello world)\n\t\t world hello=world hello=(world) \
                          hello=(world 0)"
                             .into();
    let tokens: Vec<String> = find_tokens(&source[..]);
    assert_eq!(tokens,
               vec!["hello".to_string(),
                    "good".to_string(),
                    "(nice)".to_string(),
                    "(hello world)".to_string(),
                    "world".to_string(),
                    "hello=world".to_string(),
                    "hello=(world)".to_string(),
                    "hello=(world 0)".to_string()]);
}

#[test]
fn test_unclosed_expression() {
    let sources = ["{{invalid", "{{{invalid", "{{invalid}", "{{!hello"];
    for s in sources.iter() {
        let result = Template::compile(s.to_owned());
        if let Err(e) = result {
            match e {
                TemplateError::UnclosedExpression(_, _) => {}
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
                        assert_eq!(*ht.elements.get(0).unwrap(),
                                   RawString("good{{night}}".to_owned()));
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
#[cfg(rust_ser_type)]
fn test_literal_parameter_parser() {
    match Template::compile("{{hello 1 name=\"value\" valid=false ref=someref}}") {
        Ok(t) => {
            if let HelperExpression(ref ht) = t.elements[0] {
                assert_eq!(ht.params[0], Parameter::Literal(Json::U64(1)));
                assert_eq!(ht.hash["name"],
                           Parameter::Literal(Json::String("value".to_owned())));
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
                assert_eq!(ht.hash["name"],
                           Parameter::Literal(Json::String("value".to_owned())));
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

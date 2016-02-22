use std::ops::BitOr;
use std::str::Chars;
use std::fmt::{self};
use std::collections::{BTreeMap, VecDeque};
use std::string::ToString;
use num::FromPrimitive;
use regex::Regex;
use itertools::PutBackN;

use TemplateError;
use TemplateError::*;

use self::TemplateElement::{RawString, Expression, HelperExpression,
                            HTMLExpression, HelperBlock, Comment};

#[derive(PartialEq, Clone, Debug)]
pub struct Template {
    pub name: Option<String>,
    pub elements: Vec<TemplateElement>
}

#[derive(PartialEq, Debug, Copy, Clone)]
enum ParserState {
    Text,
    HtmlExpression,
    Comment,
    HelperStart,
    HelperEnd,
    Expression,
    Raw,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Parameter {
    Name(String),
    Subexpression(Template)
}

#[derive(PartialEq, Clone, Debug)]
pub struct HelperTemplate {
    pub name: String,
    pub params: Vec<Parameter>,
    pub hash: BTreeMap<String, Parameter>,
    pub template: Option<Template>,
    pub inverse: Option<Template>,
    pub block: bool
}

impl ToString for HelperTemplate {
    fn to_string(&self) -> String {

        let mut buf = String::new();

        if self.block {
            buf.push_str(format!("{{{{#{}", self.name).as_ref());
        } else {
            buf.push_str(format!("{{{{{}", self.name).as_ref());
        }

        for p in self.params.iter() {
            buf.push_str(format!(" {}", p).as_ref());
        }

        for k in self.hash.keys() {
            buf.push_str(format!(" {}={}", k, self.hash.get(k).unwrap()).as_ref());
        }

        buf.push_str("}}");

        if self.block {
            if let Some(ref tpl) = self.template {
                buf.push_str(tpl.to_string().as_ref())
            }

            if let Some(ref ivs) = self.inverse {
                buf.push_str("{{else}}");
                buf.push_str(ivs.to_string().as_ref());
            }
            buf.push_str(format!("{{{{/{}}}}}", self.name).as_ref());
        }
        buf
    }
}

fn find_tokens(source: &str) -> Vec<String> {
    let tokenizer = Regex::new(r"[^\s\(\)]+|\([^\)]*\)").unwrap();

    let mut hash_key: Option<&str> = None;
    let mut results: Vec<String> = vec![];
    tokenizer.captures_iter(&source).map(|c| c.at(0).unwrap())
        .fold(&mut results, |r, item| {
            match hash_key {
                Some(k) => {
                    r.push(format!("{}{}", k, item));
                    hash_key = None
                },
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
    pub fn parse<S: AsRef<str>>(source: S, block: bool, line_no: usize, col_no: usize) -> Result<HelperTemplate, TemplateError> {
        let source = source.as_ref();
        // FIXME, cache this regex
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
            Ok(Parameter::Name(source.to_owned()))
        }
    }
}

impl fmt::Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Parameter::Name(ref name) => {
                try!(write!(f, "{}", name))
            },
            &Parameter::Subexpression(ref template) => {
                let template_string = template.to_string();
                try!(write!(f, "({})", template_string[2..template_string.len()-2].to_string()))
            }
        }
        Ok(())
    }
}

#[derive(PartialEq)]
enum WhiteSpaceOmit {
    Left = 0x01,
    Right = 0x10,
    Both = 0x11,
    None = 0x00
}

impl FromPrimitive for WhiteSpaceOmit {
    fn from_i64(n: i64) -> Option<WhiteSpaceOmit> {
        match n {
            0x01 => Some(WhiteSpaceOmit::Left),
            0x10 => Some(WhiteSpaceOmit::Right),
            0x11 => Some(WhiteSpaceOmit::Both),
            0x00 => Some(WhiteSpaceOmit::None),
            _ => None
        }
    }

    fn from_u64(n: u64) -> Option<WhiteSpaceOmit> {
        match n {
            0x01 => Some(WhiteSpaceOmit::Left),
            0x10 => Some(WhiteSpaceOmit::Right),
            0x11 => Some(WhiteSpaceOmit::Both),
            0x00 => Some(WhiteSpaceOmit::None),
            _ => None
        }
    }
}

impl BitOr<WhiteSpaceOmit> for WhiteSpaceOmit {
    type Output = WhiteSpaceOmit;

    fn bitor(self, right: WhiteSpaceOmit) -> WhiteSpaceOmit {
        FromPrimitive::from_u8((self as u8) | (right as u8)).unwrap()
    }
}


fn process_whitespace(buf: &str, wso: &mut WhiteSpaceOmit) -> String {
    let result = match *wso {
        WhiteSpaceOmit::Left => {
            buf.trim_left().to_string()
        },
        WhiteSpaceOmit::Right => {
            buf.trim_right().to_string()
        },
        WhiteSpaceOmit::Both => {
            buf.trim().to_string()
        },
        WhiteSpaceOmit::None => {
            buf.to_string()
        }
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
    pub fn new() -> Template {
        Template {
            elements: Vec::new(),
            name: None
        }
    }

    pub fn compile<S: AsRef<str>>(source: S) -> Result<Template, TemplateError> {
        let source = source.as_ref();
        let mut helper_stack: VecDeque<HelperTemplate> = VecDeque::new();
        let mut template_stack: VecDeque<Template> = VecDeque::new();
        template_stack.push_front(Template::new());

        let mut buffer: String = String::new();
        let mut state = ParserState::Text;
        let mut old_state = ParserState::Text;

        let mut line_no:usize = 1;
        let mut col_no:usize = 0;
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
                        
                        if state != ParserState::Raw {
                            if let Some(slice) = peek_chars(&mut it, 8) {
                                if slice == "{{#raw}}" {
                                    old_state = state;
                                    state = ParserState::Raw;
                                    iter_skip(&mut it, 8);
                                    continue;
                                }
                            }
                        }
                        
                        if let Some(slice) = peek_chars(&mut it, 8) {
                            if slice == "{{/raw}}" {
                                state = old_state;
                                iter_skip(&mut it, 8);
                                continue;
                            }
                        }
                        
                        if state == ParserState::Raw {
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
                                        let buf_clone = process_whitespace(&buffer, &mut ws_omitter);
                                        t.elements.push(RawString(buf_clone));
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
                                        },
                                        _ => unreachable!(),  // because of check above
                                    }
                                },
                                "}}}" => {
                                    iter_skip(&mut it, 3);
                                    let mut t = template_stack.front_mut().unwrap();
                                    t.elements.push(HTMLExpression(
                                        try!(Parameter::parse(buffer.clone().trim_matches(' ').to_string()))));
                                    buffer.clear();
                                    ParserState::Text
                                },
                                _ => {
                                    match if slice.len() > 2 { &slice[0..2] } else { slice.as_ref() } {
                                        "{{" => {
                                            iter_skip(&mut it, 2);
                                            if !buffer.is_empty() {
                                                let mut t = template_stack.front_mut().unwrap();
                                                let buf_clone = process_whitespace(&buffer, &mut ws_omitter);
                                                t.elements.push(RawString(buf_clone));
                                                buffer.clear();
                                            }
                                            ParserState::Expression
                                        },
                                        "}}" => {
                                            iter_skip(&mut it, 2);
                                            match state {
                                                ParserState::Expression => {
                                                    if !buffer.is_empty() {
                                                        // {{else}} or {{^}} within a helper block
                                                        if buffer.trim() == "else" || buffer.trim() == "^" {
                                                            buffer.clear(); // drop else
                                                            let t = template_stack.pop_front().unwrap();
                                                            let h = helper_stack.front_mut().unwrap();
                                                            h.template = Some(t);
                                                            template_stack.push_front(Template::new());
                                                            ParserState::Text
                                                        } else {
                                                            if find_tokens(&buffer).len() > 1 {
                                                                //inline helper
                                                                let helper = try!(HelperTemplate::parse(buffer.clone(), false, line_no, col_no));
                                                                let mut t = template_stack.front_mut().unwrap();
                                                                t.elements.push(HelperExpression(helper));
                                                                buffer.clear();
                                                                ParserState::Text
                                                            } else {
                                                                let mut t = template_stack.front_mut().unwrap();
                                                                t.elements.push(Expression(
                                                                    try!(Parameter::parse(buffer.clone().trim_matches(' ').to_string()))));
                                                                buffer.clear();
                                                                ParserState::Text
                                                            }
                                                        }
                                                    } else {
                                                        return Err(UnclosedBraces(line_no, col_no))
                                                    }
                                                },
                                                ParserState::Comment => {
                                                    let mut t = template_stack.front_mut().unwrap();
                                                    t.elements.push(Comment(buffer.clone()));
                                                    buffer.clear();
                                                    ParserState::Text
                                                },
                                                ParserState::HelperStart => {
                                                    let helper = try!(HelperTemplate::parse(buffer.clone(), true, line_no, col_no));
                                                    helper_stack.push_front(helper);
                                                    template_stack.push_front(Template::new());

                                                    buffer.clear();
                                                    ParserState::Text
                                                },
                                                ParserState::HelperEnd => {
                                                    let name = buffer.trim_matches(' ').to_string();
                                                    if name == helper_stack.front().unwrap().name {
                                                        let h = helper_stack.pop_front().unwrap();
                                                        let mut t = template_stack.front_mut().unwrap();
                                                        t.elements.push(HelperBlock(h));
                                                        buffer.clear();
                                                        ParserState::Text
                                                    } else {
                                                        return Err(MismatchingClosedHelper(
                                                            line_no, col_no,
                                                            helper_stack.front().unwrap().name.clone(),
                                                            name));
                                                    }
                                                },
                                                _ => return Err(UnexpectedClosingBraces(line_no, col_no)),
                                            }
                                        },
                                        _ => {
                                            iter_skip(&mut it, 1);
                                            buffer.push(c);
                                            state
                                        }
                                    }
                                }
                            }
                        }
                    },
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
            t.elements.push(TemplateElement::RawString(buf_clone));
        }

        if !helper_stack.is_empty(){
            return Err(UnclosedHelper(line_no, col_no, helper_stack.front().unwrap().name.clone()));
        }

        if state != ParserState::Text {
            return Err(UnclosedExpression(line_no, col_no));
        }

        return Ok(template_stack.pop_front().unwrap());
    }

    pub fn compile_with_name<S: AsRef<str>>(source: S, name: String) -> Result<Template, TemplateError> {
        let mut t = try!(Template::compile(source));
        t.name = Some(name);
        Ok(t)
    }
}

impl ToString for Template {
    fn to_string(&self) -> String {
        let mut buf = String::new();
        for v in self.elements.iter() {
            buf.push_str(v.to_string().as_ref());
        }
        buf
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

impl ToString for TemplateElement {
    fn to_string(&self) -> String {
        match *self {
            RawString(ref v) => {
                v.clone()
            },
            Expression(ref v) => {
                // {{ escape to {
                format!("{{{{{}}}}}", v)
            },
            HTMLExpression(ref v) => {
                format!("{{{{{{{}}}}}}}", v)
            },
            HelperExpression(ref helper) => {
                helper.to_string()
            }
            HelperBlock(ref helper) => {
                helper.to_string()
            }
            Comment(ref v) => {
                format!("{{!{}}}", v)
            }
        }
    }
}

#[test]
fn test_parse_helper_start_tag() {
    let source = "if not name compare=1".to_string();
    let h = HelperTemplate::parse(source, true, 0, 0).ok().unwrap();

    assert_eq!(h.name, "if".to_string());
    assert_eq!(h.params, vec::<Parameter>![Parameter::Name("not".into()),
                                           Parameter::Name("name".into())]);

    let key = "compare".to_string();
    let value = h.hash.get(&key).unwrap();
    assert_eq!(*value, Parameter::Name("1".into()));
}

#[test]
fn test_parse_template() {
    let source = "<h1>{{title}} 你好</h1> {{{content}}}
{{#if date}}<p>good</p>{{else}}<p>bad</p>{{/if}}<img>{{foo bar}}中文你好
{{#unless true}}kitkat{{^}}lollipop{{/unless}}";
    let t = Template::compile(source.to_string()).ok().unwrap();

    assert_eq!(t.elements.len(), 10);
    assert_eq!((*t.elements.get(0).unwrap()).to_string(), "<h1>".to_string());
    assert_eq!(*t.elements.get(1).unwrap(), Expression(Parameter::Name("title".to_string())));

    assert_eq!((*t.elements.get(3).unwrap()).to_string(), "{{{content}}}".to_string());

    match *t.elements.get(5).unwrap() {
        HelperBlock(ref h) => {
            assert_eq!(h.name, "if".to_string());
            assert_eq!(h.params.len(), 1);
            assert_eq!(h.template.as_ref().unwrap().elements.len(), 1);
        },
        _ => {
            panic!("Helper expected here.");
        }
    };

    match *t.elements.get(7).unwrap() {
        HelperExpression(ref h) => {
            assert_eq!(h.name, "foo".to_string());
            assert_eq!(h.params.len(), 1);
            assert_eq!(*(h.params.get(0).unwrap()), Parameter::Name("bar".into()));
        },
        _ => {
            panic!("Helper expression here");
        }
    };

    match *t.elements.get(9).unwrap() {
        HelperBlock(ref h) => {
            assert_eq!(h.name, "unless".to_string());
            assert_eq!(h.params.len(), 1);
            assert_eq!(h.inverse.as_ref().unwrap().elements.len(), 1);
        },
        _ => {
            panic!("Helper expression here");
        }
    };

}

#[test]
fn test_helper_to_string() {
    let source = "{{#ifequals name compare=\"hello\"}}hello{{else}}good{{/ifequals}}".to_string();

    let t = Template::compile(source.to_string()).ok().unwrap();

    assert_eq!(t.elements.len(), 1);
    assert_eq!(t.elements.get(0).unwrap().to_string(), source);
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
                assert_eq!(t.to_string(), "{{bar}}".to_string());
            } else {
                panic!("Subexpression expected");
            }
        },
        _ => {
            panic!("Helper expression expected");
        }
    };

    match *t.elements.get(1).unwrap() {
        HelperExpression(ref h) => {
            assert_eq!(h.name, "foo".to_string());
            assert_eq!(h.params.len(), 1);
            if let &Parameter::Subexpression(ref t) = h.params.get(0).unwrap() {
                assert_eq!(t.to_string(), "{{bar baz}}".to_string());
            } else {
                panic!("Subexpression expected");
            }
        },
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
                assert_eq!(t.to_string(), "{{baz bar}}".to_string())
            } else {
                panic!("Subexpression expected (baz bar)");
            }

            if let &Parameter::Subexpression(ref t) = h.hash.get("then").unwrap() {
                assert_eq!(t.to_string(), "{{bar}}".to_string())
            } else {
                panic!("Subexpression expected (bar)");
            }
        },
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
    let source: String = "hello   good (nice) (hello world)\n\t\t world hello=world hello=(world) hello=(world 0)".into();
    let tokens: Vec<String> = find_tokens(&source[..]);
    assert_eq!(tokens, vec::<String>!["hello".to_string(),
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
                TemplateError::UnclosedExpression(_, _) => {
                },
                _ => {
                    panic!("Unexpected error type");
                }
            }
        } else {
            panic!("Undetected error");
        }
    }
}

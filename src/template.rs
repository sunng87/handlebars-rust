use std::cmp::min;
use std::ops::BitOr;
use std::error;
use std::fmt::{self, Display, Formatter};
use std::collections::{BTreeMap, VecDeque};
use std::string::ToString;
use num::FromPrimitive;
use regex::Regex;

use support::str::SliceChars;

use self::TemplateElement::{RawString, Expression, HelperExpression,
                            HTMLExpression, HelperBlock, Comment};

#[derive(PartialEq, Clone, Debug)]
pub struct Template {
    pub elements: Vec<TemplateElement>
}

#[derive(PartialEq, Debug)]
enum ParserState {
    Text,
    HtmlExpression,
    Comment,
    HelperStart,
    HelperEnd,
    Expression,
    Invalid
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

#[derive(Debug, Clone, Copy)]
pub struct TemplateError;

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "TemplateError")
    }
}

impl error::Error for TemplateError {
    fn description(&self) -> &str {
        "Template error"
    }
}

impl HelperTemplate {
    fn parse(source: String, block: bool) -> Result<HelperTemplate, TemplateError> {
        let mut tokens = source.split(|c: char| c.is_whitespace())
            .filter(|s: &&str| !(*s).is_empty());

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
                Err(TemplateError)
        }
    }
}

impl Parameter {
    fn parse(source: String) -> Result<Parameter, TemplateError> {
        // move this to static scope when regex! is stable
        let subexpr_regex = Regex::new(r"\((\S+)\)").unwrap();

        if let Some(caps) = subexpr_regex.captures(&source) {
            let parameter = caps.at(1).unwrap();

            let mut temp = String::with_capacity(source.len());
            temp.push_str("{{");
            temp.push_str(parameter);
            temp.push_str("}}");

            let sub_template = try!(Template::compile(temp));
            Ok(Parameter::Subexpression(sub_template))
        } else {
            Ok(Parameter::Name(source.clone()))
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


fn process_whitespace(buf: &String, wso: &mut WhiteSpaceOmit) -> String {
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
            buf.clone()
        }
    };
    *wso = WhiteSpaceOmit::None;
    result
}

impl Template {
    pub fn compile(source: String) -> Result<Template, TemplateError> {
        let mut helper_stack: VecDeque<HelperTemplate> = VecDeque::new();
        let mut template_stack: VecDeque<Template> = VecDeque::new();
        template_stack.push_front(Template{ elements: Vec::new() });

        let mut buffer: String = String::new();
        let mut state = ParserState::Text;

        let mut c:usize = 0;
        let mut ws_omitter = WhiteSpaceOmit::None;
        let source_len = source.chars().count();
        while c < source_len {
            let mut slice = source.slice_chars_alt(c, min(c+3, source_len)).to_string();
            if slice == "{{~" {
                ws_omitter = ws_omitter | WhiteSpaceOmit::Right;
                // read another char and remove ~
                slice = source.slice_chars_alt(c, min(c+4, source_len)).to_string();
                slice.remove(2);
                c += 1;
            }
            if slice == "~}}" {
                ws_omitter = ws_omitter | WhiteSpaceOmit::Left;
                c += 1;
                slice = source.slice_chars_alt(c, min(c+3, source_len)).to_string();
            }
            state = match slice.as_ref() {
                "{{{" | "{{!" | "{{#" | "{{/" => {
                    c += 2;
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
                        _ => ParserState::Invalid
                    }
                },
                "}}}" => {
                    c += 2;
                    let mut t = template_stack.front_mut().unwrap();
                    t.elements.push(HTMLExpression(
                        try!(Parameter::parse(buffer.clone().trim_matches(' ').to_string()))));
                    buffer.clear();
                    ParserState::Text
                },
                _ => {
                    match if slice.len() > 2 { slice.slice_chars_alt(0, 2) } else { slice.as_ref() } {
                        "{{" => {
                            c += 1;
                            if !buffer.is_empty() {
                                let mut t = template_stack.front_mut().unwrap();
                                let buf_clone = process_whitespace(&buffer, &mut ws_omitter);
                                t.elements.push(RawString(buf_clone));
                                buffer.clear();
                            }
                            ParserState::Expression
                        },
                        "}}" => {
                            c += 1;
                            match state {
                                ParserState::Expression => {
                                    if !buffer.is_empty() {
                                        // {{else}} or {{^}} within a helper block
                                        if buffer.trim() == "else" || buffer.trim() == "^" {
                                            buffer.clear(); // drop else
                                            let t = template_stack.pop_front().unwrap();
                                            let h = helper_stack.front_mut().unwrap();
                                            h.template = Some(t);
                                            template_stack.push_front(Template{ elements: Vec::new() });
                                            ParserState::Text
                                        } else if buffer.contains(' ') {
                                            //inline helper
                                            let helper = try!(HelperTemplate::parse(buffer.clone(), false));
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

                                    } else {
                                        ParserState::Invalid
                                    }
                                },
                                ParserState::Comment => {
                                    let mut t = template_stack.front_mut().unwrap();
                                    t.elements.push(Comment(buffer.clone()));
                                    buffer.clear();
                                    ParserState::Text
                                },
                                ParserState::HelperStart => {
                                    let helper = try!(HelperTemplate::parse(buffer.clone(), true));
                                    helper_stack.push_front(helper);
                                    template_stack.push_front(Template{ elements: Vec::new() });

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
                                        ParserState::Invalid
                                    }
                                },
                                _ => ParserState::Invalid
                            }
                        },
                        _ => {
                            buffer.push(slice.chars().nth(0).unwrap());
                            state
                        }
                    }
                }
            };
            if state == ParserState::Invalid {
                return Err(TemplateError);
            }
            c += 1;
        }

        if !buffer.is_empty() {
            let mut t = template_stack.front_mut().unwrap();
            let buf_clone = process_whitespace(&buffer, &mut ws_omitter);
            t.elements.push(TemplateElement::RawString(buf_clone));
        }

        if !helper_stack.is_empty() {
            return Err(TemplateError);
        }

        return Ok(template_stack.pop_front().unwrap());
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
    let h = HelperTemplate::parse(source, true).ok().unwrap();

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
{{#if date}}<p>good</p>{{else}}<p>bad</p>{{/if}}<img>{{foo bar}}{{#unless true}}kitkat{{^}}lollipop{{/unless}}";
    let t = Template::compile(source.to_string()).ok().unwrap();

    assert_eq!(t.elements.len(), 9);
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

    match *t.elements.get(8).unwrap() {
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
    let source = "{{#ifequals name compare=\"hello\"}}hello{{else}}good";

    let t = Template::compile(source.to_string());

    assert!(t.is_err());
}

#[test]
fn test_subexpression() {
    let source = "{{foo (bar)}}";
    let t = Template::compile(source.to_string()).ok().unwrap();

    assert_eq!(t.elements.len(), 1);
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

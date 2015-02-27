use std::cmp::min;
use std::ops::BitOr;
use std::num::FromPrimitive;
use std::fmt::{self, Debug, Formatter};
use std::iter::IteratorExt;
use std::collections::{BTreeMap, VecDeque};
use std::string::ToString;
use serialize::json::Json;

use self::TemplateElement::{RawString, Expression, HelperExpression,
                            HTMLExpression, HelperBlock, Comment};

#[derive(PartialEq, Clone)]
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

#[derive(PartialEq, Clone)]
pub struct Helper {
    name: String,
    params: Vec<String>,
    hash: BTreeMap<String, Json>,
    template: Option<Template>,
    inverse: Option<Template>,
    block: bool
}

impl Helper {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn params(&self) -> &Vec<String> {
        &self.params
    }

    pub fn hash(&self) -> &BTreeMap<String, Json> {
        &self.hash
    }

    pub fn template(&self) -> Option<&Template> {
        match self.template {
            Some(ref t) => {
                Some(t)
            },
            None => None
        }
    }

    pub fn inverse(&self) -> Option<&Template> {
        match self.inverse {
            Some(ref t) => {
                Some(t)
            },
            None => None
        }
    }

    pub fn is_block(&self) -> bool {
        self.block
    }
}

impl ToString for Helper {
    fn to_string(&self) -> String {

        let mut buf = String::new();

        if self.block {
            buf.push_str(format!("{{{{#{}", self.name).as_slice());
        } else {
            buf.push_str(format!("{{{{{}", self.name).as_slice());
        }

        for p in self.params.iter() {
            buf.push_str(format!(" {}", p).as_slice());
        }

        for k in self.hash.keys() {
            buf.push_str(format!(" {}={}", k, self.hash.get(k).unwrap()).as_slice());
        }

        buf.push_str("}}");

        if self.block {
            let tpl = self.template();
            if tpl.is_some() {
                buf.push_str(tpl.unwrap().to_string().as_slice());
            }
            let ivs = self.inverse();
            if ivs.is_some() {
                buf.push_str("{{else}}");
                buf.push_str(ivs.unwrap().to_string().as_slice());
            }
            buf.push_str(format!("{{{{/{}}}}}", self.name).as_slice());
        }
        buf
    }
}

#[derive(Debug, Copy)]
pub struct TemplateError;

impl Helper {
    fn parse(source: String, block: bool) -> Result<Helper, TemplateError> {
        let mut tokens = source.split(|&: c: char| c.is_whitespace())
            .filter(|s: &&str| !(*s).is_empty());

        let name = tokens.next();
        match name {
            Some(n) => {
                let mut params: Vec<String> = Vec::new();
                let mut hash: BTreeMap<String, Json> = BTreeMap::new();

                for t in tokens {
                    if t.contains('=') {
                        let kv = t.split('=').collect::<Vec<&str>>();
                        hash.insert(kv.get(0).unwrap().to_string(),
                                    kv.get(1).unwrap().parse::<Json>().unwrap());
                    } else {
                        params.push(t.to_string());
                    }
                }

                Ok(Helper{
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

#[derive(PartialEq, FromPrimitive)]
enum WhiteSpaceOmit {
    Left = 0x01,
    Right = 0x10,
    Both = 0x11,
    None = 0x00
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
        let mut helper_stack: VecDeque<Helper> = VecDeque::new();
        let mut template_stack: VecDeque<Template> = VecDeque::new();
        template_stack.push_front(Template{ elements: Vec::new() });

        let mut buffer: String = String::new();
        let mut state = ParserState::Text;

        let mut c:usize = 0;
        let mut ws_omitter = WhiteSpaceOmit::None;
        let source_len = source.chars().count();
        while c < source_len {
            let mut slice = source.slice_chars(c, min(c+3, source_len)).to_string();
            if slice == "{{~" {
                ws_omitter = ws_omitter | WhiteSpaceOmit::Right;
                // read another char and remove ~
                slice = source.slice_chars(c, min(c+4, source_len)).to_string();
                slice.remove(2);
                c += 1;
            }
            if slice == "~}}" {
                ws_omitter = ws_omitter | WhiteSpaceOmit::Left;
                c += 1;
                slice = source.slice_chars(c, min(c+3, source_len)).to_string();
            }
            state = match slice.as_slice() {
                "{{{" | "{{!" | "{{#" | "{{/" => {
                    c += 2;
                    if !buffer.is_empty() {
                        let mut t = template_stack.front_mut().unwrap();
                        let buf_clone = process_whitespace(&buffer, &mut ws_omitter);
                        t.elements.push(RawString(buf_clone));
                        buffer.clear();
                    }
                    match slice.as_slice() {
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
                        buffer.clone().trim_matches(' ').to_string()));
                    buffer.clear();
                    ParserState::Text
                },
                _ => {
                    match if slice.len() > 2 { slice.slice_chars(0, 2) } else { slice.as_slice() } {
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
                                            match Helper::parse(buffer.clone(), false){
                                                Ok(helper) => {
                                                    let mut t = template_stack.front_mut().unwrap();
                                                    t.elements.push(HelperExpression(helper));
                                                    buffer.clear();
                                                    ParserState::Text
                                                },
                                                Err(_) => ParserState::Invalid
                                            }
                                        } else {
                                            let mut t = template_stack.front_mut().unwrap();
                                            t.elements.push(Expression(
                                                buffer.clone().trim_matches(' ').to_string()));
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
                                    match Helper::parse(buffer.clone(), true) {
                                        Ok(helper) => {
                                            helper_stack.push_front(helper);
                                            template_stack.push_front(Template{ elements: Vec::new() });

                                            buffer.clear();
                                            ParserState::Text
                                        },
                                        Err(_) => ParserState::Invalid
                                    }
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
                            buffer.push(slice.char_at(0));
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
            buf.push_str(v.to_string().as_slice());
        }
        buf
    }
}

#[derive(PartialEq, Clone)]
pub enum TemplateElement {
    RawString(String),
    Expression(String),
    HTMLExpression(String),
    HelperExpression(Helper),
    HelperBlock(Helper),
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

impl Debug for TemplateElement {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        try!(writeln!(f, "{:?}", self.to_string()));
        Ok(())
    }
}

#[test]
fn test_parse_helper_start_tag() {
    let source = "if not name compare=1".to_string();
    let h = Helper::parse(source, true).ok().unwrap();

    assert_eq!(h.name, "if".to_string());
    assert_eq!(h.params, vec::<String>!["not".to_string(), "name".to_string()]);

    let key = "compare".to_string();
    let value = h.hash.get(&key).unwrap();
    assert_eq!(value.as_u64().unwrap(), 1u64);
}

#[test]
fn test_parse_template() {
    let source = "<h1>{{title}} 你好</h1> {{{content}}}
{{#if date}}<p>good</p>{{else}}<p>bad</p>{{/if}}<img>{{foo bar}}{{#unless true}}kitkat{{^}}lollipop{{/unless}}";
    let t = Template::compile(source.to_string()).ok().unwrap();

    assert_eq!(t.elements.len(), 9);
    assert_eq!((*t.elements.get(0).unwrap()).to_string(), "<h1>".to_string());
    assert_eq!((*t.elements.get(1).unwrap()).to_string(),
               Expression("title".to_string()).to_string());

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
            assert_eq!(*(h.params.get(0).unwrap()), "bar".to_string());
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
fn test_white_space_omitter() {
    let source = "hello~     {{~world~}} \n  !{{~#if true}}else{{/if~}}".to_string();
    let t = Template::compile(source).ok().unwrap();

    assert_eq!(t.elements.len(), 4);

    assert_eq!(t.elements[0], RawString(String::from_str("hello~")));
    assert_eq!(t.elements[1], Expression(String::from_str("world")));
    assert_eq!(t.elements[2], RawString(String::from_str("!")));
}

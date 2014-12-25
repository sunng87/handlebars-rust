use std::cmp::min;
use std::iter::IteratorExt;
use std::collections::{HashMap, RingBuf};

use self::TemplateElement::{RawString, Expression, HelperExpression,
                            HTMLExpression, HelperBlock, Comment};

#[deriving(PartialEq, Show)]
pub struct Template {
    pub elements: Vec<TemplateElement>
}

#[deriving(PartialEq, Show)]
enum ParserState {
    Text,
    HtmlExpression,
    Comment,
    HelperStart,
    HelperEnd,
    Expression,
    Invalid
}

#[deriving(PartialEq, Show)]
pub struct Helper {
    name: String,
    params: Vec<String>,
    hash: HashMap<String, String>,
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

    pub fn hash(&self) -> &HashMap<String, String> {
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

#[deriving(Show, Copy)]
pub struct TemplateError;

impl Helper {
    fn parse(source: String, block: bool) -> Result<Helper, TemplateError> {
        let mut tokens = source.split(|&: c: char| c.is_whitespace())
            .filter(|s: &&str| !(*s).is_empty());

        let name = tokens.next();
        match name {
            Some(n) => {
                let mut params: Vec<String> = Vec::new();
                let mut hash: HashMap<String, String> = HashMap::new();

                for t in tokens {
                    if t.contains_char('=') {
                        let kv = t.split('=').collect::<Vec<&str>>();
                        hash.insert(kv.get(0).unwrap().to_string(),
                                    kv.get(1).unwrap().to_string());
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

impl Template {
    pub fn compile(source: String) -> Result<Template, TemplateError> {
        let mut helper_stack: RingBuf<Helper> = RingBuf::new();
        let mut template_stack: RingBuf<Template> = RingBuf::new();
        template_stack.push_front(Template{ elements: Vec::new() });

        let mut buffer: String = String::new();
        let mut state = ParserState::Text;

        let mut c:uint = 0;
        let source_len = source.chars().count();
        while c < source_len {
            let slice = source.slice_chars(c, min(c+3, source_len));

            state = match slice {
                "{{{" | "{{!" | "{{#" | "{{/" => {
                    c += 2;
                    if !buffer.is_empty() {
                        let mut t = template_stack.front_mut().unwrap();
                        t.elements.push(RawString(buffer.clone()));
                        buffer.clear();
                    }
                    match slice {
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
                        buffer.clone().trim_chars(' ').to_string()));
                    buffer.clear();
                    ParserState::Text
                },
                _ => {
                    match if slice.chars().count() > 2 { slice.slice_chars(0, 2) } else { slice } {
                        "{{" => {
                            c += 1;
                            if !buffer.is_empty() {
                                let mut t = template_stack.front_mut().unwrap();
                                t.elements.push(RawString(buffer.clone()));
                                buffer.clear();
                            }
                            ParserState::Expression
                        },
                        "}}" => {
                            c += 1;
                            match state {
                                ParserState::Expression => {
                                    if !buffer.is_empty() {
                                        // {{else}} within a helper block
                                        if buffer.trim_chars(' ') == "else" {
                                            buffer.clear(); // drop else
                                            let t = template_stack.pop_front().unwrap();
                                            let h = helper_stack.front_mut().unwrap();
                                            h.template = Some(t);
                                            template_stack.push_front(Template{ elements: Vec::new() });
                                            ParserState::Text
                                        } else if buffer.contains_char(' ') {
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
                                                buffer.clone().trim_chars(' ').to_string()));
                                            buffer.clear();
                                            ParserState::Text
                                        }

                                    } else {
                                        ParserState::Invalid
                                    }
                                },
                                ParserState::Comment => {
                                    let mut t = template_stack.front_mut().unwrap();
                                    t.elements.push(Comment);
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
                                    let name = buffer.trim_chars(' ').to_string();
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
                            buffer.push(source.char_at(c));
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
            t.elements.push(TemplateElement::RawString(buffer.clone()));
        }

        return Ok(template_stack.pop_front().unwrap());
    }
}

#[deriving(Show, PartialEq)]
pub enum TemplateElement {
    RawString(String),
    Expression(String),
    HTMLExpression(String),
    HelperExpression(Helper),
    HelperBlock(Helper),
    Comment,
}

#[test]
fn test_parse_helper_start_tag() {
    let source = "if not name compare=1".to_string();
    let h = Helper::parse(source, true).ok().unwrap();

    assert_eq!(h.name, "if".to_string());
    assert_eq!(h.params, vec::<String>!["not".to_string(), "name".to_string()]);

    let key = "compare".to_string();
    let value = h.hash.get(&key).unwrap();
    assert_eq!(*value, "1".to_string());
}

#[test]
fn test_parse_template() {
    let source = "<h1>{{title}}</h1> {{{content}}}
{{#if date}}<p>good</p>{{else}}<p>bad</p>{{/if}}<img>{{foo bar}}";
    let t = Template::compile(source.to_string()).ok().unwrap();

    assert_eq!(t.elements.len(), 8);
    assert_eq!(*t.elements.get(0).unwrap(), RawString("<h1>".to_string()));
    assert_eq!(*t.elements.get(1).unwrap(), Expression("title".to_string()));

    assert_eq!(*t.elements.get(3).unwrap(), HTMLExpression("content".to_string()));

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
}

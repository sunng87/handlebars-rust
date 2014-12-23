use serialize::json::{Json, ToJson};
use regex::Regex;
use std::iter::IteratorExt;
use std::collections::RingBuf;

pub struct Context {
    data: Json
}

static NULL_VALUE: Json = Json::Null;
static ARRAY_INDEX_MATCHER: Regex = regex!(r"\[\d+\]$");

impl Context {
    pub fn null() -> Context {
        Context {
            data: NULL_VALUE.clone()
        }
    }

    pub fn wraps<'a, T: ToJson>(e: &T) -> Context {
        Context {
            data: e.to_json()
        }
    }

    pub fn navigate(&self, path: &String, relative_path: &String) -> &Json {
        let mut path_stack :RingBuf<&str> = RingBuf::new();
        for p in (*path).split('/') {
            match p {
                "this" | "." | "" => {
                    continue;
                }
                ".." => {
                    path_stack.pop_back();
                }
                _ => {
                    for dot_p in p.split('.') {
                        match ARRAY_INDEX_MATCHER.find(dot_p) {
                            Some((s, _)) => {
                                let arr = dot_p.slice_to(s);
                                if arr != "this" {
                                    path_stack.push_back(dot_p.slice_to(s));
                                }
                                path_stack.push_back(dot_p.slice_from(s));
                            },
                            None => {
                                if dot_p != "this" {
                                    path_stack.push_back(dot_p);
                                }
                            }
                        }
                    }
                }
            }
        }

        for p in (*relative_path).split('/') {
            match p {
                "this" | "." | "" => {
                    continue;
                }
                ".." => {
                    path_stack.pop_back();
                }
                _ => {
                    for dot_p in p.split('.') {
                        match ARRAY_INDEX_MATCHER.find(dot_p) {
                            Some((s, _)) => {
                                let arr = dot_p.slice_to(s);
                                if arr != "this" {
                                    path_stack.push_back(dot_p.slice_to(s));
                                }
                                path_stack.push_back(dot_p.slice_from(s));
                            },
                            None => {
                                if dot_p != "this" {
                                    path_stack.push_back(dot_p);
                                }
                            }
                        }
                    }
                }
            }
        }

        let paths :Vec<&str> = path_stack.iter().map(|x| *x).collect();
        let mut data: &Json = &self.data;
        for p in paths.iter() {
            if ARRAY_INDEX_MATCHER.is_match(*p) {
                data = match *data {
                    Json::Array(ref a) => {
                        let index = p.slice_chars(1, p.len()-1);
//                        println!("========== {}", index);
                        let index_u: Option<uint> = from_str(index);
                        match index_u {
                            Some(i) => a.get(i).unwrap(),
                            None => &NULL_VALUE
                        }
                    },
                    _ => {
                        &NULL_VALUE
                    }
                }
            } else {
                data = match data.find(*p) {
                    Some(d) => d,
                    None => &NULL_VALUE
                }
            }
        }
        data
    }
}

pub trait JsonRender {
    fn render(&self) -> String;
}

impl JsonRender for Json {
    fn render(&self) -> String {
        match *self {
            Json::String(_) => {
                let s = format!("{}", *self);
                s.slice_chars(1, s.char_len()-1).to_string()
            },
            _ => {
                format!("{}", *self)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use context::{JsonRender, Context};
    use std::collections::BTreeMap;
    use serialize::json::{Json, ToJson};

    struct Address {
        city: String,
        country: String
    }

    impl ToJson for Address {
        fn to_json(&self) -> Json {
            let mut m = BTreeMap::new();
            m.insert("city".to_string(), self.city.to_json());
            m.insert("country".to_string(), self.country.to_json());
            Json::Object(m)
        }
    }

    struct Person {
        name: String,
        age: i16,
        addr: Address,
        titles: Vec<String>
    }

    impl ToJson for Person {
        fn to_json(&self) -> Json {
            let mut m = BTreeMap::new();
            m.insert("name".to_string(), self.name.to_json());
            m.insert("age".to_string(), self.age.to_json());
            m.insert("addr".to_string(), self.addr.to_json());
            m.insert("titles".to_string(), self.titles.to_json());
            Json::Object(m)
        }
    }

    #[test]
    fn test_render() {
        let v = "hello";
        let ctx = Context::wraps(&v.to_string());
        let this = "this".to_string();
        assert_eq!(ctx.navigate(&this, &this).render(), v.to_string());
    }

    #[test]
    fn test_navigation() {
        let addr = Address {
            city: "Beijing".to_string(),
            country: "China".to_string()
        };

        let person = Person {
            name: "Ning Sun".to_string(),
            age: 27,
            addr: addr,
            titles: vec!["programmer".to_string(),
                         "cartographier".to_string()]
        };

        let ctx = Context::wraps(&person);
        let this = "this".to_string();
        let that = "./name/../addr/country".to_string();

        assert_eq!(ctx.navigate(&this, &that).render(), "China".to_string());

        let v = true;
        let ctx2 = Context::wraps(&v);
        assert_eq!(ctx2.navigate(&"this".to_string(), &"this".to_string()).render(), "true".to_string());

        let this2 = "this".to_string();
        let that2 = "titles[0]".to_string();
        assert_eq!(ctx.navigate(&this2, &that2).render(), "programmer".to_string());
    }
}

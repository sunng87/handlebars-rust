use serialize::json::{Json, ToJson};
use regex::Regex;
use std::collections::VecDeque;

pub struct Context {
    data: Json,
    default: Json,
    array_index_matcher: Regex,
}

#[inline]
fn parse_json_visitor<'a>(path_stack: &mut VecDeque<&'a str>, path: &'a str) {
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
                    if dot_p != "this" {
                        path_stack.push_back(dot_p);
                    }
                }
            }
        }
    }
}

impl Context {
    pub fn null() -> Context {
        Context {
            data: Json::Null,
            default: Json::Null,
            array_index_matcher: Regex::new(r"\[\d+\]$").unwrap(),
        }
    }

    pub fn wraps<T: ToJson>(e: &T) -> Context {
        Context {
            data: e.to_json(),
            default: Json::Null,
            array_index_matcher: Regex::new(r"\[\d+\]$").unwrap(),
        }
    }

    pub fn navigate(&self, path: &str, relative_path: &str) -> &Json {
        let mut path_stack :VecDeque<&str> = VecDeque::new();
        parse_json_visitor(&mut path_stack, path);
        parse_json_visitor(&mut path_stack, relative_path);

        let paths :Vec<&str> = path_stack.iter().map(|x| *x).collect();
        let mut data: &Json = &self.data;
        for p in paths.iter() {
            match self.array_index_matcher.find(*p) {
                Some((s, _)) => {
                    let arr = &p[..s];
                    let idx = &p[s+1 .. p.len()-1];

                    let root = if arr == "this" {
                        Some(data)
                    } else {
                        data.find(arr)
                    };

                    data = match root {
                        Some(d) => {
                            if let Json::Array(ref l) = *d {
                                match idx.parse::<usize>() {
                                    Ok(idx_u) => l.get(idx_u).unwrap(),
                                    Err(_) => &self.default
                                }
                            } else {
                                &self.default
                            }
                        },
                        None => &self.default
                    };
                },
                None => {
                    data = match data.find(*p) {
                        Some(d) => d,
                        None => &self.default
                    };
                }
            }
        }
        data
    }
}

pub trait JsonRender {
    fn render(&self) -> String;
}

pub trait JsonTruthy {
    fn is_truthy(&self) -> bool;
}

impl JsonRender for Json {
    fn render(&self) -> String {
        if let Json::String(_) = *self {
            self.as_string().unwrap_or("").to_string()
        } else {
            format!("{}", *self)
        }
    }
}

impl JsonTruthy for Json {
    fn is_truthy(&self) -> bool {
        match *self {
            Json::I64(i) => i != 0,
            Json::U64(i) => i != 0,
            Json::F64(i) => i != 0.0 || ! i.is_nan(),
            Json::Boolean (ref i) => *i,
            Json::Null => false,
            Json::String (ref i) => i.len() > 0,
            Json::Array (ref i) => i.len() > 0,
            Json::Object (ref i) => i.len() > 0
        }
    }
}

#[cfg(test)]
mod test {
    use context::{JsonRender, Context};
    use std::collections::BTreeMap;
    use serialize::json::{Json, ToJson};

    #[test]
    fn test_json_render() {
        let raw = "<p>Hello world</p>\n<p thing=\"hello\"</p>";
        let thing = Json::String(raw.to_string());

        assert_eq!(raw, thing.render());
    }

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

use serialize::json::{Json, ToJson};
use std::iter::IteratorExt;
use std::collections::RingBuf;

pub struct Context {
    data: Json
}

static NULL_VALUE: Json = Json::Null;

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
                "this" | "." => {
                    continue;
                }
                ".." => {
                    path_stack.pop_back();
                }
                _ => {
                    for dot_p in p.split('.') {
                        if p != "this" {
                            path_stack.push_back(p)
                        }
                    }
                }
            }
        }

        for p in (*relative_path).split('/') {
            match p {
                "this" | "." => {
                    continue;
                }
                ".." => {
                    path_stack.pop_back();
                }
                _ => {
                    for dot_p in p.split('.') {
                        if p != "this" {
                            path_stack.push_back(p)
                        }
                    }
                }
            }
        }

        let paths :Vec<&str> = path_stack.iter().map(|x| *x).collect();
        match self.data.find_path(paths.as_slice()){
            Some(j) => j,
            None => &NULL_VALUE
        }
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
        addr: Address
    }

    impl ToJson for Person {
        fn to_json(&self) -> Json {
            let mut m = BTreeMap::new();
            m.insert("name".to_string(), self.name.to_json());
            m.insert("age".to_string(), self.age.to_json());
            m.insert("addr".to_string(), self.addr.to_json());
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
            addr: addr
        };

        let ctx = Context::wraps(&person);
        let this = "this".to_string();
        let that = "./name/../addr/country".to_string();

        assert_eq!(ctx.navigate(&this, &that).render(), "China".to_string());

        let v = true;
        let ctx2 = Context::wraps(&v);
        assert_eq!(ctx2.navigate(&"this".to_string(), &"this".to_string()).render(), "true".to_string());
    }
}

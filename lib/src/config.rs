#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ValueRef<'a> {
    pub names: &'a [&'a str],
    pub def: &'a str,
}

pub trait Config {
    fn get<'s>(&'s self, names: &[&str], def: &'s str) -> &'s str;

    fn get_ref<'s>(&'s self, vref: &ValueRef<'s>) -> &'s str {
        self.get(vref.names, vref.def)
    }
}

pub mod map {
    use std::collections::HashMap;

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub enum Entry {
        Value(String),
        Section(HashMap<String, Entry>),
    }

    impl Entry {
        fn get<'s>(&'s self, names: &[&str], def: &'s str) -> &'s str {
            match names.split_first() {
                Some((first_name, other_names)) => match self {
                    Entry::Value(_) => def,
                    Entry::Section(section) => section
                        .get(&first_name.to_ascii_lowercase().to_string())
                        .map_or(def, |entry| entry.get(other_names, def))
                },
                None => match self {
                    Entry::Value(value) => value,
                    Entry::Section(_) => def,
                },
            }
        }
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct Config {
        cfg: Entry,
    }

    impl super::Config for Config {
        fn get<'s>(&'s self, names: &[&str], def: &'s str) -> &'s str {
            self.cfg.get(names, def)
        }
    }

    fn normalise(entry: &Entry) -> Entry {
        match entry {
            Entry::Value(v) => Entry::Value(v.to_owned()),
            Entry::Section(m) => {
                let m: HashMap<String, Entry> = m.iter()
                    .map(|(k, v)| (k.to_lowercase(), normalise(v)))
                    .collect();
                Entry::Section(m)
            }
        }
    }

    pub fn new(cfg: HashMap<String, Entry>) -> Config {
        Config { cfg: normalise(&Entry::Section(cfg)) }
    }
}

pub mod env {
    use std::collections::HashMap;

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct Config {
        env: HashMap<String, String>,
    }

    impl super::Config for Config {
        fn get<'s>(&'s self, names: &[&str], def: &'s str) -> &'s str {
            let mapped_names: Vec<String> = names.iter().map(|name| {
                name.to_ascii_uppercase().replace("-", "_")
            }).collect();
            let env_name = "DUNSUMDAY_".to_owned() + &mapped_names.join("_");

            // self.env.get(&env_name).map(|s| s.to_str()).unwrap_or(&def)
            match self.env.get(&env_name) {
                Some(v) => v,
                None => &def,
            }
        }
    }

    pub fn new() -> Config {
        let mut env = HashMap::new();
        for (name_os, val_os) in std::env::vars_os() {
            match (name_os.into_string(), val_os.into_string()) {
                (Ok(name), Ok(val)) => {
                    env.insert(name, val);
                },
                _ => (),
            }
        }
        Config { env }
    }
}

pub mod file {
    use std::{fs::File, path::Path};
    use super::map::{self, Entry};
    use serde_yaml::Value;

    fn parse(value: &Value) -> Entry {
        match value {
            Value::Null => Entry::Value("".to_owned()),
            Value::Bool(b) => Entry::Value(b.to_string()),
            Value::Number(n) => Entry::Value(n.to_string()),
            Value::String(s) => Entry::Value(s.to_owned()),
            Value::Sequence(s) => {
                Entry::Section(s.iter()
                    .enumerate()
                    .map(|(i, v)| (i.to_string(), parse(v)))
                    .collect())
            }
            Value::Mapping(m) => {
                Entry::Section(m.iter()
                    .filter(|(k, v)| k.is_string())
                    .map(|(k, v)| {
                        k.as_str()
                            .map(|k_str| (k_str.to_owned(), parse(v)))
                    })
                    .flatten()
                    .collect())
            }
            Value::Tagged(_) => Entry::Value("".to_owned())
        }
    }

    pub fn new<P>(path: P) -> Result<map::Config, String>
    where
        P: AsRef<Path> + core::fmt::Debug
    {
        let file = File::open(path.as_ref())
            .map_err(|e| format!("error opening file ({path:?}): {e}"))?;
        let value: Value = serde_yaml::from_reader(file)
            .map_err(|e| format!(
                "error loading config from file ({path:?}): {e}"))?;
        let entry = parse(&value);
        if let Entry::Section(e) = entry {
            Ok(map::new(e))
        } else {
            Err("invalid config file: top-level must be a map".to_owned())
        }
    }
}

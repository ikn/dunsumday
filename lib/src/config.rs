//! Simple, general-purpose, hierarchical configuration.
//!
//! Configuration *value*s are generally referred to using a *path* of *name*s
//! (slice of strings), each of which walks a level down the hierarchy of
//! *section*s.  For example, `&["interface", "colours", "background"]`.
//!
//! Configuration paths are case-insensitive.
//!
//! A [`Config`] implementation may or may not allow a value and a section to
//! exist at the same path.
//!
//! All configuration values are strings.

pub mod parse;
pub mod validate;

pub trait ValueParser<T>: std::fmt::Debug {
    fn parse(&self, value: &str) -> Result<T, String>;
}

pub trait ValueValidator<T>: std::fmt::Debug {
    fn validate(&self, value: &T) -> Result<(), String>;
}

/// Everything needed to read a configuration value.
#[derive(Clone, Debug)]
pub struct ValueRef<'a, T> {
    /// Path to read the value from.
    pub names: &'a [&'a str],
    /// Default to use when there is no value at the path.
    pub def: &'a str,
    pub type_: &'a dyn ValueParser<T>,
    pub validators: Vec<&'a dyn ValueValidator<T>>,
}

/// Read configuration values.
pub trait Config {
    /// Get the value at the path given by `names`, or the default `def`.
    fn get<'s>(&'s self, names: &[&str], def: &'s str) -> &'s str;
}

/// Get a value using a [reference](ValueRef).
pub fn get_ref<C, T>(config: &C, vref: &ValueRef<T>) -> Result<T, String>
where
    C: Config + ?Sized,
{
    let raw = config.get(vref.names, vref.def);
    let parsed = vref.type_.parse(raw)?;
    for val in &vref.validators {
        val.validate(&parsed)?;
    }
    Ok(parsed)
}

/// Implementation of [`Config`] using an in-memory map.
///
/// A value and a section may not exist at the same path.
///
/// When multiple values have equivalent paths (because paths are
/// case-insensitive), reading the value at the path will always return the same
/// value, but there is no defined scheme for how this value is chosen.
pub mod map {
    use std::collections::HashMap;

    /// A value or a section.
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

    /// Implementation of [`Config`](super::Config) using an in-memory map.
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct Config {
        cfg: Entry,
    }

    impl super::Config for Config {
        fn get<'s>(&'s self, names: &[&str], def: &'s str) -> &'s str {
            self.cfg.get(names, def)
        }
    }

    /// Copy an entry and lowercase its keys.
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

    /// Construct a config from a hierarchical map.
    pub fn new(cfg: HashMap<String, Entry>) -> impl super::Config {
        Config { cfg: normalise(&Entry::Section(cfg)) }
    }
}

/// Implementation of [`Config`] using the process's environment variables.
///
/// - The configuration values become fixed at the time of construction.
/// - If reading an environment variable fails, it is ignored.
/// - Path names are separated using `_` characters.
/// - Only uppercase environment variables are included.
/// - A value and a section may exist at the same path.
/// - When reading a value, `-` characters in path names will match `_`
///   characters in environment variable names.
pub mod env {
    use std::collections::HashMap;

    /// Implementation of [`Config`](super::Config) using the process's
    /// environment variables.
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct Config {
        prefix: String,
        env: HashMap<String, String>,
    }

    impl super::Config for Config {
        fn get<'s>(&'s self, names: &[&str], def: &'s str) -> &'s str {
            let mapped_names: Vec<String> = names.iter().map(|name| {
                name.to_ascii_uppercase().replace('-', "_")
            }).collect();
            let env_name = self.prefix.to_owned() + &mapped_names.join("_");

            match self.env.get(&env_name) {
                Some(v) => v,
                None => def,
            }
        }
    }

    /// Construct a config from the current process environment.
    ///
    /// Only environment variables starting with `prefix` are included, and
    /// `prefix` is removed when reading values.
    pub fn new(prefix: String) -> impl super::Config {
        let mut env = HashMap::new();
        for (name_os, val_os) in std::env::vars_os() {
            if let (Ok(name), Ok(val)) =
                (name_os.into_string(), val_os.into_string())
            {
                env.insert(name, val);
            }
        }
        Config { prefix, env }
    }
}

/// Implementation of [`Config`] using a YAML file.
///
/// A value and a section may not exist at the same path.
///
/// When multiple values have equivalent paths (because paths are
/// case-insensitive), the last matching value in the file is returned.
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
                    .flat_map(|(k, v)| {
                        k.as_str()
                            .map(|k_str| (k_str.to_owned(), parse(v)))
                    })
                    .collect())
            }
            Value::Tagged(_) => Entry::Value("".to_owned())
        }
    }

    /// Construct a config from a YAML file.
    pub fn new<P>(path: P) -> Result<impl super::Config, String>
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

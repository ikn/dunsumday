use std::path::PathBuf;
use shellexpand;
use super::ValueParser;

#[derive(Clone, Debug)]
pub struct StringParser { }

impl ValueParser<String> for self::StringParser {
    fn parse(&self, value: &str) -> Result<String, String> {
        Ok(value.to_owned())
    }
}

pub const STRING: StringParser = StringParser {};

#[derive(Clone, Debug)]
pub struct BoolParser { }

impl ValueParser<bool> for self::BoolParser {
    fn parse(&self, value: &str) -> Result<bool, String> {
        value.parse::<bool>()
            .map_err(|e| format!("invalid boolean value: {value}"))
    }
}

pub const BOOL: BoolParser = BoolParser {};

#[derive(Clone, Debug)]
pub struct FilePathParser { }

impl ValueParser<PathBuf> for self::FilePathParser {
    fn parse(&self, value: &str) -> Result<PathBuf, String> {
        Ok(PathBuf::from(shellexpand::tilde(value).into_owned()))
    }
}

pub const FILE_PATH: FilePathParser = FilePathParser {};

#[derive(Clone, Debug)]
pub struct WebPortParser { }

impl ValueParser<u16> for self::WebPortParser {
    fn parse(&self, value: &str) -> Result<u16, String> {
        value.parse::<u16>()
            .map_err(|e| format!("invalid port number: {value}"))
    }
}

pub const WEB_PORT: WebPortParser = WebPortParser {};

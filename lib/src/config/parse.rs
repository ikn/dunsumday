use super::ValueParser;

#[derive(Clone, Debug)]
pub struct StringParser { }

impl ValueParser<String> for self::StringParser {
    fn parse(&self, value: &str) -> Result<String, String> {
        Ok(value.to_owned())
    }
}

pub static STRING: StringParser = StringParser {};

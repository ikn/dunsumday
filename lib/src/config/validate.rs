use super::ValueValidator;

#[derive(Clone, Debug)]
pub struct WebPathValidator { }

impl ValueValidator<String> for self::WebPathValidator {
    fn validate(&self, path: &String) -> Result<(), String> {
        if !path.starts_with("/") {
            Err(format!("path must start with / character: {path}"))
        } else {
            Ok(())
        }
    }
}

pub const WEB_PATH: &WebPathValidator = &WebPathValidator {};

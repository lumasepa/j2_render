use std::error::Error;
use std::fmt::{Formatter, Display, Error as FmtError};
use std::any::Any;

#[derive(Debug)]
pub struct WrapError {
    pub description: String,
}

impl WrapError {
    pub fn new<T: Display>(description: &str, inner: &T) -> Self {
        let mut description = description.to_string();
        description.push('\n');
        description += &format!("-> Caused By : {}", inner);
        WrapError {description: description}
    }
    pub fn new_none(description: &str) -> Self {
        let mut description = description.to_string();
        description.push('\n');
        description += "-> Caused By : None";
        WrapError {description: description}
    }
    pub fn new_first(description: &str)  -> Self {
        WrapError {description: format!("-> Caused By : {}", description)}
    }
}

impl Display for WrapError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "{}", self.description)
    }
}

impl Error for WrapError {}

pub trait ToWrapErrorResult<T> {
    fn wrap(self, description: &str) -> Result<T, WrapError>;
}

impl<T, E> ToWrapErrorResult<T> for Result<T, E> where E : Display, {
    fn wrap(self, description: &str) -> Result<T, WrapError> {
        self.map_err(|err|{
            WrapError::new(description, &err)
        })
    }
}

impl<T> ToWrapErrorResult<T> for Option<T> {
    fn wrap(self, description: &str) -> Result<T, WrapError> {
        self.ok_or(WrapError::new_none(description))
    }
}
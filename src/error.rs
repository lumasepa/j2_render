use std::any::Any;
use std::error::Error;
use std::fmt::{Display, Error as FmtError, Formatter};

#[macro_use]
mod macros {
    macro_rules! wrap_err {
        ($fstr:literal, $($es:expr),*) => {
            WrapError::new_first(&format!($fstr, $($es,)*))
        };
    }
    macro_rules! wrap_result {
        ($fstr:literal, $($es:expr),*) => {
            Err(WrapError::new_first(&format!($fstr, $($es,)*)))
        };
        ($err:expr, $fstr:literal, $($es:expr),*) => {
            $err.wrap(&format!($fstr, $($es,)*))
        };
    }
}

#[derive(Debug)]
pub struct WrapError {
    pub description: String,
}

impl WrapError {
    pub fn new<T: Display>(description: &str, inner: &T) -> Self {
        let mut description = description.to_string();
        description.push('\n');
        let tab_inner = format!("{}", inner).replace("\n", "\n    ");
        let tab_inner = tab_inner.trim_end();
        let tab_inner = "    ".to_owned() + tab_inner;
        description += &format!("-> Caused By : \n{}", tab_inner);
        WrapError {
            description: description,
        }
    }
    pub fn new_none(description: &str) -> Self {
        let mut description = description.to_string();
        description.push('\n');
        description += "-> Caused By : \n    None\n";
        WrapError {
            description: description,
        }
    }
    pub fn new_first(description: &str) -> Self {
        WrapError {
            description: format!("-> Caused By : {}", description),
        }
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

impl<T, E> ToWrapErrorResult<T> for Result<T, E>
where
    E: Display,
{
    fn wrap(self, description: &str) -> Result<T, WrapError> {
        self.map_err(|err| WrapError::new(description, &err))
    }
}

impl<T> ToWrapErrorResult<T> for Option<T> {
    fn wrap(self, description: &str) -> Result<T, WrapError> {
        self.ok_or(WrapError::new_none(description))
    }
}

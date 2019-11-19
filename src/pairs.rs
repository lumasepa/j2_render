use crate::error::{ToWrapErrorResult, WrapError};
use std::fmt::{Display, Error as FmtError, Formatter};
use std::slice::Iter;

#[derive(Debug)]
pub struct Pairs {
    inner: Vec<(String, String)>,
}

impl Display for Pairs {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        for (k, v) in self.inner.iter() {
            write!(f, "{}={}\n", k, v)?;
        }
        Ok(())
    }
}

impl Pairs {
    pub fn new() -> Pairs {
        Pairs { inner: vec![] }
    }

    pub fn iter(&self) -> Iter<(String, String)> {
        self.inner.iter()
    }

    pub fn insert(&mut self, k: String, v: String) {
        self.inner.push((k, v))
    }

    pub fn get<T: PartialEq<String>>(&self, key: T) -> Option<String> {
        self.inner
            .iter()
            .rev()
            .find(|(k, _)| key == *k)
            .map(|(_, v)| v.to_owned())
    }

    pub fn try_from_args(args: &mut Vec<String>) -> Result<Pairs, WrapError> {
        let mut pairs = Pairs::new();
        while let Some(arg) = args.pop() {
            if arg.starts_with("-") {
                args.push(arg);
                break;
            }
            let (key, value) = wrap_result!(Self::parse_pair(&arg), "Error parsing pair {}", arg)?;
            pairs.insert(key, value);
        }
        Ok(pairs)
    }

    fn parse_pair(pair: &str) -> Result<(String, String), WrapError> {
        let mut pair_v: Vec<&str> = pair.splitn(2, '=').collect();
        let value = wrap_result!(pair_v.pop(), "Expected key=value pattern, value not found : {}", pair)?;
        let key = wrap_result!(pair_v.pop(), "Expected key=value pattern, key not found : {}", pair)?;

        Ok((key.to_string(), value.to_string()))
    }
}

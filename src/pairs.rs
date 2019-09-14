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

    pub fn get_couples<T: PartialEq<String> + Display>(&self, first: T, second: T) -> Vec<(String, Option<String>)> {
        let mut couples = vec![];
        let mut iter = self.inner.iter();
        while let Some((key_first, value_first)) = iter.next() {
            if first == *key_first {
                if let Some((key_second, value_second)) = iter.next() {
                    if second == *key_second {
                        couples.push((value_first.to_owned(), Some(value_second.to_owned())))
                    } else {
                        couples.push((value_first.to_owned(), None))
                    }
                } else {
                    couples.push((value_first.to_owned(), None))
                }
            }
        }
        return couples;
    }

    pub fn remove_all(&mut self, key: String) {
        let to_remove: Vec<usize> = self
            .inner
            .iter()
            .enumerate()
            .filter(|(i, (k, v))| *k == key)
            .map(|(i, (k, v))| i)
            .collect();

        for id in to_remove {
            self.inner.remove(id);
        }
    }

    pub fn remove_first(&mut self, key: String) {
        let to_remove = self
            .inner
            .iter()
            .enumerate()
            .find(|(i, (k, v))| *k == key)
            .map(|(id, (k, v))| id);

        if let Some(id) = to_remove {
            self.inner.remove(id);
        }
    }

    pub fn try_from_args(args: &mut Vec<String>) -> Result<Pairs, WrapError> {
        let mut pairs = Pairs::new();
        while let Some(arg) = args.pop() {
            if arg.starts_with("-") {
                args.push(arg);
                break;
            }
            let (key, value) = Self::parse_pair(&arg).wrap(&format!("Error parsing pair {}", arg))?;
            pairs.insert(key, value);
        }
        Ok(pairs)
    }

    fn parse_pair(pair: &str) -> Result<(String, String), WrapError> {
        let mut pair: Vec<&str> = pair.splitn(2, '=').collect();
        let value = pair.pop().wrap("Expected key=value pattern, key not found")?;
        let key = pair.pop().wrap("Expected key=value pattern, value not found")?;
        Ok((key.to_string(), value.to_string()))
    }

    pub fn is_input(&self) -> bool {
        self.get("source").or(self.get("s")).is_some()
    }

    pub fn is_output(&self) -> bool {
        self.get("destination").or(self.get("d")).is_some()
    }
}

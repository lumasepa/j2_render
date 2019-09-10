use crate::pairs::Pairs;
use crate::error::{WrapError, ToWrapErrorResult};

#[derive(Debug)]
pub enum Source {
    File { path: String },
    StdIn,
    Env { key: Option<String> },
    Value { value: String },
    Http { url: String },
    K8s { resource: String, uri: String }
}

impl Source {
    pub fn try_from_pairs(pairs: &Pairs) -> Result<Self, WrapError> {
        let source = pairs.get("source").or(pairs.get("s")).wrap("Expected source in input")?;
        let source = match source.as_ref() {
            "file" => Source::File{
                path: pairs.get("file").wrap("Expected file path in source file")?.to_owned()
            },
            "stdin" => Source::StdIn,
            "env" => Source::Env{key: pairs.get("key").map(|k| k.to_owned())},
            "var" => Source::Value {value: pairs.get("value").wrap("Error value not found for source var")?.to_owned()},
            "http" => panic!("Not implemented yet!"),
            "k8s" => panic!("Not implemented yet!"),
            _ => return Err(WrapError::new_first(&format!("source {} not recognized", source))),
        };
        return Ok(source)
    }
}

use crate::pairs::Pairs;
use crate::error::{WrapError, ToWrapErrorResult};
use std::path::Path;
use std::ffi::OsStr;

#[derive(Debug)]
pub enum Destination {
    File { path: String, format: Option<String> },
    StdOut,
    Http { url: String },
    K8s { resource: String, uri: String }
}

impl Destination {
    pub fn try_from_pairs(pairs: &Pairs) -> Result<Self, WrapError> {
        let destination = pairs.get("destination").or(pairs.get("d")).unwrap_or("stdout".to_string());
        let destination = match destination.as_ref() {
            "file" => Destination::File{
                path: pairs.get("file").wrap("Expected file path in out file")?.to_owned(),
                format: Path::new(&pairs.get("file").unwrap()).extension().and_then(OsStr::to_str).and_then(|f| Some(f.to_string()))
            },
            "stdout" => Destination::StdOut,
            "http" => panic!("Not implemented yet!"),
            "k8s" => panic!("Not implemented yet!"),
            _ => return Err(WrapError::new_first(&format!("destination {} not recognized", destination))),
        };
        return Ok(destination)
    }
}

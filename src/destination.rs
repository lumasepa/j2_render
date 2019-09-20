#[macro_use]
use crate::error::{ToWrapErrorResult, WrapError};
use crate::pairs::Pairs;
use std::ffi::OsStr;
use std::path::Path;

#[derive(Debug)]
pub enum Destination {
    File {
        path: String,
        format: Option<String>,
    },
    StdOut,
    Http {
        method: String,
        url: String,
        headers: Vec<(String, String)>,
    },
    K8s {
        resource: String,
        uri: String,
    },
}

impl Destination {
    pub fn try_from_pairs(pairs: &Pairs) -> Result<Self, WrapError> {
        let destination = pairs
            .get("destination")
            .or(pairs.get("d"))
            .unwrap_or("stdout".to_string());
        let destination = match destination.as_ref() {
            "file" => Destination::File {
                path: wrap_result!(pairs.get("file"), "Expected file path in out file {}", pairs)?.to_owned(),
                format: Path::new(&pairs.get("file").unwrap())
                    .extension()
                    .and_then(OsStr::to_str)
                    .and_then(|f| Some(f.to_string())),
            },
            "std" => Destination::StdOut,
            "http" => Destination::Http {
                method: pairs.get("method").or(Some("GET".to_string())).unwrap(),
                url: wrap_result!(
                    pairs.get("url"),
                    "Error 'url' not found for destination=http : {}",
                    pairs
                )?,
                headers: pairs
                    .get_couples("header", "value")
                    .iter()
                    .filter_map(|(k, v)| {
                        if let Some(v) = v {
                            return Some((k.to_owned(), v.to_owned()));
                        }
                        None
                    })
                    .collect(),
            },
            "k8s" => panic!("Not implemented yet!"),
            _ => {
                return Err(WrapError::new_first(&format!(
                    "destination {} not recognized",
                    destination
                )))
            }
        };
        return Ok(destination);
    }
}

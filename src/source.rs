use crate::error::{ToWrapErrorResult, WrapError};
use crate::pairs::Pairs;
use std::collections::HashMap;
use std::io::Read;
use std::{env, fs, io};
use tera::Context;

#[derive(Debug)]
pub enum Source {
    File { path: String },
    StdIn,
    Env { key: Option<String> },
    Var { value: String },
    Http { url: String },
    K8s { resource: String, uri: String },
}

impl Source {
    pub fn try_from_pairs(pairs: &Pairs) -> Result<Self, WrapError> {
        let source = pairs
            .get("source")
            .or(pairs.get("s"))
            .wrap("Expected source in input")?;
        let source = match source.as_ref() {
            "file" => Source::File {
                path: pairs.get("file").wrap("Expected file path in source file")?.to_owned(),
            },
            "stdin" => Source::StdIn,
            "env" => Source::Env {
                key: pairs.get("key").map(|k| k.to_owned()),
            },
            "var" => Source::Var {
                value: pairs
                    .get("value")
                    .wrap("Error value not found for source var")?
                    .to_owned(),
            },
            "http" => panic!("Not implemented yet!"),
            "k8s" => panic!("Not implemented yet!"),
            _ => return Err(WrapError::new_first(&format!("source {} not recognized", source))),
        };
        return Ok(source);
    }

    pub fn get_content(&self) -> Result<String, WrapError> {
        let data = match self {
            Source::File { path } => {
                let data = fs::read_to_string(&path).wrap(&format!("Error reading file {}", path))?;
                data
            }
            Source::StdIn => {
                let mut data = String::new();
                io::stdin().read_to_string(&mut data).wrap("Error reading from stdin")?;
                data
            }
            Source::Env { key } => {
                let env_vars = env::vars().collect::<HashMap<String, String>>();
                let mut ctx = Context::new();
                if let Some(key) = key {
                    let value = env_vars.get(key)
                        .map(|key| key.to_owned())
                        .unwrap_or_else(|| {
                            eprintln!("Env var {} not found", key);
                            String::new()
                        });
                    ctx.insert(key, &value);
                } else {
                    for (k, v) in env_vars {
                        ctx.insert(&k, &v);
                    }
                };
                ctx.as_json()
                    .wrap("Error transforming env to json")?
                    .to_string()
            }
            Source::Var { value } => value.to_owned(),
            Source::Http { .. } => panic!(""),
            Source::K8s { .. } => panic!(""),
        };
        Ok(data)
    }
}

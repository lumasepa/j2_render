use crate::error::{ToWrapErrorResult, WrapError};
use crate::j2::tera::tera_render;
use crate::pairs::Pairs;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::io::Read;
use std::path::Path;
use std::{env, fs, io};
use tera::Context;

#[derive(Debug, Clone)]
pub enum Source {
    File { path: String },
    StdIn,
    Env { key: Option<String> },
    Var { value: String },
}

impl Source {
    pub fn try_from_pairs(pairs: &Pairs) -> Result<Self, WrapError> {
        let source = pairs
            .get("source")
            .or(pairs.get("s"))
            .wrap_err("Expected source in input")?;
        let source: Source = match source.as_ref() {
            "file" => Source::File {
                path: pairs.get("file").wrap_err("Expected file path in source=file")?,
            },
            "stdin" => Source::StdIn,
            "env" => Source::Env {
                key: pairs.get("key").map(|k| k.to_owned()),
            },
            "var" => Source::Var {
                value: pairs.get("value").wrap_err("Error 'value' not found for source=var")?,
            },
            _ => return Err(WrapError::new_first(&format!("source '{}' not recognized", source))),
        };
        return Ok(source);
    }

    pub fn try_get_format(&self) -> Option<String> {
        match self {
            Source::File { path } => Path::new(&path)
                .extension()
                .and_then(OsStr::to_str)
                .map(|ext| ext.to_owned()),
            Source::StdIn => None,
            Source::Env { .. } => Some("json".to_string()),
            Source::Var { .. } => Some("string".to_string()), // default
        }
    }

    pub fn get_content(&self) -> Result<String, WrapError> {
        let data = match self {
            Source::File { path } => {
                let data = wrap_result!(fs::read_to_string(&path), "Error reading file {}", path)?;
                data
            }
            Source::StdIn => {
                let mut data = String::new();
                io::stdin()
                    .read_to_string(&mut data)
                    .wrap_err("Error reading from stdin")?;
                data
            }
            Source::Env { key } => {
                let env_vars = env::vars().collect::<HashMap<String, String>>();
                let mut ctx = Context::new();
                if let Some(key) = key {
                    let value = env_vars.get(key).map(|key| key.to_owned()).unwrap_or_else(|| {
                        eprintln!("Env var {} not found", key);
                        String::new()
                    });
                    ctx.insert(key, &value);
                } else {
                    for (k, v) in env_vars {
                        ctx.insert(&k, &v);
                    }
                };
                ctx.as_json().wrap_err("Error transforming env to json")?.to_string()
            }
            Source::Var { value } => value.to_owned(),
        };
        Ok(data)
    }

    pub fn render(&self, ctx: &Context) -> Result<Source, WrapError> {
        let source = match self {
            Source::File { path } => Source::File {
                path: tera_render(path.to_owned(), ctx),
            },
            Source::StdIn => Source::StdIn,
            Source::Env { key } => Source::Env {
                key: key.as_ref().map(|key| tera_render(key.to_owned(), ctx)),
            },
            Source::Var { value } => Source::Var {
                value: tera_render(value.to_owned(), ctx),
            },
        };
        Ok(source)
    }
}

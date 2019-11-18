use crate::error::{ToWrapErrorResult, WrapError};
use crate::j2::tera::tera_render;
use crate::pairs::Pairs;
use reqwest::header::HeaderMap;
use reqwest::Url;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::io::Read;
use std::path::Path;
use std::{env, fs, io};
use tera::Context;

#[derive(Debug, Clone)]
pub enum Source {
    File {
        path: String,
    },
    StdIn,
    Env {
        key: Option<String>,
    },
    Var {
        value: String,
    },
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

impl Source {
    pub fn try_from_pairs(pairs: &Pairs) -> Result<Self, WrapError> {
        let source = pairs
            .get("source")
            .or(pairs.get("s"))
            .wrap("Expected source in input")?;
        let source: Source = match source.as_ref() {
            "file" => Source::File {
                path: pairs.get("file").wrap("Expected file path in source=file")?,
            },
            "stdin" => Source::StdIn,
            "env" => Source::Env {
                key: pairs.get("key").map(|k| k.to_owned()),
            },
            "var" => Source::Var {
                value: pairs.get("value").wrap("Error 'value' not found for source=var")?,
            },
            "http" => Source::Http {
                method: pairs.get("method").or(Some("GET".to_string())).unwrap(),
                url: pairs.get("url").wrap("Error 'url' not found for source=http")?,
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
            Source::Http { url, .. } => Url::parse(url).ok().map(|url| url.path().to_string()).and_then(|path| {
                Path::new(&path)
                    .extension()
                    .and_then(OsStr::to_str)
                    .map(|ext| ext.to_owned())
            }),
            Source::K8s { .. } => panic!(""),
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
                io::stdin().read_to_string(&mut data).wrap("Error reading from stdin")?;
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
                ctx.as_json().wrap("Error transforming env to json")?.to_string()
            }
            Source::Var { value } => value.to_owned(),
            Source::Http {
                method,
                url,
                headers: _,
            } => {
                let client = reqwest::Client::new();
                let request = match method.as_ref() {
                    "GET" => client.get(url),
                    "POST" => client.post(url),
                    "PUT" => client.put(url),
                    "DELETE" => client.delete(url),
                    _ => panic!(),
                };
                let mut _headers = HeaderMap::new();
                //                for (k, v) in headers {
                //                    _headers.insert(k, v);
                //                }
                let _response = request.headers(_headers).send();

                panic!()
            }
            Source::K8s { .. } => panic!(""),
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
            Source::Http { method, url, headers } => Source::Http {
                method: tera_render(method.to_owned(), ctx),
                url: tera_render(url.to_owned(), ctx),
                headers: headers
                    .iter()
                    .map(|(k, v)| (tera_render(k.to_owned(), ctx), tera_render(v.to_owned(), ctx)))
                    .collect(),
            },
            Source::K8s { .. } => panic!(""),
        };
        Ok(source)
    }
}

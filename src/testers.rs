use std::path::Path;
use tera::{Result, Value};

pub fn is_file(value: Option<&Value>, _: &[Value]) -> Result<bool> {
    if let Some(Value::String(path)) = value {
        let path = Path::new(&path);
        return Ok(path.is_file());
    } else {
        return Err("is_file: Invalid type, expected string".into());
    }
}

pub fn is_directory(value: Option<&Value>, _: &[Value]) -> Result<bool> {
    if let Some(Value::String(path)) = value {
        let path = Path::new(&path);
        return Ok(path.is_dir());
    } else {
        return Err("is_directory: Invalid type, expected string".into());
    }
}

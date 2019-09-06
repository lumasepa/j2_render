use crate::inners::exec_cmd;
use base64;
use glob::glob;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use tera::{Result, Value};

pub fn bash(piped_arg: Value, args: HashMap<String, Value>) -> Result<Value> {
    let data = if let Value::String(data) = piped_arg {
        data
    } else {
        return Err("bash: Invalid type, expected string".into());
    };

    let command = if let Some(Value::String(command)) = args.get("command") {
        command
    } else {
        return Err("bash: Invalid type for arg command, expected string".into());
    };

    let mut bash_cmd = Command::new("bash");

    bash_cmd
        .env("__data", data)
        .arg("-c")
        .arg(format!("echo \"$__data\" | {}", command));

    return exec_cmd(&mut bash_cmd, command, &args);
}

pub fn sed(piped_arg: Value, args: HashMap<String, Value>) -> Result<Value> {
    let data = if let Value::String(data) = piped_arg {
        data
    } else {
        return Err("sed: Invalid type, expected string".into());
    };

    let expr = if let Some(Value::String(expr)) = args.get("expression") {
        expr
    } else {
        return Err("sed: Invalid type for arg command, expected string".into());
    };

    let mut bash_cmd = Command::new("bash");
    let command = format!("sed -e \"{}\"", expr);
    bash_cmd
        .env("__data", data)
        .arg("-c")
        .arg(format!("echo \"$__data\" | {}", command));

    return exec_cmd(&mut bash_cmd, &command, &args);
}

pub fn file_glob(piped_arg: Value, _: HashMap<String, Value>) -> Result<Value> {
    let mut files_matched = vec![];

    if let Value::String(path) = piped_arg {
        let paths = match glob(&path) {
            Ok(paths) => paths,
            Err(e) => return Err(format!("file_glob: error in glob : {:?}", e).into()),
        };
        for entry in paths {
            match entry {
                Ok(path) => files_matched.push(Value::String(format!("{}", path.display()))),
                Err(e) => println!("{:?}", e),
            }
        }
        return Ok(Value::Array(files_matched));
    } else {
        return Err("file_glob: Invalid type, expected string".into());
    }
}

pub fn read_file(piped_arg: Value, _: HashMap<String, Value>) -> Result<Value> {
    if let Value::String(path) = piped_arg {
        match fs::read_to_string(path) {
            Ok(contents) => return Ok(Value::String(contents)),
            Err(e) => return Err(format!("read_file: error reading file : {:?}", e).into()),
        }
    } else {
        return Err("read_file: Invalid type, expected string".into());
    }
}

pub fn file_name(piped_arg: Value, _: HashMap<String, Value>) -> Result<Value> {
    if let Value::String(path) = piped_arg {
        let path = Path::new(&path);
        let file_name = match path.file_name() {
            Some(file_name) => file_name,
            None => return Err(format!("file_name: error extracting filename : path is root, no filename").into()),
        };

        match file_name.to_str() {
            Some(file_name) => return Ok(Value::String(file_name.to_string())),
            None => return Err(format!("file_name: error decoding filename").into()),
        }
    } else {
        return Err("file_name: Invalid type, expected string".into());
    }
}

pub fn file_dir(piped_arg: Value, _: HashMap<String, Value>) -> Result<Value> {
    if let Value::String(path) = piped_arg {
        let path = Path::new(&path);
        let file_name = match path.parent() {
            Some(file_name) => file_name,
            None => return Err(format!("file_dir: error extracting filename : path is root, no filename").into()),
        };

        match file_name.to_str() {
            Some(file_name) => return Ok(Value::String(file_name.to_string())),
            None => return Err(format!("file_dir: error decoding filename").into()),
        }
    } else {
        return Err("file_dir: Invalid type, expected string".into());
    }
}

pub fn strip_line_breaks(piped_arg: Value, _: HashMap<String, Value>) -> Result<Value> {
    if let Value::String(lines) = piped_arg {
        return Ok(Value::String(lines.replace("\n", "")));
    } else {
        return Err("strip_line_breaks: Invalid type, expected string".into());
    }
}

pub fn remove_extension(piped_arg: Value, _: HashMap<String, Value>) -> Result<Value> {
    if let Value::String(filename) = piped_arg {
        let mut parts: Vec<_> = filename.split('.').collect();
        parts.pop();
        let filename = parts.join(".");
        return Ok(Value::String(filename));
    } else {
        return Err("remove_extension: Invalid type, expected string".into());
    }
}

pub fn b64encode(piped_arg: Value, _: HashMap<String, Value>) -> Result<Value> {
    if let Value::String(data) = piped_arg {
        let encoded_data = base64::encode(&data);
        return Ok(Value::String(encoded_data));
    } else {
        return Err("b64encode: Invalid type, expected string".into());
    }
}

pub fn b64decode(piped_arg: Value, _: HashMap<String, Value>) -> Result<Value> {
    if let Value::String(data) = piped_arg {
        return base64::decode(&data)
            .map_err(|e| format!("b64decode: decoding error : {}", e).into())
            .and_then(|decoded_data| {
                String::from_utf8(decoded_data)
                    .map(Value::String)
                    .map_err(|e| format!("b64decode: utf8 decoding error : {}", e).into())
            });
    } else {
        return Err("b64decode: Invalid type, expected string".into());
    }
}

pub fn str(piped_arg: Value, _: HashMap<String, Value>) -> Result<Value> {
    return Ok(Value::String(piped_arg.to_string()));
}

pub fn from_json(piped_arg: Value, _: HashMap<String, Value>) -> Result<Value> {
    if let Value::String(data) = piped_arg {
        let value = data
            .parse::<serde_json::Value>()
            .expect(&format!("from_json: error parsing json : {}", data));
        return Ok(value);
    } else {
        return Err("from_json: Invalid type, expected string".into());
    }
}

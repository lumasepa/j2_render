use crate::inners::exec_cmd;
use std::collections::HashMap;
use std::process::Command;
use tera::{Error, Result, Value};

pub fn bash(args: HashMap<String, Value>) -> Result<Value> {
    let command = if let Some(Value::String(command)) = args.get("command") {
        command
    } else {
        return Err("bash: Invalid type for arg command, expected string".into());
    };

    let mut bash_cmd = Command::new("bash");
    bash_cmd.arg("-c").arg(command);

    return exec_cmd(&mut bash_cmd, command, &args);
}

pub fn tab_all_lines(args: HashMap<String, Value>) -> Result<Value> {
    if let Some(Value::String(lines)) = args.get("lines") {
        if let Some(Value::Number(num_spaces)) = args.get("num_spaces") {
            let num_spaces = num_spaces.as_u64().ok_or(Error::from(
                "tab_all_lines: Error number of spaces is not unsigned integer",
            ))? as usize;
            let spaces = " ".repeat(num_spaces);
            let lines: Vec<_> = lines.split('\n').map(|line| spaces.clone() + line).collect();
            return Ok(Value::String(lines.join("\n")));
        } else {
            return Err("tab_all_lines: Invalid type for arg num_spaces, expected number".into());
        }
    } else {
        return Err("tab_all_lines: Invalid type for arg lines, expected string".into());
    }
}

pub fn str(args: HashMap<String, Value>) -> Result<Value> {
    return Ok(Value::String(
        args.get("value").expect("str: expected value argument").to_string(),
    ));
}

pub fn from_json(args: HashMap<String, Value>) -> Result<Value> {
    let value = args
        .get("value")
        .expect("from_json: expected value argument")
        .to_string();
    let value = value
        .parse::<serde_json::Value>()
        .expect(&format!("from_json: error parsing json : {}", value));
    return Ok(value);
}

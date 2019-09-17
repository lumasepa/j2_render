use crate::error::ToWrapErrorResult;
use crate::j2::inners::exec_cmd;
use crate::j2::tera::tera_render;
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use tera::{Context, Error, Result, Value};

pub fn render(args: HashMap<String, Value>) -> Result<Value> {
    let path = if let Some(Value::String(path)) = args.get("path") {
        path
    } else {
        return Err("render: Invalid type for arg path, expected string".into());
    };
    let ctx = if let Some(ctx) = args.get("ctx") {
        ctx
    } else {
        return Err("render: arg ctx not found".into());
    };
    let template = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(e) => return Err(format!("render: error reading file : {:?}", e).into()),
    };
    let mut template_ctx = Context::new();
    template_ctx.insert("ctx", ctx);

    let result = tera_render(template, &template_ctx);
    return Ok(Value::String(result));
}

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

pub fn jmespath(args: HashMap<String, Value>) -> Result<Value> {
    let path = if let Some(Value::String(path)) = args.get("path") {
        path
    } else {
        return Err("jmespath: Invalid type for arg path, expected string".into());
    };
    let ctx = if let Some(Value::Object(ctx)) = args.get("ctx") {
        ctx
    } else {
        return Err("jmespath: Invalid type for arg path, expected string".into());
    };
    todo!()
    //    use serde_json::Error;
    //    let expr = jmespath::compile(&path).wrap(&format!("Error parsing jmespath : {}", path)).into()?;
    //    let result = expr.search(&ctx).wrap(&format!("Error evaluating jmespath : {}", path)).into()?;
    //    return Ok(result)
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

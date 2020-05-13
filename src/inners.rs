use std::collections::HashMap;
use std::process::Command;
use tera::{Result, Value};
use anyhow::Context;

pub fn exec_cmd(command: &mut Command, cmd_str: &str, env: &HashMap<String, Value>) -> Result<Value> {
    for (k, v) in env.iter() {
        let value = if let Value::String(data) = v {
            data
        } else {
            return Err("Invalid type for args, expected string".into());
        };
        command.env(k, value);
    }
    let out = command
        .output()
        .context(format!("Error executing command : {}", cmd_str))
        .map_err(|e| e.to_string())?;

    let stdout = String::from_utf8(out.stdout)
        .context(format!("bash: Error reading stdout of command {}", cmd_str))
        .map_err(|e| e.to_string())?;
    let stderr = String::from_utf8(out.stderr)
        .context(format!("bash: Error reading stderr of command {}", cmd_str))
        .map_err(|e| e.to_string())?;
    if stderr != "" {
        eprintln!("command {} stderr : {}", cmd_str, stderr);
    }

    return Ok(Value::String(stdout));
}

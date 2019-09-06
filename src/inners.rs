use std::collections::HashMap;
use std::process::Command;
use tera::{Result, Value};

pub fn exec_cmd(command: &mut Command, cmd_str: &String, env: &HashMap<String, Value>) -> Result<Value> {
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
        .expect(&format!("Error executing command : {}", cmd_str));

    let stdout = String::from_utf8(out.stdout).expect(&format!("bash: Error reading stdout of command {}", cmd_str));
    let stderr = String::from_utf8(out.stderr).expect(&format!("bash: Error reading stderr of command {}", cmd_str));
    if stderr != "" {
        eprintln!("command {} stderr : {}", cmd_str, stderr);
    }

    return Ok(Value::String(stdout));
}

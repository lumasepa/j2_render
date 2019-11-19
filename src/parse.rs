use crate::error::{ToWrapErrorResult, WrapError};
use crate::help;
use crate::input::RawInput;
use crate::pairs::Pairs;
use crate::source::Source;
use molysite::hcl::parse_hcl;
use serde_json::{Map, Value};

use std::process::exit;

macro_rules! get_value {
    ($data:ident, $typ:path) => {
        match $data {
            $typ(data) => data,
            _ => Err(WrapError::new_first(&format!("Error parsing hcl of ops file")))?,
        }
    };
}

pub fn parse_pairs(pairs_objects: Vec<Pairs>) -> Result<Vec<RawInput>, WrapError> {
    let mut inputs = vec![];
    for pairs in pairs_objects {
        let input = RawInput::try_from_pairs(pairs).wrap_err("Error parsing input pairs")?;
        inputs.push(input);
    }
    return Ok(inputs);
}

pub fn parse_input_ops(input: Map<String, Value>) -> Result<Pairs, WrapError> {
    let mut pairs = Pairs::new();
    for (k, v) in input.iter() {
        pairs.insert("source".to_string(), k.to_owned());
        let body: &Vec<Value> = get_value!(v, Value::Array);
        let body = body.get(0).wrap_err("Error parsing hcl of ops file")?;
        let body: &Map<String, Value> = get_value!(body, Value::Object);
        for (k, v) in body {
            let value: &String = get_value!(v, Value::String);
            pairs.insert(k.to_owned(), value.to_owned());
        }
    }

    return Ok(pairs);
}

pub fn parse_ops_file(path: String) -> Result<Vec<Pairs>, WrapError> {
    let file = Source::File { path: path.clone() };
    let ops = file.get_content()?;
    let json_ast = parse_hcl(&ops);
    let json_ast = wrap_result!(json_ast.as_ref(), "Error parsing hcl of ops file {}", path)?;
    let json_ast = wrap_result!(
        json_ast.to_string().parse::<serde_json::Value>(),
        "Error parsing to json the hcl of ops file {}",
        path
    )?;

    let root = get_value!(json_ast, Value::Object);

    let mut pairs_objs: Vec<Pairs> = vec![];

    todo!();

    let inputs: Vec<Value> = if let Some(inputs) = root.get("in") {
        get_value!(inputs, Value::Array).to_owned()
    } else {
        vec![]
    };

    for input in inputs {
        let input: Map<String, Value> = get_value!(input, Value::Object);
        let input_pairs = parse_input_ops(input)?;
        pairs_objs.push(input_pairs);
    }

    Ok(pairs_objs)
}

pub fn parse_args(mut args: &mut Vec<String>) -> Result<(Vec<Pairs>, Option<String>), WrapError> {
    args.reverse();
    args.pop(); // binary name

    let mut parsed_args = vec![];
    let mut output_path = None;

    while let Some(arg) = args.pop() {
        match arg.as_str() {
            "--ops" => {
                let path = args
                    .pop()
                    .wrap_err("Error specified --ops flag but not file path provided")?;
                let mut pairs_objs = parse_ops_file(path)?;
                parsed_args.append(&mut pairs_objs);
            }
            "--var" | "-v" => {
                let variable = args
                    .pop()
                    .wrap_err("error specified --var/-v flag but not value provided")?;
                let mut parts: Vec<&str> = variable.splitn(2, '=').collect();
                let value = parts.pop().wrap_err("Error no key=value found")?;
                let key = parts.pop().wrap_err("Error no key=value found")?;
                args.push("source=var".to_string());
                args.push(format!("value={}", value));
                args.push(format!("as={}", key));
                parsed_args.push(Pairs::try_from_args(&mut args)?);
            }
            "--out" | "-o" => {
                let path = args
                    .pop()
                    .wrap_err("Error specified --out flag but not file path provided")?;
                output_path = Some(path)
            }
            "--in" => parsed_args.push(Pairs::try_from_args(&mut args)?),
            "--stdin" | "-i" => {
                args.push(format!("source=stdin"));
                parsed_args.push(Pairs::try_from_args(&mut args)?);
            }
            "--file" | "-f" => {
                let path = args
                    .pop()
                    .wrap_err("error specified --file/-f flag but not context file path provided")?;

                args.push(format!("source=file"));
                args.push(format!("file={}", path));
                parsed_args.push(Pairs::try_from_args(&mut args)?);
            }
            "--env" | "-e" => {
                if let Some(variable_name) = args.pop() {
                    if variable_name.starts_with("-") {
                        args.push(arg);
                    } else {
                        args.push(format!("key={}", variable_name));
                    }
                }
                args.push("source=env".to_string());
                parsed_args.push(Pairs::try_from_args(&mut args)?);
            }
            "--help" | "help" | "-h" => {
                let topic = args.pop();
                help(topic);
                exit(0);
            }
            _ => wrap_result!("Error argument {} not recognized", arg)?,
        };
    }

    return Ok((parsed_args, output_path));
}

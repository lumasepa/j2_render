use crate::source::Source;
use crate::error::{WrapError, ToWrapErrorResult};
use crate::pairs::Pairs;
use crate::destination::Destination;
use std::process::exit;
use crate::help;
use std::path::Path;
use std::ffi::OsStr;

#[derive(Debug)]
pub struct Pick {
    name: Option<String>,
    path: String,
    namespace: Option<String>,
}

#[derive(Debug)]
pub struct Output {
    destination: Destination,
    format: Option<String>,
}

#[derive(Debug)]
pub struct Input {
    source: Source,
    format: String,
    picks: Option<Vec<Pick>>,
    namespace: Option<String>,
}

pub fn get_source_content(source: Source) -> Result<String, WrapError> {
    panic!()
}

pub fn parse_input(pairs: Pairs) -> Result<Input, WrapError> {
    let source = Source::try_from_pairs(&pairs).wrap("Error parsing source from pairs")?;
    let format = pairs.get("format").or(pairs.get("f")).wrap("Expected format=")?;
    let path = pairs.get("path").or(pairs.get("p"));
    let namespace = pairs.get("namespace").or(pairs.get("n"));
    let name = pairs.get("as");
    let picks = path.map(|path| {
        let mut picks = vec![];
        let pick = Pick {
            name,
            path,
            namespace: namespace.clone(),
        };
        picks.push(pick);
        picks
    });
    return Ok(Input {
        source,
        picks,
        namespace,
        format,
    });
}

pub fn parse_output(pairs: Pairs) -> Result<Output, WrapError> {
    let destination = Destination::try_from_pairs(&pairs).wrap("Error parsing source from pairs")?;
    let format = pairs.get("format").or(pairs.get("f"));
    return Ok(Output { destination, format });
}

pub fn parse_pairs(pairs_objects: Vec<Pairs>) -> Result<(Vec<Input>, Vec<Output>), WrapError> {
    let mut inputs = vec![];
    let mut outputs = vec![];
    for pairs in pairs_objects {
        if pairs.is_input() {
            let input = parse_input(pairs).wrap("Error parsing input pairs")?;
            inputs.push(input);
        } else if pairs.is_output() {
            let output = parse_output(pairs).wrap("Error parsing output pairs")?;
            outputs.push(output);
        } else {
            return Err(WrapError::new_first("Error: pairs object without source or destination"))
        }
    }
    return Ok((inputs, outputs))
}

pub fn parse_args(mut args: &mut Vec<String>) -> Result<Vec<Pairs>, WrapError> {
    args.reverse();
    args.pop(); // binary name

    let mut parsed_args = vec![];

    while let Some(arg) = args.pop() {
        let pairs = match arg.as_str() {
            "--var" | "-v" => {
                let variable = args
                    .pop()
                    .wrap("error specified --var/-v flag but not value provided")?;
                let mut parts: Vec<&str> = variable.splitn(2, '=').collect();
                let value = parts.pop().wrap("Error no key=value found")?;
                let key = parts.pop().wrap("Error no key=value found")?;
                args.push("source=var".to_string());
                args.push(format!("value={}", value));
                args.push(format!("namespace={}", key));
                Pairs::try_from_args(&mut args)?
            }
            "--out" | "-o" => Pairs::try_from_args(&mut args)?,
            "--in"  => Pairs::try_from_args(&mut args)?,
            "--stdin" | "-i" => {
                args.push(format!("source=stdin"));
                Pairs::try_from_args(&mut args)?
            }
            "--file" | "-f" => {
                let path = args
                    .pop()
                    .wrap("error specified --file/-f flag but not context file path provided")?;

                let extension = Path::new(&path)
                    .extension()
                    .and_then(OsStr::to_str);

                if let Some(extension) = extension {
                    args.push(format!("format={}", extension));
                }

                args.push(format!("source=file"));
                args.push(format!("file={}", path));
                Pairs::try_from_args(&mut args)?
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
                args.push("format=string".to_string());
                Pairs::try_from_args(&mut args)?
            }
            "--help" | "help" | "-h" => {
                help();
                exit(0);
            }
            _ => panic!("Error argument {} not recognized", arg),
        };
        parsed_args.push(pairs);
    }

    return Ok(parsed_args);
}
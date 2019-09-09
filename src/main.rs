#![feature(or_patterns)]

use molysite::hcl::parse_hcl;
use serde_json;
use serde_yaml;
use tera::{Context, Tera};
use regex::Regex;

use std::ffi::OsStr;
use std::io::Write;
use std::process::exit;
use std::{
    collections::HashMap,
    env, fs,
    io::{self, Read},
    path::Path,
};

mod filters;
mod functions;
mod inners;
mod testers;
mod error;

use crate::error::{ToWrapErrorResult, WrapError};

pub struct Config {
    pub template: Option<String>,
    pub context: Context,
    pub output: Option<Output>,
}

struct Pick {
    name: Option<String>,
    path: String,
    namespace: Option<String>
}

struct Output {
    format: Option<String>,
    filepath: Option<String>
}

struct Input {
    format: String,
    picks: Option<Vec<Pick>>,
    namespace: Option<String>,
    content: String,
}

pub fn parse_input_manipulation(mut args: &mut Vec<String>) -> Input {
    while let Some(arg) = args.pop() {
        if arg.starts_with("-") {
            args.push(arg);
        } else {

        }
    }
    panic!()
}

pub fn parse_output_manipulation(&mut args: &mut Vec<String>) -> Option<Output> {
    panic!()
}


pub fn parse_args_2() -> Result<Config, WrapError> {
    let mut args = env::args().collect::<Vec<String>>();
    args.reverse();

    let mut config = Config{
        template: None,
        context: Context::new(),
        output: None,
    };

    let mut inputs : Vec<Input> = vec![];

    args.pop(); // binary name

    while let Some(arg) = args.pop() {
        match arg.as_str() {
            "--var" | "-v" => {
                let variable = args
                    .pop()
                    .wrap("error specified --var/-v flag but not value provided")?;
                let mut parts : Vec<&str> = variable.splitn(2, '=').collect();
                let key = parts.pop().expect("Error no key=value found");
                let value = parts.pop().expect("Error no key=value found");
                args.push(format!("value={}", value));
                args.push(format!("key={}", key));
                let input = parse_input_manipulation(&mut args);
                inputs.push(input);
            }
            "--out" | "-o" => {
                config.output = parse_output_manipulation(&mut args);
            }
            "--stdin" | "-i" => {
                let mut value = String::new();
                io::stdin().read_to_string(&mut value).expect("Error readinf from stdin");
                args.push(format!("value={}", value));
                let input = parse_input_manipulation(&mut args);
                inputs.push(input);
            }
            "--file" | "-f" => {
                let path = args
                    .pop()
                    .expect("error specified --file/-f flag but not context file path provided");

                let extension = Path::new(&path)
                    .extension()
                    .and_then(OsStr::to_str)
                    .expect("Error no extension found in ctx file");

                let value = fs::read_to_string(&path).expect(&format!("Error reading context file {}", path));

                args.push(format!("value={}", value));
                args.push(format!("format={}", extension));
                let input = parse_input_manipulation(&mut args);
                inputs.push(input);
            }
            "--env" | "-e" => {
                if let Some(variable_name) =  args.pop(){
                    let value = env::vars()
                        .collect::<HashMap<String, String>>()
                        .get(&variable_name).wrap("Env var expected not found")?;
                    args.push(format!("value={}", value));
                } else {
                    args.push("value=__env__".to_string());
                }

                let input = parse_input_manipulation(&mut args);
                inputs.push(input);
            }
            "--help" | "help" | "-h" => {
                help();
                exit(0);
            }
            _ => panic!("Error argument {} not recognized", arg),
        }
    }

    return Ok(config);
}

pub fn help() {
   println!(
        "
j2_render [FLAGS]

    OPTIONS:
        SOURCES = file=file_path,stdin,env,value=value,http=url <- auth?
        DESTINATIONS = http=url,file=file_path,stdout,env
        FORMATS = json5,json,yaml,hcl,tfvars,tf,template,j2,tpl
        JMES_PATH = jmespath expression -- http://jmespath.org/tutorial.html

        INPUT_MANIPULATION = source=SOURCE format|f=FORMATS namespace|n=key.key.key path|p=JMES_PATH as=key
        OUTPUT_MANIPULATION = format|f=FORMATS file=filepath

    FLAGS:

    Inputs :
    --input [INPUT_MANIPULATION]
    --file/-f file_path [INPUT_MANIPULATION]    source=file=file_path format=file_extension
    --stdin/-i [INPUT_MANIPULATION]             source=stdin
    --env/-e                                    source=env
    --env/-e env_var_name [INPUT_MANIPULATION]  source=value=env_var_value namespace=env_var_name
    --var/-v key=value [INPUT_MANIPULATION]     source=value=value namespace=key

    Output:
    --out/-o OUTPUT_MANIPULATION

    --help/-h   -- shows this help
    "
    )
}

fn extract_format(string: &str) -> Option<(String,String)> {
    let mut parts : Vec<&str>= string.splitn(2, '+').collect();
    let format = parts.pop().expect("");
    if parts.len() == 0  {
        return None
    }
    let other = parts.pop().expect("");
    return Some((format.to_string(), other.to_string()))
}

pub fn parse_args() -> Config {
    let mut args = env::args().collect::<Vec<String>>();
    args.reverse();

    let mut config = Config{
        template: None,
        context: Context::new(),
        output: None,
    };

    args.pop(); // binary name

    while let Some(arg) = args.pop() {
        match arg.as_str() {
            "--var" | "-v" => {
                let variable = args
                    .pop()
                    .expect("error specified --var/-v flag but not value provided");
                let mut parts : Vec<&str> = variable.splitn(2, '=').collect();
                let key = parts.pop().expect("Error no key=value found");
                let value = parts.pop().expect("Error no key=value found");

                if let Some((format, key)) = extract_format(key) {
                    process_inputs(&mut config, format, value.to_string());
                } else {
                    config.context.insert(&key, &value)
                }
            }
            "--out" | "-o" => {
                let filepath = args
                    .pop()
                    .expect("error specified --out/-o flag but not file path provided");
                config.out_file = Some(filepath);
            }
            "--stdin" | "-i" => {
                let format = args
                    .pop()
                    .expect("error specified --stdin/-i flag but not format provided");
                let mut data = String::new();
                io::stdin().read_to_string(&mut data).expect("Error readinf from stdin");
                process_inputs(&mut config, format, data);
            }
            "--file" | "-f" => {
                let path = args
                    .pop()
                    .expect("error specified --file/-f flag but not context file path provided");
                let (format, path) = if let Some((format, path)) = extract_format(&path) {
                    (format, path)
                } else {
                    let extension = Path::new(&path)
                        .extension()
                        .and_then(OsStr::to_str)
                        .expect("Error no extension found in ctx file");
                    (extension.to_string(), path)
                };

                let data = fs::read_to_string(&path).expect(&format!("Error reading context file {}", path));
                process_inputs(&mut config, format, data);
            }
            "--env" | "-e" => {
                let env_vars = env::vars().collect::<HashMap<String, String>>();
                for (k, v) in env_vars.iter() {
                    config.context.insert(k, v);
                }
            }
            "--help" | "help" | "-h" => {
                help();
                exit(0);
            }
            _ => panic!("Error argument {} not recognized", arg),
        }
    }
    return config;
}

pub fn process_inputs(mut config: &mut Config, format: String, data: String) {
    if format == "template" || format == "tpl" || format == "j2" {
        config.template = Some(data)
    } else {
        populate_ctx(&mut config.context, format, data);
    }
}

pub fn populate_ctx(context: &mut Context, format: String, data: String) {
    match format.as_ref() {
        "yaml" | "yml" => {
            let value: serde_yaml::Value = serde_yaml::from_str(&data).expect("Error parsing yaml");
            let object = value.as_mapping().expect("Error expected object in root of yaml file");
            for (k, v) in object.iter() {
                let k = k.as_str().expect("Error decoding key of yaml, key is not a string");
                context.insert(k, v);
            }
        }
        "json" => {
            let value = data.parse::<serde_json::Value>().expect("Error parsing json");
            let object = value.as_object().expect("Error expected object in root of json file");
            for (k, v) in object.iter() {
                context.insert(k, v);
            }
        }
        "toml" | "tml" => {
            let value = data.parse::<toml::Value>().expect("Error parsing toml");
            let object = value.as_table().expect("Error expected object in root of toml file");
            for (k, v) in object.iter() {
                context.insert(k, v);
            }
        }
        "hcl" | "tfvars" | "tf" => {
            let value = parse_hcl(&data).expect("Error parsing hcl/tf/tfvars");
            let value = value.to_string().parse::<serde_json::Value>().expect("Error parsing json of hcl/tf/tfvars");

            let object = value
                .as_object()
                .expect("Error expected object in root of hcl/tf/tfvars file");
            for (k, v) in object.iter() {
                context.insert(k, v);
            }
        }
        _ => panic!("Format {} not recognized", format),
    }
}

pub fn tera_render(template: String, context: Context) -> String {
    let mut tera = Tera::default();
    tera.add_raw_template("template", &template)
        .expect("Error loading template in engine");

    tera.register_filter("bash", filters::bash);
    tera.register_filter("sed", filters::sed);
    tera.register_filter("glob", filters::file_glob);
    tera.register_filter("read_file", filters::read_file);
    tera.register_filter("file_name", filters::file_name);
    tera.register_filter("file_dir", filters::file_dir);
    tera.register_filter("strip_line_breaks", filters::strip_line_breaks);
    tera.register_filter("remove_extension", filters::remove_extension);
    tera.register_filter("b64decode", filters::b64decode);
    tera.register_filter("b64encode", filters::b64encode);
    tera.register_filter("str", filters::str);
    tera.register_filter("to_json", filters::str);
    tera.register_filter("from_json", filters::from_json);

    tera.register_function("tab_all_lines", Box::new(functions::tab_all_lines));
    tera.register_function("bash", Box::new(functions::bash));
    tera.register_function("str", Box::new(functions::str));
    tera.register_function("to_json", Box::new(functions::str));
    tera.register_function("from_json", Box::new(functions::from_json));

    tera.register_tester("file", testers::is_file);
    tera.register_tester("directory", testers::is_directory);

    tera.render("template", &context).expect("Error rendering template")
}

pub fn main() -> std::result::Result<(), String> {
    let Config{template, context, out_file } = parse_args();

    let rendered = if let Some(template) = template {
        tera_render(template, context)
    }else {
        serde_yaml::to_string(&context);
        context.as_json().expect("Error encoding ctx as json").to_string()
    };

    if let Some(filepath) = out_file {
        let mut file = fs::File::create(&filepath).expect("Error creating output file");
        file.write_all(rendered.as_ref()).expect("Error writing to output file");
    } else {
        io::stdout()
            .write_all(rendered.as_ref())
            .expect("Error writing to stdout");
    }

    Ok(())
}

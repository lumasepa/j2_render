#![feature(or_patterns)]

use molysite::hcl::parse_hcl;
use serde_json;
use serde_yaml;
use std::ffi::OsStr;
use std::io::Write;
use std::process::exit;
use std::{
    collections::HashMap,
    env, fs,
    io::{self, Read},
    path::Path,
};
use tera::{Context, Tera};

mod filters;
mod functions;
mod inners;
mod testers;

pub struct Config {
    pub template: String,
    pub context: Context,
    pub out_file: Option<String>,
    pub print_ctx: bool,
}

pub fn help() {
    println!(
        "
j2_render [FLAGS]

    OPTIONS:

        FORMATS = json,yaml,hcl,tfvars,tf,template,j2,tpl
        FILE_PATH = file_path.FORMAT or FORMAT+file_path   -- path to a file ctx, template or output
        VAR = key=value or FORMAT+key=value   -- if format provided value will be parsed as format

    FLAGS:

    --stdin/-i FORMAT   -- read from stdin context or template
    --out/-o file_path   -- output file for rendered template, default stdout
    --env/-e    -- load env vars in ctx
    --file/-f FILE_PATH   -- loads a file as context or template depending on extension or format
    --var/-v VAR   -- adds a pair key value to the context or a template depending on format
    --print-ctx/-p   -- print the context as json and exits
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
        template: String::new(),
        context: Context::new(),
        out_file: None,
        print_ctx: false
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
            "--print-ctx" | "-p" => config.print_ctx = true,
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
        config.template = data
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

pub fn main() -> std::result::Result<(), String> {
    let Config{template, context,print_ctx, out_file } = parse_args();

    if print_ctx {
        println!("{}", context.as_json().expect("Error encoding ctx as json").to_string());
        exit(0)
    }

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

    let rendered = tera.render("template", &context).expect("Error rendering template");

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

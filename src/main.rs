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
use anyhow::{Result, Context as AnyhowContext, anyhow};

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

    VAR: key[+FORMAT]=value
    FORMAT: yaml yml json toml tml hcl tfvars tf
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

pub fn parse_args() -> Result<Config> {
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
                    .ok_or(anyhow!("error specified --var/-v flag but not value provided"))?;
                let mut parts : Vec<&str> = variable.splitn(2, '=').collect();
                let key = parts.pop().ok_or(anyhow!("Error no key=value found"))?;
                let value = parts.pop().ok_or(anyhow!("Error no key=value found"))?;

                if let Some((format, key)) = extract_format(key) {
                    process_inputs(&mut config, format, value.to_string()).context("Error processing inputs from --var arg")?;
                } else {
                    config.context.insert(key, &value)
                }
            }
            "--print-ctx" | "-p" => config.print_ctx = true,
            "--out" | "-o" => {
                let filepath = args
                    .pop()
                    .ok_or(anyhow!("error specified --out/-o flag but not file path provided"))?;
                config.out_file = Some(filepath);
            }
            "--stdin" | "-i" => {
                let format = args
                    .pop()
                    .ok_or(anyhow!("error specified --stdin/-i flag but not format provided"))?;
                let mut data = String::new();
                io::stdin().read_to_string(&mut data).context("Error readinf from stdin")?;
                process_inputs(&mut config, format, data).context("Error parsing inputs from --stdin")?;
            }
            "--file" | "-f" => {
                let path = args
                    .pop()
                    .ok_or(anyhow!("error specified --file/-f flag but not context file path provided"))?;
                let (format, path) = if let Some((format, path)) = extract_format(&path) {
                    (format, path)
                } else {
                    let extension = Path::new(&path)
                        .extension()
                        .and_then(OsStr::to_str)
                        .ok_or(anyhow!("Error no extension found in ctx file"))?;
                    (extension.to_string(), path)
                };

                let data = fs::read_to_string(&path).with_context(|| format!("Error reading context file {}", path))?;
                process_inputs(&mut config, format, data).with_context(|| format!("Error parsing inputs from --file {}", path))?;
            }
            "--template" | "-t" => {
                let path = args
                    .pop()
                    .ok_or(anyhow!("error specified --template/-t flag but not context file path provided"))?;
                let data = fs::read_to_string(&path).with_context(|| format!("Error reading template file {}", path))?;
                process_inputs(&mut config, "tpl".into(), data).with_context(|| format!("Error parsing inputs from --file {}", path))?;
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
    return Ok(config);
}

pub fn process_inputs(mut config: &mut Config, format: String, data: String) -> Result<()> {
    if format == "template" || format == "tpl" || format == "j2" {
        config.template = data
    } else {
        populate_ctx(&mut config.context, format, data)?;
    }
    Ok(())
}

pub fn populate_ctx(context: &mut Context, format: String, data: String) -> Result<()> {
    match format.as_ref() {
        "yaml" | "yml" => {
            let value: serde_yaml::Value = serde_yaml::from_str(&data).context("Error parsing yaml")?;
            let object = value.as_mapping().context("Error expected object in root of yaml file")?;
            for (k, v) in object.iter() {
                let k = k.as_str().context("Error decoding key of yaml, key is not a string")?;
                context.insert(k, v);
            }
        }
        "json" => {
            let value = data.parse::<serde_json::Value>().context("Error parsing json")?;
            let object = value.as_object().context("Error expected object in root of json file")?;
            for (k, v) in object.iter() {
                context.insert(k, v);
            }
        }
        "toml" | "tml" => {
            let value = data.parse::<toml::Value>().context("Error parsing toml")?;
            let object = value.as_table().context("Error expected object in root of toml file")?;
            for (k, v) in object.iter() {
                context.insert(k, v);
            }
        }
        "hcl" | "tfvars" | "tf" => {
            let value = parse_hcl(&data).map_err(|e| anyhow!("Error {} parsing hcl/tf/tfvars", e))?;
            let value = value.to_string().parse::<serde_json::Value>().context("Error parsing json of hcl/tf/tfvars")?;

            let object = value
                .as_object()
                .context("Error expected object in root of hcl/tf/tfvars file")?;
            for (k, v) in object.iter() {
                context.insert(k, v);
            }
        }
        _ => panic!("Format {} not recognized", format),
    }
    Ok(())
}

pub fn main() -> Result<()> {
    let Config { template, context, print_ctx, out_file } = parse_args()?;

    if print_ctx {
        println!("{}", context.into_json().to_string());
        exit(0)
    }

    let mut tera = Tera::default();
    tera.add_raw_template("template", &template)
        .context("Error loading template in engine")?;

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

    tera.register_function("tab_all_lines", functions::tab_all_lines);
    tera.register_function("bash", functions::bash);
    tera.register_function("str", functions::str);
    tera.register_function("to_json", functions::str);
    tera.register_function("from_json", functions::from_json);

    tera.register_tester("file", testers::is_file);
    tera.register_tester("directory", testers::is_directory);

    let rendered = tera.render("template", &context).context("Error rendering template")?;

    if let Some(filepath) = out_file {
        let mut file = fs::File::create(&filepath).context("Error creating output file")?;
        file.write_all(rendered.as_ref()).context("Error writing to output file")?;
    } else {
        io::stdout()
            .write_all(rendered.as_ref())
            .context("Error writing to stdout")?;
    }

    Ok(())
}

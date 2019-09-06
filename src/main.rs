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

pub fn help() {
    println!(
        "
j2_render [opts]

    --stdin -i [json,yaml,hcl,tfvars,template] read from stdin context or template
    --out [file_path] output file for rendered template, default stdout
    --env -e load env vars in ctx
    --ctx -c [file_path] context files to be loaded in context, multiple files allowed, default empty
    --var -v key=value adds a pair key value to the context
    --template -t [file_path] template to be rendered, default empty
    --print-ctx -p print the context as json and exits
    --help shows this help
    "
    )
}

pub fn parse_args() -> (String, Option<String>, bool, Context) {
    let mut args = env::args().collect::<Vec<String>>();
    args.reverse();

    let mut template = String::new();
    let mut context = Context::new();
    let mut out = None;
    let mut print_ctx = false;

    args.pop(); // binary name

    while let Some(arg) = args.pop() {
        match arg.as_str() {
            "--print-ctx" | "-p" => print_ctx = true,
            "--var" | "-v" => {
                let variable = args
                    .pop()
                    .expect("error specified --var/-v flag but not value provided");
                let mut parts : Vec<&str> = variable.split('=').collect();
                let key = parts.pop().expect("Error no key=value found");
                let value = parts.join("=");
                context.insert(&key, &value)
            }
            "--out" | "-o" => {
                let filepath = args
                    .pop()
                    .expect("error specified --out/-o flag but not file path provided");
                out = Some(filepath);
            }
            "--stdin" | "-i" => {
                let format = args
                    .pop()
                    .expect("error specified --stdin/-i flag but not format provided");
                let mut data = String::new();
                io::stdin().read_to_string(&mut data).expect("Error readinf from stdin");
                if format == "template" {
                    template = data
                } else {
                    populate_ctx(&mut context, format, data);
                }
            }
            "--env" | "-e" => {
                let env_vars = env::vars().collect::<HashMap<String, String>>();
                for (k, v) in env_vars.iter() {
                    context.insert(k, v);
                }
            }
            "--ctx" | "-c" => {
                let path = args
                    .pop()
                    .expect("error specified --ctx/-c flag but not context file path provided");
                let extension = Path::new(&path)
                    .extension()
                    .and_then(OsStr::to_str)
                    .expect("Error no extension found in ctx file");
                let data = fs::read_to_string(&path).expect(&format!("Error reading context file {}", path));

                populate_ctx(&mut context, extension.to_string(), data);
            }
            "--template" | "-t" => {
                let path = args
                    .pop()
                    .expect("error specified --template/-t flag but not template path provided");
                template = fs::read_to_string(path).expect("Error reading template")
            }
            "--help" | "help" | "-h" => {
                help();
                exit(0);
            }
            _ => panic!("Error argument {} not recognized", arg),
        }
    }
    return (template, out, print_ctx, context);
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
    let (template, out, print_ctx, context) = parse_args();

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

    if let Some(filepath) = out {
        let mut file = fs::File::create(&filepath).expect("Error creating output file");
        file.write_all(rendered.as_ref()).expect("Error writing to output file");
    } else {
        io::stdout()
            .write_all(rendered.as_ref())
            .expect("Error writing to stdout");
    }

    Ok(())
}

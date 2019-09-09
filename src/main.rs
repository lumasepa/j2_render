#![feature(or_patterns)]

use molysite::hcl::parse_hcl;
use regex::Regex;
use serde_json;
use serde_yaml;
use tera::{Context, Tera};

use std::ffi::OsStr;
use std::io::Write;
use std::process::exit;
use std::{
    collections::HashMap,
    env, fs,
    io::{self, Read},
    path::Path,
};

mod error;
mod j2;
mod pairs;
mod source;
mod destination;

use crate::error::{ToWrapErrorResult, WrapError};
use crate::j2::tera::tera_render;
use crate::pairs::Pairs;
use crate::source::Source;
use crate::destination::Destination;

pub struct Config {
    pub template: Option<String>,
    pub context: Context,
    pub output: Option<Output>,
}

pub struct Pick {
    name: Option<String>,
    path: String,
    namespace: Option<String>,
}

pub struct Output {
    destination: Destination,
    format: Option<String>,
}

pub struct Input {
    source: Source,
    format: String,
    picks: Option<Vec<Pick>>,
    namespace: Option<String>,
}

pub fn get_source_content(source: Source) -> Result<String, WrapError> {
    panic!()
}

pub fn parse_input_pairs(pairs: Pairs) -> Result<Input, WrapError> {
    let source = Source::try_from_pairs(&pairs).wrap("Error parsing source from pairs")?;
    let format = pairs.get("format").wrap("Expected format=")?;
    let path = pairs.get("path");
    let namespace = pairs.get("namespace");
    let name = pairs.get("as");
    let picks = path.map(|path|{
        let mut picks = vec![];
        let pick = Pick{name,path,namespace: namespace.clone()};
        picks.push(pick);
        picks
    });
    return Ok(Input{source,picks,namespace,format});
}

pub fn parse_output_pairs(pairs: Pairs) -> Result<Output, WrapError> {
    let destination = Destination::try_from_pairs(&pairs).wrap("Error parsing source from pairs")?;
    let format = pairs.get("format");
    return Ok(Output{destination, format})
}


pub fn parse_args() -> Result<Config, WrapError> {
    let mut args = env::args().collect::<Vec<String>>();
    args.reverse();

    let mut config = Config {
        template: None,
        context: Context::new(),
        output: None,
    };

    let mut inputs: Vec<Input> = vec![];

    args.pop(); // binary name

    while let Some(arg) = args.pop() {
        let pairs = match arg.as_str() {
            "--var" | "-v" => {
                let variable = args
                    .pop()
                    .wrap("error specified --var/-v flag but not value provided")?;
                let mut parts: Vec<&str> = variable.splitn(2, '=').collect();
                let key = parts.pop().expect("Error no key=value found");
                let value = parts.pop().expect("Error no key=value found");
                args.push("source=var".to_string());
                args.push(format!("value={}", value));
                args.push(format!("namespace={}", key));
                Pairs::try_from_args(&mut args)?
            }
            "--out" | "-o" => {
                Pairs::try_from_args(&mut args)?
            }
            "--stdin" | "-i" => {
                let mut value = String::new();
                io::stdin()
                    .read_to_string(&mut value)
                    .expect("Error readinf from stdin");
                args.push(format!("source=stdin"));
                Pairs::try_from_args(&mut args)?
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

                args.push(format!("source=file"));
                args.push(format!("file={}", path));
                args.push(format!("format={}", extension));
                Pairs::try_from_args(&mut args)?
            }
            "--env" | "-e" => {
                if let Some(variable_name) = args.pop() {
                    if ! variable_name.starts_with("-") {
                        args.push(arg);
                    } else {
                        args.push(format!("key={}", variable_name));
                    }
                }
                args.push("source=env".to_string());
                Pairs::try_from_args(&mut args)?
            }
            "--help" | "help" | "-h" => {
                help();
                exit(0);
            }
            _ => panic!("Error argument {} not recognized", arg),
        };
    }

    return Ok(config);
}

pub fn help() {
    println!(
        "
render [FLAGS]
    render is a tool to work with configurations and data in general.

    It can load data from different inputs and in different formats,
    all data is transformed to json and loaded in the \"context\"
    you can arranged it as you want in the \"context\". Then the
    \"context\" is used to render the template then the template is
    sent to the outputs or in case of no template provided the \"context\"
    is sent to the outputs.

    To declare an input or output use a flag --flag and then write the pairs
    for that flag, a pair is a key=value, think that the flag is an input
    object creation and the pairs are the field of the object. If a new flag
    is found in the args a new object creation starts. Some flags sets
    automatically some pairs, behind each FLAG you will find which pairs
    it sets.

    To modify an input before adding it to the context the path= pair
    is used, path= accepts a JMES_PATH expresion to modify the input.

    To arrange the input in the context the namespace= and as= pairs
    are used. The namespace= indicates the KEY_PATH where the input
    is going to be placed in the context. The as= indicates the key
    or index where the input is going to be placed.

    The template engine of render is tera https://tera.netlify.com/ an
    implementation of jinja2 https://jinja.palletsprojects.com/en/2.10.x/
    please read tera documentation, jinja2 is a really pollewfull template
    engine, use it wisely.

    Some extra function, filter and testers are available in render, this
    functionality adds a lot of

    Workflow:

    inputs.iter |> read input |> parse input to json |> jmespath |> context

    context |> template ? render template : context |> outputs


    FLAGS:

    Inputs :
    --in SOURCE [INPUT_MANIPULATION]

    --file/-f file_path [INPUT_MANIPULATION]
        source=file file=file_path format=file_extension

    --stdin/-i [INPUT_MANIPULATION]
        source=stdin

    --env/-e
        source=env

    --env/-e env_var_name [INPUT_MANIPULATION]
        source=env key=env_var_name namespace=env_var_name

    --var/-v KEY_PATH=value [INPUT_MANIPULATION]
        source=var value=value namespace=KEY_PATH

    --http url [INPUT_MANIPULATION]
        source=http url=url format=url_extension

    --k8s namespace::[secret,configmap]::uri [INPUT_MANIPULATION]
        source=k8s k8s_namespace=namespace resource=[secret,configmap] uri=uri format=yaml

    Output:
    --out/-o DESTINATION [OUTPUT_MANIPULATION]

    --help/-h   -- shows this help
    --print-inputs -- print inputs and exits

    Options:
        INPUT_MANIPULATION:
            format|f=INPUT_FORMATS
            namespace|n=KEY_PATH
            path|p=JMES_PATH 
            as=key

        OUTPUT_MANIPULATION:
            format|f=OUTPUT_DATA_FORMATS

        SOURCES:
        source|s=
            file file=file_path|template
            stdin
            env key=env_var_name namespace=env_var_name
            value value=value,
            http url=url|template
                [method=[GET,POST,PUT,template]]  -- default POST
                [header=key:value|template]
                [basic=user:pass|template]
                [token=value|template]
                [digest=user:pass|template]
            k8s source=k8s k8s_namespace=namespace|template resource=[secret,configmap,template] uri=uri|template
                [kubectlconfig=path]  -- default env.KUBECTLCONFIG

        DESTINATIONS:
        destination|d=
            file file|f=file_path|template
            stdout
            http url=url|template
                [method=[GET,POST,PUT,template]]  -- default POST
                [header=key:value|template]
                [basic=user:pass|template]
                [token=value|template]
                [digest=user:pass|template]
            k8s source=k8s k8s_namespace=namespace|template resource=[secret,configmap,template] uri=uri|template
                [kubectlconfig=path]  -- default env.KUBECTLCONFIG


        KEY_PATH = absolute JMES_PATH to a key like root.first[5].third

        JMES_PATH = jmespath expression -- http://jmespath.org/tutorial.html

        Formats:
            INPUT_DATA_FORMATS: json5,json,yaml,hcl,tfvars,tf,csv
            OUTPUT_DATA_FORMATS: json,yaml,csv
            TEMPLATE_FORMATS : template,j2,tpl
            INPUT_FORMATS : INPUT_DATA_FORMATS + TEMPLATE_FORMATS

    "
    )
}
//
//fn extract_format(string: &str) -> Option<(String, String)> {
//    let mut parts: Vec<&str> = string.splitn(2, '+').collect();
//    let format = parts.pop().expect("");
//    if parts.len() == 0 {
//        return None;
//    }
//    let other = parts.pop().expect("");
//    return Some((format.to_string(), other.to_string()));
//}
//
//pub fn parse_args() -> Config {
//    let mut args = env::args().collect::<Vec<String>>();
//    args.reverse();
//
//    let mut config = Config {
//        template: None,
//        context: Context::new(),
//        output: None,
//    };
//
//    args.pop(); // binary name
//
//    while let Some(arg) = args.pop() {
//        match arg.as_str() {
//            "--var" | "-v" => {
//                let variable = args
//                    .pop()
//                    .expect("error specified --var/-v flag but not value provided");
//                let mut parts: Vec<&str> = variable.splitn(2, '=').collect();
//                let key = parts.pop().expect("Error no key=value found");
//                let value = parts.pop().expect("Error no key=value found");
//
//                if let Some((format, key)) = extract_format(key) {
//                    process_inputs(&mut config, format, value.to_string());
//                } else {
//                    config.context.insert(&key, &value)
//                }
//            }
//            "--out" | "-o" => {
//                let filepath = args
//                    .pop()
//                    .expect("error specified --out/-o flag but not file path provided");
//                config.out_file = Some(filepath);
//            }
//            "--stdin" | "-i" => {
//                let format = args
//                    .pop()
//                    .expect("error specified --stdin/-i flag but not format provided");
//                let mut data = String::new();
//                io::stdin().read_to_string(&mut data).expect("Error readinf from stdin");
//                process_inputs(&mut config, format, data);
//            }
//            "--file" | "-f" => {
//                let path = args
//                    .pop()
//                    .expect("error specified --file/-f flag but not context file path provided");
//                let (format, path) = if let Some((format, path)) = extract_format(&path) {
//                    (format, path)
//                } else {
//                    let extension = Path::new(&path)
//                        .extension()
//                        .and_then(OsStr::to_str)
//                        .expect("Error no extension found in ctx file");
//                    (extension.to_string(), path)
//                };
//
//                let data = fs::read_to_string(&path).expect(&format!("Error reading context file {}", path));
//                process_inputs(&mut config, format, data);
//            }
//            "--env" | "-e" => {
//                let env_vars = env::vars().collect::<HashMap<String, String>>();
//                for (k, v) in env_vars.iter() {
//                    config.context.insert(k, v);
//                }
//            }
//            "--help" | "help" | "-h" => {
//                help();
//                exit(0);
//            }
//            _ => panic!("Error argument {} not recognized", arg),
//        }
//    }
//    return config;
//}
//
//pub fn process_inputs(mut config: &mut Config, format: String, data: String) {
//    if format == "template" || format == "tpl" || format == "j2" {
//        config.template = Some(data)
//    } else {
//        populate_ctx(&mut config.context, format, data);
//    }
//}
//
//pub fn populate_ctx(context: &mut Context, format: String, data: String) {
//    match format.as_ref() {
//        "yaml" | "yml" => {
//            let value: serde_yaml::Value = serde_yaml::from_str(&data).expect("Error parsing yaml");
//            let object = value.as_mapping().expect("Error expected object in root of yaml file");
//            for (k, v) in object.iter() {
//                let k = k.as_str().expect("Error decoding key of yaml, key is not a string");
//                context.insert(k, v);
//            }
//        }
//        "json" => {
//            let value = data.parse::<serde_json::Value>().expect("Error parsing json");
//            let object = value.as_object().expect("Error expected object in root of json file");
//            for (k, v) in object.iter() {
//                context.insert(k, v);
//            }
//        }
//        "toml" | "tml" => {
//            let value = data.parse::<toml::Value>().expect("Error parsing toml");
//            let object = value.as_table().expect("Error expected object in root of toml file");
//            for (k, v) in object.iter() {
//                context.insert(k, v);
//            }
//        }
//        "hcl" | "tfvars" | "tf" => {
//            let value = parse_hcl(&data).expect("Error parsing hcl/tf/tfvars");
//            let value = value
//                .to_string()
//                .parse::<serde_json::Value>()
//                .expect("Error parsing json of hcl/tf/tfvars");
//
//            let object = value
//                .as_object()
//                .expect("Error expected object in root of hcl/tf/tfvars file");
//            for (k, v) in object.iter() {
//                context.insert(k, v);
//            }
//        }
//        _ => panic!("Format {} not recognized", format),
//    }
//}

pub fn main() -> std::result::Result<(), String> {
    panic!()
//    let Config {
//        template,
//        context,
//        out_file,
//    } = parse_args();
//
//    let rendered = if let Some(template) = template {
//        tera_render(template, context)
//    } else {
//        serde_yaml::to_string(&context);
//        context.as_json().expect("Error encoding ctx as json").to_string()
//    };
//
//    if let Some(filepath) = out_file {
//        let mut file = fs::File::create(&filepath).expect("Error creating output file");
//        file.write_all(rendered.as_ref()).expect("Error writing to output file");
//    } else {
//        io::stdout()
//            .write_all(rendered.as_ref())
//            .expect("Error writing to stdout");
//    }
//
//    Ok(())
}

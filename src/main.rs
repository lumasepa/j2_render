#![feature(or_patterns)]
#![allow(dead_code, incomplete_features)]

use tera::Context;

use std::env;
use std::process::exit;

#[macro_use]
mod error;
mod destination;
mod input;
mod j2;
mod output;
mod pairs;
mod parse;
mod source;

use crate::error::{ToWrapErrorResult, WrapError};

use crate::input::CtxOrTemplate::{Ctx, Template};
use crate::j2::tera::tera_render;
use crate::parse::{parse_args, parse_pairs};

pub fn help(topic: Option<String>) {
    if let Some(topic) = topic {
        match topic.as_ref() {
            "template_engine" => {}
            "jmespath" => {}
            "ops_file" => {}
            "pair_args" => {}
            _ => {}
        };
        return;
    }

    println!(
        "
render [FLAGS]
    render is a tool to work with configurations and data in general.

    It can load data from different inputs and in different formats,
    all data is transformed to json and loaded in the Context
    you can arranged it as you want in the Context. Then the
    Context is used to render the template then the template is
    sent to the outputs or in case of no template provided the Context
    is sent to the outputs.

    To declare an input or output use a flag --flag and then write the pairs
    for that flag, a pair is a key=value, think that the flag is an input
    object creation and the pairs are the field of the object. If a new flag
    is found in the args a new object creation starts. Some flags sets
    automatically some pairs, behind each FLAG you will find which pairs
    it sets.

    To avoid overwriting keys from diferent inputs in the Context 
    the namespace= is used, it indicates the key  where the input
    is going to be placed in the Context.

    The template engine of render is tera https://tera.netlify.com/ an
    implementation of jinja2 https://jinja.palletsprojects.com/en/2.10.x/
    please read tera documentation, jinja2 is a really pollewfull template
    engine, use it wisely.

    Some extra function, filter and testers are available in render, this
    functionality adds a lot of power.

    Workflow:

    inputs.iter |> read input |> parse input |> transform to json |> context

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

    --s3 s3://bucket/path [INPUT_MANIPULATION]
        source=s3 bucket=bucket path=path format=s3_url_extension

    Output:
    --out/-o DESTINATION [OUTPUT_MANIPULATION]

    --help/-h   -- shows this help
    --help/-h [template_engine,jmespath,ops_file,pair_args]  -- shows documentation of topic
    --print-inputs -- print inputs and exits

    Options:
        INPUT_MANIPULATION:
            format|f=INPUT_FORMATS
            namespace|n=key

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
            k8s k8s_namespace=namespace|template resource=[secret,configmap,template] uri=uri|template
                [kubectlconfig=path]  -- default env.KUBECTLCONFIG

        DESTINATIONS:
        destination|d=
            file file=file_path|template
            stdout

        JMES_PATH = jmespath expression -- http://jmespath.org/tutorial.html

        Formats:
            ENCODINGS: base64
            INPUT_DATA_FORMATS: ENCODING?+json5,json,yaml,hcl,tfvars,tf,csv
            OUTPUT_DATA_FORMATS: json,yaml,csv
            TEMPLATE_FORMATS : template,j2,tpl
            INPUT_FORMATS : INPUT_DATA_FORMATS + TEMPLATE_FORMATS
    "
    )
}

pub fn main() {
    if let Err(e) = cli_main() {
        println!("{}", e);
        exit(1)
    }
}

pub fn cli_main() -> std::result::Result<(), WrapError> {
    let mut args = env::args().collect::<Vec<String>>();

    let pairs_objects = parse_args(&mut args).wrap("Error parsing args")?;

    let mut context = Context::new();
    let mut template: Option<String> = None;

    for pairs in pairs_objects.iter() {
        println!("{}---------------------", pairs)
    }

    let (inputs, outputs) = parse_pairs(pairs_objects).wrap("Error parsing pairs")?;

    for raw_input in inputs {
        let rendered_inputs = raw_input.render(&mut context).wrap("")?;
        for rendered_input in rendered_inputs {
            let ctx_or_template = rendered_input.resolve()?;
            match ctx_or_template {
                Ctx(ctx) => context.extend(ctx),
                Template(tpl) => template = Some(tpl),
            }
        }
    }

    let rendered = template
        .map(|t| tera_render(t, &context))
        .unwrap_or_else(|| format!("{}", context.as_json().unwrap()));

    for output in outputs {}

    Ok(())
}

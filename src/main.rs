#![feature(or_patterns)]
#![allow(dead_code, incomplete_features)]

use tera::Context;

use std::process::exit;
use std::{env, fs, io};

#[macro_use]
mod error;
mod input;
mod j2;
mod pairs;
mod parse;
mod source;

use crate::error::{ToWrapErrorResult, WrapError};

use crate::input::CtxOrTemplate::{Ctx, Template};
use crate::j2::tera::tera_render;
use crate::parse::{parse_args, parse_pairs};
use std::io::Write;

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

    To avoid overwriting keys from different inputs in the Context 
    the namespace= is used, it indicates the key  where the input
    is going to be placed in the Context.

    The template engine of render is tera https://tera.netlify.com/ an
    implementation of jinja2 https://jinja.palletsprojects.com/en/2.10.x/
    please read tera documentation, jinja2 is a really pollewfull template
    engine, use it wisely.

    Some extra function, filter and testers are available in render, this
    functionality adds a lot of power.

    Workflow:

    inputs.for_each |> input.if |> render input |> read input |> transform to json |> context

    context |> render template |> outputs


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

    --var/-v key=value [INPUT_MANIPULATION]
        source=var value=value namespace=key

    Output:
    --out/-o file_path

    --help/-h   -- shows this help
    --help/-h [template_engine,jmespath,ops_file,pair_args]  -- shows documentation of topic
    --print-inputs -- print inputs and exits

    Options:
        INPUT_MANIPULATION:
            format|f=INPUT_FORMATS
            namespace|n=key
            for_each=JMES_PATH
            if=JMES_PATH

        SOURCES:
        source|s=
            file file=file_path|template
            stdin
            env key=env_var_name namespace=env_var_name
            value value=value,

        JMES_PATH = jmespath expression -- http://jmespath.org/tutorial.html

        Formats:
            INPUT_DATA_FORMATS: json5,json,yaml,hcl,tfvars,tf,csv
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
    if args.len() == 1 {
        help(None);
        return Ok(());
    }
    let (pairs_objects, output_path) = parse_args(&mut args).wrap_err("Error parsing args")?;

    let mut context = Context::new();
    let mut template: Option<String> = None;

    for pairs in pairs_objects.iter() {
        println!("{}---------------------", pairs)
    }

    let inputs = parse_pairs(pairs_objects).wrap_err("Error parsing pairs")?;

    for raw_input in inputs {
        let rendered_inputs = raw_input.render(&mut context).wrap_err("")?;
        for rendered_input in rendered_inputs {
            let ctx_or_template = rendered_input.resolve()?;
            match ctx_or_template {
                Ctx(ctx) => context.extend(ctx),
                Template(tpl) => template = Some(tpl),
            }
        }
    }

    let template = template.wrap_err("Error: Template n provided")?;

    let rendered = tera_render(template, &context);

    if let Some(filepath) = output_path {
        let mut file = fs::File::create(&filepath).expect("Error creating output file");
        file.write_all(rendered.as_ref()).expect("Error writing to output file");
    } else {
        io::stdout()
            .write_all(rendered.as_ref())
            .expect("Error writing to stdout");
    }
    Ok(())
}

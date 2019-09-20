use crate::error::{ToWrapErrorResult, WrapError};
use crate::j2::tera::tera_render;
use crate::pairs::Pairs;
use crate::source::Source;
use jmespath::Expression;
use json5;
use molysite::hcl::parse_hcl;
use molysite::types::JsonValue;
use serde_json::Value;
use tera::Context;

#[derive(Debug)]
pub struct Pick {
    name: Option<String>,
    expr: Expression<'static>,
}

#[derive(Debug)]
pub struct Input {
    // TODO rename to input template same for source, output and destination
    for_each: Option<Expression<'static>>,
    condition: Option<String>,

    source: Source,

    format: Option<String>,

    picks: Vec<Pick>,
    namespace: Option<String>,
}

#[derive(Debug)]
pub struct RenderedInput {
    source: Source,
    format: String,

    picks: Vec<Pick>,
    namespace: Option<String>,
}

impl Input {
    pub fn try_from_pairs(pairs: Pairs) -> Result<Input, WrapError> {
        let source = wrap_result!(
            Source::try_from_pairs(&pairs),
            "Error parsing source from pairs : {}",
            pairs
        )?;
        let format = pairs.get("format").or(pairs.get("f"));
        let namespace = pairs.get("as");
        let condition = pairs.get("if");
        let for_each = pairs.get("for_each").map(|for_each| {
            wrap_result!(jmespath::compile(&for_each), "Error parsing jmespath : {}", for_each).unwrap()
        });
        let picks = pairs
            .get_couples("path", "as")
            .iter()
            .map(|(path, name)| {
                let expr = wrap_result!(jmespath::compile(&path), "Error parsing jmespath : {}", path).unwrap();
                Pick {
                    name: name.to_owned(),
                    expr,
                }
            })
            .collect();
        return Ok(Input {
            condition,
            for_each,
            source,
            picks,
            namespace,
            format,
        });
    }

    pub fn render(&self, ctx: &Context) -> Vec<Input> {
        if let Some(for_each) = &self.for_each {
        } else {
            let cond = if let Some(condition) = &self.condition {
                let result = tera_render(condition.to_owned(), ctx);
                result.as_str() == "true"
            } else {
                true
            };
            if cond {
                let source = self.source.render(ctx).wrap("Error");
            }
        }
        todo!()
    }

    pub fn get_content(&self) -> Result<String, WrapError> {
        self.source.get_content()
    }

    pub fn deserialize(&self, data: String) -> Result<Context, WrapError> {
        let mut context = Context::new();
        let source_format = self.source.try_get_format();
        let format = self
            .format
            .as_ref()
            .or(source_format.as_ref())
            .ok_or(wrap_err!("Error format not found for Input {:?}", &self))?;

        match format.as_str() {
            "yaml" | "yml" => {
                let value: serde_yaml::Value =
                    wrap_result!(serde_yaml::from_str(&data), "Error parsing yaml : {:?}", self.source)?;
                let object = wrap_result!(
                    value.as_mapping(),
                    "Error expected object in root of yaml file : {:?}",
                    self.source
                )?;
                for (k, v) in object.iter() {
                    let k = wrap_result!(k.as_str(), "Error decoding key of yaml, key '{:?}' is not a string", k)?;
                    context.insert(k, v);
                }
            }
            "json" => {
                let value = wrap_result!(data.parse::<serde_json::Value>(), "Error parsing json : {}", data)?;
                let object = wrap_result!(
                    value.as_object(),
                    "Error expected object in root of json file : {:?}",
                    self.source
                )?;
                for (k, v) in object.iter() {
                    context.insert(k, v);
                }
            }
            "json5" => {
                let value = wrap_result!(
                    json5::from_str::<serde_json::Value>(&data),
                    "Error parsing json5 : {}",
                    data
                )?;
                let object = wrap_result!(
                    value.as_object(),
                    "Error expected object in root of json5 file : {:?}",
                    self.source
                )?;
                for (k, v) in object.iter() {
                    context.insert(k, v);
                }
            }
            "toml" | "tml" => {
                let value = wrap_result!(data.parse::<toml::Value>(), "Error parsing toml : {}", data)?;

                let object = wrap_result!(
                    value.as_table(),
                    "Error expected object in root of toml file : {:?}",
                    self.source
                )?;
                for (k, v) in object.iter() {
                    context.insert(k, v);
                }
            }
            "hcl" | "tfvars" | "tf" => {
                let value = wrap_result!(parse_hcl(&data), "Error parsing hcl/tf/tfvars : \n {}", data)?;
                let value = wrap_result!(
                    value.to_string().parse::<serde_json::Value>(),
                    "Error parsing json of hcl/tf/tfvars : {:?}",
                    self.source
                )?;

                let object = wrap_result!(
                    value.as_object(),
                    "Error expected object in root of hcl/tf/tfvars file : {:?}",
                    self.source
                )?;
                for (k, v) in object.iter() {
                    context.insert(k, v);
                }
            }
            _ => return Err(wrap_err!("Format {} not recognized", format)),
        }
        Ok(context)
    }

    pub fn filter_by_jmespath(&self, ctx: Context) -> Result<Context, WrapError> {
        let json_obj = ctx.as_json().wrap("Error converting ctx to json")?;

        for pick in self.picks.iter() {
            let result = wrap_result!(pick.expr.search(&json_obj), "Error evaluating jmespath : {}", pick.expr)?;
        }
        panic!()
    }
}

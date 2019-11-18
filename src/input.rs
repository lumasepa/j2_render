use crate::error::{ToWrapErrorResult, WrapError};
use crate::j2::tera::tera_render;
use crate::pairs::Pairs;
use crate::source::Source;
use jmespath::Expression;
use json5;
use molysite::hcl::parse_hcl;
use serde_json::Value;
use tera::Context;

#[derive(Debug)]
pub struct RawInput {
    for_each: Option<Expression<'static>>,
    condition: Option<String>,

    source: Source,

    format: Option<String>,
    namespace: Option<String>,
}

#[derive(Debug)]
pub struct RenderedInput {
    source: Source,

    format: Option<String>,
    namespace: Option<String>,
}

pub enum CtxOrTemplate {
    Ctx(Context),
    Template(String),
}

use CtxOrTemplate::*;

impl RenderedInput {
    pub fn resolve(&self) -> Result<CtxOrTemplate, WrapError> {
        let format = self.get_format()?;
        let content = self.get_content().wrap("Error getting content of input")?;

        if ["j2", "tpl", "template"].contains(&format.as_str()) {
            return Ok(Template(content));
        } else {
            let mut ctx = self.deserialize(content).wrap("Error deserializing input")?;
            if let Some(namespace) = &self.namespace {
                let mut new_ctx = Context::new();
                new_ctx.insert(namespace, &ctx);
                ctx = new_ctx
            }
            return Ok(Ctx(ctx));
        }
    }

    fn get_content(&self) -> Result<String, WrapError> {
        self.source.get_content()
    }

    fn get_format(&self) -> Result<String, WrapError> {
        let source_format = self.source.try_get_format();
        let format = self
            .format
            .as_ref()
            .or(source_format.as_ref())
            .ok_or(wrap_err!("Error format not found for Input {:?}", &self))?;
        Ok(format.to_owned())
    }

    fn deserialize(&self, data: String) -> Result<Context, WrapError> {
        let mut context = Context::new();
        let format = self.get_format()?;

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
}

impl RawInput {
    pub fn try_from_pairs(pairs: Pairs) -> Result<RawInput, WrapError> {
        let source = Source::try_from_pairs(&pairs).wrap("Error parsing source from pairs")?;
        let format = pairs.get("format").or(pairs.get("f"));
        let namespace = pairs.get("as");
        let condition = pairs.get("if");
        let for_each = pairs.get("for_each").map(|for_each| {
            jmespath::compile(&for_each)
                .wrap(&format!("Error parsing jmespath : {}", for_each))
                .unwrap()
        });
        return Ok(RawInput {
            condition,
            for_each,
            source,
            namespace,
            format,
        });
    }

    pub fn render(&self, ctx: &mut Context) -> Result<Vec<RenderedInput>, WrapError> {
        let mut rendered_inputs = vec![];
        if let Some(for_each) = &self.for_each {
            let json_obj = ctx.as_json().wrap("Error converting ctx to json")?;
            let elements = for_each.search(json_obj).wrap("")?;
            let elements = if let Some(elements) = elements.as_array() {
                elements
            } else {
                return Err(wrap_err!("{:?}", self));
            };
            for element in elements {
                ctx.insert("element", element);
                let input = RawInput {
                    for_each: None,
                    condition: self.condition.clone(),
                    source: self.source.clone(),
                    format: self.format.clone(),
                    namespace: self.namespace.clone(),
                };
                let mut rendered_input = input.render(ctx).wrap("")?;
                rendered_inputs.append(&mut rendered_input);
            }
            ctx.insert("element", &Value::Null)
        } else {
            let cond = if let Some(condition) = &self.condition {
                let result = tera_render(condition.to_owned(), ctx);
                result.as_str() == "true"
            } else {
                true
            };
            if cond {
                let source = self.source.render(ctx).wrap("Error")?;
                let format = self.format.as_ref().map(|f| tera_render(f.to_owned(), ctx));
                let namespace = self.namespace.as_ref().map(|n| tera_render(n.to_owned(), ctx));
                rendered_inputs.push(RenderedInput {
                    source,
                    format,
                    namespace,
                })
            }
        }
        Ok(rendered_inputs)
    }
}

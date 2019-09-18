use crate::error::{ToWrapErrorResult, WrapError};
use crate::j2::tera::tera_render;
use crate::pairs::Pairs;
use crate::source::Source;
use jmespath::Expression;
use json5;
use molysite::hcl::parse_hcl;
use tera::Context;

#[derive(Debug)]
pub struct Pick {
    name: Option<String>,
    path: String,
}

#[derive(Debug)]
pub struct RawInput {
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

    format: Option<String>,

    picks: Vec<Pick>,
    namespace: Option<String>,
}

impl RenderedInput {
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
            //            let result = pick
            //                .path
            //                .search(&json_obj)
            //                .wrap(&format!("Error evaluating jmespath : {}", pick.path))?;
        }
        panic!()
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
        let picks = pairs
            .get_couples("path", "as")
            .iter()
            .map(|(path, name)| Pick {
                name: name.to_owned(),
                path: path.to_owned(),
            })
            .collect();
        return Ok(RawInput {
            condition,
            for_each,
            source,
            picks,
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
                let picks = self
                    .picks
                    .iter()
                    .map(|pick| Pick {
                        path: pick.path.clone(),
                        name: pick.name.clone(),
                    })
                    .collect();
                let input = RawInput {
                    for_each: None,
                    condition: self.condition.clone(),
                    source: self.source.clone(),
                    format: self.format.clone(),
                    picks,
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
                let picks = self
                    .picks
                    .iter()
                    .map(|pick| Pick {
                        path: pick.path.clone(),
                        name: pick.name.as_ref().map(|n| tera_render(n.to_owned(), ctx)),
                    })
                    .collect();
                rendered_inputs.push(RenderedInput {
                    source,
                    format,
                    namespace,
                    picks,
                })
            }
        }
        Ok(rendered_inputs)
    }
}

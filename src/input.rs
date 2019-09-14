use crate::error::{ToWrapErrorResult, WrapError};
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
    for_each: Option<String>,
    condition: Option<String>,
    source: Source,
    format: String,
    picks: Vec<Pick>,
    namespace: Option<String>,
}

impl Input {
    pub fn try_from_pairs(pairs: Pairs) -> Result<Input, WrapError> {
        let source = Source::try_from_pairs(&pairs).wrap("Error parsing source from pairs")?;
        let format = pairs
            .get("format")
            .or(pairs.get("f"))
            .or(source.try_get_format())
            .wrap(&format!("Expected format= in pairs :\n{}\n", pairs))?;
        let namespace = pairs.get("as");
        let condition = pairs.get("if");
        let for_each = pairs.get("for_each");
        let picks = pairs
            .get_couples("path", "as")
            .iter()
            .map(|(path, name)| {
                let expr = jmespath::compile(&path)
                    .wrap(&format!("Error parsing jmespath : {}", path))
                    .unwrap();
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

    pub fn get_content(&self) -> Result<String, WrapError> {
        self.source.get_content()
    }

    pub fn deserialize(&self, data: String) -> Result<Context, WrapError> {
        let mut context = Context::new();
        match self.format.as_ref() {
            "yaml" | "yml" => {
                let value: serde_yaml::Value = serde_yaml::from_str(&data).wrap("Error parsing yaml")?;
                let object = value.as_mapping().wrap("Error expected object in root of yaml file")?;
                for (k, v) in object.iter() {
                    let k = k.as_str().wrap("Error decoding key of yaml, key is not a string")?;
                    context.insert(k, v);
                }
            }
            "json" => {
                let value = data.parse::<serde_json::Value>().wrap("Error parsing json")?;
                let object = value.as_object().wrap("Error expected object in root of json file")?;
                for (k, v) in object.iter() {
                    context.insert(k, v);
                }
            }
            "json5" => {
                let value = json5::from_str::<serde_json::Value>(&data).wrap("Error parsing json5")?;
                let object = value.as_object().wrap("Error expected object in root of json5 file")?;
                for (k, v) in object.iter() {
                    context.insert(k, v);
                }
            }
            "toml" | "tml" => {
                let value = data.parse::<toml::Value>().wrap("Error parsing toml")?;
                let object = value.as_table().wrap("Error expected object in root of toml file")?;
                for (k, v) in object.iter() {
                    context.insert(k, v);
                }
            }
            "hcl" | "tfvars" | "tf" => {
                let value = parse_hcl(&data).wrap("Error parsing hcl/tf/tfvars")?;
                let value = value
                    .to_string()
                    .parse::<serde_json::Value>()
                    .wrap("Error parsing json of hcl/tf/tfvars")?;

                let object = value
                    .as_object()
                    .wrap("Error expected object in root of hcl/tf/tfvars file")?;
                for (k, v) in object.iter() {
                    context.insert(k, v);
                }
            }
            _ => return Err(WrapError::new_first(&format!("Format {} not recognized", self.format))),
        }
        Ok(context)
    }

    pub fn filter_by_jmespath(&self, ctx: Context) -> Result<Context, WrapError> {
        let json_obj = ctx.as_json().wrap("Error converting ctx to json")?;

        for pick in self.picks.iter() {
            let result = pick
                .expr
                .search(&json_obj)
                .wrap(&format!("Error evaluating jmespath : {}", pick.expr))?;
        }
        panic!()
    }
}

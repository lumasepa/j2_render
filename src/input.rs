use molysite::hcl::parse_hcl;
use crate::error::{WrapError, ToWrapErrorResult};
use tera::Context;
use crate::source::Source;
use crate::pairs::Pairs;
use molysite::types::JsonValue;
use serde_json::Value;
use json5;







#[derive(Debug)]
pub struct Pick {
    name: Option<String>,
    path: String,
    namespace: Option<String>,
}

#[derive(Debug)]
pub struct Input {
    source: Source,
    format: String,
    picks: Vec<Pick>,
    namespace: Option<String>,
}

impl Input {
    pub fn try_from_pairs(pairs: Pairs) -> Result<Input, WrapError> {
        let source = Source::try_from_pairs(&pairs).wrap("Error parsing source from pairs")?;
        let format = pairs.get("format").or(pairs.get("f")).wrap("Expected format=")?;
        let path = pairs.get("path").or(pairs.get("p"));
        let namespace = pairs.get("namespace").or(pairs.get("n"));
        let name = pairs.get("as");
        let picks = path.map(|path| {
            let mut picks = vec![];
            let pick = Pick {
                name,
                path,
                namespace: namespace.clone(),
            };
            picks.push(pick);
            picks
        }).or_else(|| Some(vec![])).unwrap();
        return Ok(Input {
            source,
            picks,
            namespace,
            format,
        });
    }

    pub fn get_content(&self) -> Result<String, WrapError>{
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
            let expr = jmespath::compile(&pick.path).wrap(&format!("Error parsing jmespath : {}", pick.path))?;
            let result = expr.search(&json_obj).wrap(&format!("Error evaluating jmespath : {}", pick.path))?;

        }
        panic!()
    }
}

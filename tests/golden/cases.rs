#![allow(missing_docs)]

use std::{collections::BTreeMap, sync::OnceLock};

use serde::Deserialize;
use serde_json::{Map, Value};
use xgrammar_structural_tag::{Model, ToolChoice, ToolParam, build_structural_tag};

#[derive(Deserialize)]
struct RawCaseSpec {
    tools: BTreeMap<String, Value>,
    cases: Vec<RawCase>,
}

#[derive(Deserialize)]
struct RawCase {
    name: String,
    #[serde(default)]
    models: Vec<String>,
    tools: Vec<String>,
    tool_choice: Value,
    reasoning: bool,
}

struct PreparedSpec {
    cases: Vec<PreparedCase>,
}

struct PreparedCase {
    name: String,
    models: Vec<String>,
    tools: Vec<ToolParam>,
    tool_choice: ToolChoice,
    reasoning: bool,
}

fn spec() -> &'static PreparedSpec {
    static SPEC: OnceLock<PreparedSpec> = OnceLock::new();
    SPEC.get_or_init(|| {
        let raw: RawCaseSpec =
            serde_json::from_str(include_str!("cases.json")).expect("golden case spec is valid");
        let cases = raw
            .cases
            .into_iter()
            .map(|case| {
                let tools = case
                    .tools
                    .into_iter()
                    .map(|name| {
                        let value = raw.tools.get(&name).unwrap_or_else(|| {
                            panic!("golden case references unknown tool '{name}'")
                        });
                        serde_json::from_value::<ToolParam>(value.clone())
                            .expect("golden tool fixture deserializes")
                    })
                    .collect();
                let tool_choice = serde_json::from_value::<ToolChoice>(case.tool_choice)
                    .expect("golden tool choice fixture deserializes");
                PreparedCase {
                    name: case.name,
                    models: case.models,
                    tools,
                    tool_choice,
                    reasoning: case.reasoning,
                }
            })
            .collect();
        PreparedSpec { cases }
    })
}

pub fn build_cases(model: Model) -> xgrammar_structural_tag::Result<Value> {
    let spec = spec();
    let mut cases = Map::new();

    for case in &spec.cases {
        if !case.models.is_empty() && !case.models.iter().any(|name| name == model.as_str()) {
            continue;
        }

        cases.insert(
            case.name.clone(),
            serde_json::to_value(build_structural_tag(
                model,
                &case.tools,
                case.tool_choice.clone(),
                case.reasoning,
            )?)?,
        );
    }

    Ok(Value::Object(cases))
}

#![allow(missing_docs)]

use std::{collections::BTreeMap, sync::OnceLock};

use serde::Deserialize;
use serde_json::{Map, Value};
use xgrammar_structural_tag::{
    Model, ToolChoice, ToolParam, build_structural_tag, builders::ReasoningMode,
    builders::StructuralTagOptions,
};

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
    reasoning: ReasoningMode,
    #[serde(default = "default_true")]
    parallel_tool_calls: bool,
    #[serde(default)]
    any_order: bool,
    #[serde(default = "default_true")]
    exclude_special_tokens: bool,
    max_whitespace_cnt: Option<i32>,
}

fn default_true() -> bool {
    true
}

struct PreparedSpec {
    cases: Vec<PreparedCase>,
}

struct PreparedCase {
    name: String,
    models: Vec<String>,
    tools: Vec<ToolParam>,
    tool_choice: ToolChoice,
    options: StructuralTagOptions,
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
                    options: StructuralTagOptions::default()
                        .with_reasoning(case.reasoning)
                        .with_parallel_tool_calls(case.parallel_tool_calls)
                        .with_any_order(case.any_order)
                        .with_exclude_special_tokens(case.exclude_special_tokens)
                        .with_max_whitespace_cnt(case.max_whitespace_cnt),
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
                case.options,
            )?)?,
        );
    }

    Ok(Value::Object(cases))
}

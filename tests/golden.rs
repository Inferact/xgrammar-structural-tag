#![allow(missing_docs)]

use std::{fs, path::Path};

use serde_json::{Map, Value, json};
use strum::VariantArray;
use xgrammar_structural_tag::{
    AllowedToolRef, AllowedToolsMode, BuiltinToolParam, FunctionDefinition, FunctionToolParam,
    Model, ToolChoice, ToolParam, get_model_structural_tag,
};

fn function_tool(name: &str) -> ToolParam {
    ToolParam::Function(FunctionToolParam::new(
        FunctionDefinition::new(name).with_parameters(json!({
            "type": "object",
            "properties": { "q": { "type": "string" } },
            "required": ["q"]
        })),
    ))
}

fn strict_false_tool(name: &str) -> ToolParam {
    let mut tool = FunctionToolParam::new(FunctionDefinition::new(name).with_parameters(json!({
        "type": "object",
        "properties": { "q": { "type": "string" } },
        "required": ["q"]
    })));
    tool.function.strict = Some(false);
    ToolParam::Function(tool)
}

fn missing_parameters_tool(name: &str) -> ToolParam {
    ToolParam::Function(FunctionToolParam::new(FunctionDefinition::new(name)))
}

fn builtin_tool() -> ToolParam {
    ToolParam::Builtin(
        BuiltinToolParam::new("web_search_preview")
            .with_name("browser.search")
            .with_parameters(json!({
                "type": "object",
                "properties": { "query": { "type": "string" } },
                "required": ["query"]
            })),
    )
}

fn build_cases(model: Model) -> xgrammar_structural_tag::Result<Value> {
    let no_tools: Vec<ToolParam> = vec![];
    let one_tool = vec![function_tool("search")];
    let two_tools = vec![function_tool("search"), function_tool("alt")];
    let strict_false = vec![strict_false_tool("loose")];
    let missing_params = vec![missing_parameters_tool("missing")];

    let mut cases = Map::new();
    cases.insert(
        "auto_no_tools".to_string(),
        serde_json::to_value(get_model_structural_tag(
            model,
            &no_tools,
            ToolChoice::auto(),
            false,
        )?)?,
    );
    cases.insert(
        "auto_one_tool".to_string(),
        serde_json::to_value(get_model_structural_tag(
            model,
            &one_tool,
            ToolChoice::auto(),
            false,
        )?)?,
    );
    cases.insert(
        "required_two_tools".to_string(),
        serde_json::to_value(get_model_structural_tag(
            model,
            &two_tools,
            ToolChoice::required(),
            false,
        )?)?,
    );
    cases.insert(
        "reasoning_required_one_tool".to_string(),
        serde_json::to_value(get_model_structural_tag(
            model,
            &one_tool,
            ToolChoice::required(),
            true,
        )?)?,
    );
    cases.insert(
        "forced_search".to_string(),
        serde_json::to_value(get_model_structural_tag(
            model,
            &two_tools,
            ToolChoice::function("search"),
            false,
        )?)?,
    );
    cases.insert(
        "allowed_required_alt".to_string(),
        serde_json::to_value(get_model_structural_tag(
            model,
            &two_tools,
            ToolChoice::allowed_tools(
                AllowedToolsMode::Required,
                vec![AllowedToolRef::function("alt")],
            ),
            false,
        )?)?,
    );
    cases.insert(
        "strict_false".to_string(),
        serde_json::to_value(get_model_structural_tag(
            model,
            &strict_false,
            ToolChoice::auto(),
            false,
        )?)?,
    );
    cases.insert(
        "missing_parameters".to_string(),
        serde_json::to_value(get_model_structural_tag(
            model,
            &missing_params,
            ToolChoice::auto(),
            false,
        )?)?,
    );

    if model == Model::Harmony {
        cases.insert(
            "builtin_auto".to_string(),
            serde_json::to_value(get_model_structural_tag(
                model,
                &[builtin_tool()],
                ToolChoice::auto(),
                false,
            )?)?,
        );
    }

    Ok(Value::Object(cases))
}

#[test]
fn generated_structural_tags_match_golden_files() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for model in Model::VARIANTS {
        let path = root
            .join("tests")
            .join("golden")
            .join(format!("{}.json", model.as_str()));
        let expected: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let actual = build_cases(*model).unwrap();
        assert_eq!(actual, expected, "golden mismatch for {model}");
    }
}

#![allow(missing_docs)]

use std::{fs, path::Path};

use serde_json::Value;
use strum::VariantArray;
use xgrammar_structural_tag::Model;

#[path = "golden/cases.rs"]
mod golden_cases;

#[test]
fn generated_structural_tags_match_golden_files() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for model in Model::VARIANTS {
        let path = root
            .join("tests")
            .join("golden")
            .join(format!("{}.json", model.as_str()));
        let expected: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let actual = golden_cases::build_cases(*model).unwrap();
        assert_eq!(actual, expected, "golden mismatch for {model}");
    }
}

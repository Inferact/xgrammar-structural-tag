//! Generate Rust structural-tag golden fixtures.
//!
//! Run with `--write` to refresh files under `tests/golden/`; without it the
//! generated JSON is printed to stdout.

use std::{env, fs, path::PathBuf};

use strum::VariantArray;
use xgrammar_structural_tag::Model;

#[path = "../tests/golden/cases.rs"]
mod golden_cases;

fn main() -> xgrammar_structural_tag::Result<()> {
    let write = env::args().any(|arg| arg == "--write");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let golden_dir = root.join("tests").join("golden");
    if write {
        fs::create_dir_all(&golden_dir).expect("create golden directory");
    }

    for model in Model::VARIANTS {
        let value = golden_cases::build_cases(*model)?;
        let rendered = serde_json::to_string_pretty(&value)?;
        if write {
            fs::write(
                golden_dir.join(format!("{}.json", model.as_str())),
                rendered,
            )
            .expect("write golden fixture");
        } else {
            println!("===== {model}.json =====\n{rendered}");
        }
    }
    Ok(())
}

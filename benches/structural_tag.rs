#![allow(missing_docs)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use strum::VariantArray;
use xgrammar_structural_tag::Model;

#[path = "../tests/golden/cases.rs"]
mod golden_cases;

fn bench_build_cases_value(c: &mut Criterion) {
    let mut group = c.benchmark_group("build_cases_value");
    for model in Model::VARIANTS {
        group.bench_function(model.as_str(), |b| {
            b.iter(|| black_box(golden_cases::build_cases(black_box(*model)).unwrap()))
        });
    }
    group.finish();
}

fn bench_build_cases_json_string(c: &mut Criterion) {
    let mut group = c.benchmark_group("build_cases_json_string");
    for model in Model::VARIANTS {
        group.bench_function(model.as_str(), |b| {
            b.iter(|| {
                let cases = golden_cases::build_cases(black_box(*model)).unwrap();
                black_box(serde_json::to_string(&cases).unwrap())
            })
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_build_cases_value,
    bench_build_cases_json_string
);
criterion_main!(benches);

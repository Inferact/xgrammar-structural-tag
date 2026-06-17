#!/usr/bin/env python3
"""Benchmark upstream xgrammar structural-tag golden case construction.

This script intentionally keeps Python measurements separate from Rust
Criterion output. Run both benchmarks manually and compare the per-model
numbers.
"""

from __future__ import annotations

import argparse
import json
import statistics
import timeit
from typing import Iterable

try:
    from generate_python_golden import MODELS, build_cases
except ModuleNotFoundError as err:
    if err.name == "xgrammar":
        raise SystemExit(
            "missing Python package 'xgrammar'; install it or run with "
            "`uv run --with xgrammar==0.2.2 python scripts/bench_python_structural_tag.py`"
        ) from err
    raise


def parse_models(value: str | None) -> list[str]:
    if value is None:
        return list(MODELS)
    requested = [item.strip() for item in value.split(",") if item.strip()]
    unknown = sorted(set(requested) - set(MODELS))
    if unknown:
        raise SystemExit(f"unknown model(s): {', '.join(unknown)}")
    return requested


def bench_model(model: str, repeat: int, number: int, warmup: int) -> dict[str, object]:
    timer = timeit.Timer(lambda: build_cases(model))
    if warmup:
        timer.timeit(number=warmup)

    samples = timer.repeat(repeat=repeat, number=number)
    per_op = [sample / number for sample in samples]
    cases = build_cases(model)
    output = json.dumps(cases, ensure_ascii=False, separators=(",", ":"))

    return {
        "model": model,
        "case_count": len(cases),
        "output_bytes": len(output.encode("utf-8")),
        "median_us": statistics.median(per_op) * 1_000_000,
        "min_us": min(per_op) * 1_000_000,
        "max_us": max(per_op) * 1_000_000,
    }


def print_table(rows: Iterable[dict[str, object]]) -> None:
    print("| model | cases | bytes | median_us/op | min_us/op | max_us/op |")
    print("| --- | ---: | ---: | ---: | ---: | ---: |")
    for row in rows:
        print(
            "| {model} | {case_count} | {output_bytes} | {median_us:.2f} | "
            "{min_us:.2f} | {max_us:.2f} |".format(**row)
        )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--models",
        help="Comma-separated model list. Defaults to all upstream models.",
    )
    parser.add_argument("--repeat", type=int, default=10)
    parser.add_argument("--number", type=int, default=100)
    parser.add_argument("--warmup", type=int, default=10)
    args = parser.parse_args()

    rows = [
        bench_model(model, repeat=args.repeat, number=args.number, warmup=args.warmup)
        for model in parse_models(args.models)
    ]
    print_table(rows)


if __name__ == "__main__":
    main()

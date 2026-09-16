#!/usr/bin/env python3
# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "xgrammar==0.2.7",
# ]
# ///
"""Generate xgrammar-origin golden fixtures for this crate.

Run this script through uv so its inline dependency metadata supplies the
matching Python xgrammar release. The Rust crate itself has no runtime xgrammar
dependency.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

from xgrammar import get_model_structural_tag

SPEC_PATH = Path(__file__).resolve().parents[1] / "tests" / "golden" / "cases.json"


def load_spec() -> dict[str, Any]:
    return json.loads(SPEC_PATH.read_text(encoding="utf-8"))


SPEC = load_spec()
MODELS = SPEC["models"]["upstream"]


def prepare_cases() -> list[dict[str, Any]]:
    cases = []
    for case in SPEC["cases"]:
        cases.append(
            {
                "name": case["name"],
                "models": case.get("models", []),
                "tools": [SPEC["tools"][name] for name in case["tools"]],
                "tool_choice": case["tool_choice"],
                "reasoning": case["reasoning"],
                "parallel_tool_calls": case.get("parallel_tool_calls", True),
                "any_order": case.get("any_order", False),
                "exclude_special_tokens": case.get("exclude_special_tokens", True),
                "max_whitespace_cnt": case.get("max_whitespace_cnt"),
            }
        )
    return cases


CASES = prepare_cases()


def dump_tag(
    model: str,
    tools: list[dict[str, Any]],
    tool_choice: Any,
    reasoning: str,
    **options: Any,
) -> dict[str, Any]:
    return get_model_structural_tag(
        model,
        tools=tools,
        tool_choice=tool_choice,
        reasoning=reasoning,
        **options,
    ).model_dump()


def build_cases(model: str) -> dict[str, Any]:
    cases = {}
    for case in CASES:
        if case["models"] and model not in case["models"]:
            continue
        cases[case["name"]] = dump_tag(
            model,
            case["tools"],
            case["tool_choice"],
            case["reasoning"],
            parallel_tool_calls=case["parallel_tool_calls"],
            any_order=case["any_order"],
            exclude_special_tokens=case["exclude_special_tokens"],
            max_whitespace_cnt=case["max_whitespace_cnt"],
        )
    return cases


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--check", action="store_true", help="Compare fixtures without writing them"
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=Path(__file__).resolve().parents[1] / "tests" / "golden",
    )
    args = parser.parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=True)

    for model in MODELS:
        path = args.output_dir / f"{model}.json"
        cases = build_cases(model)
        if args.check:
            actual = json.loads(path.read_text(encoding="utf-8"))
            for name, expected in cases.items():
                assert actual.get(name) == expected, (
                    f"upstream mismatch: {model}/{name}"
                )
            assert actual.keys() == cases.keys(), f"case list mismatch: {model}"
            continue
        path.write_text(
            json.dumps(cases, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
        )
    if args.check:
        print(f"All {len(MODELS)} upstream model fixtures match XGrammar.")


if __name__ == "__main__":
    main()

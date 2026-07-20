#!/usr/bin/env python3
# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "xgrammar==0.2.4",
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
            }
        )
    return cases


CASES = prepare_cases()


def dump_tag(
    model: str,
    tools: list[dict[str, Any]],
    tool_choice: Any,
    reasoning: bool,
) -> dict[str, Any]:
    return get_model_structural_tag(
        model,
        tools=tools,
        tool_choice=tool_choice,
        reasoning=reasoning,
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
        )
    return cases


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=Path(__file__).resolve().parents[1] / "tests" / "golden",
    )
    args = parser.parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=True)

    for model in MODELS:
        path = args.output_dir / f"{model}.json"
        path.write_text(
            json.dumps(build_cases(model), ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
        )


if __name__ == "__main__":
    main()

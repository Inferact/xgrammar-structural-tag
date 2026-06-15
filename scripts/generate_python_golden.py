#!/usr/bin/env python3
"""Generate xgrammar-origin golden fixtures for this crate.

This script intentionally depends on Python xgrammar. It is a development tool
for refreshing `tests/golden/*.json` against an installed xgrammar build, while
the Rust crate itself has no runtime xgrammar dependency.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

from xgrammar import get_model_structural_tag


MODELS = [
    "llama",
    "kimi",
    "deepseek_r1",
    "deepseek_v3_1",
    "qwen_3_5",
    "qwen_3_coder",
    "qwen_3",
    "harmony",
    "deepseek_v3_2",
    "minimax",
    "glm_4_7",
    "deepseek_v4",
]


def function_tool(name: str) -> dict[str, Any]:
    return {
        "type": "function",
        "function": {
            "name": name,
            "parameters": {
                "type": "object",
                "properties": {"q": {"type": "string"}},
                "required": ["q"],
            },
        },
    }


def strict_false_tool(name: str) -> dict[str, Any]:
    tool = function_tool(name)
    tool["function"]["strict"] = False
    return tool


def missing_parameters_tool(name: str) -> dict[str, Any]:
    return {"type": "function", "function": {"name": name}}


def builtin_tool() -> dict[str, Any]:
    return {
        "type": "web_search_preview",
        "name": "browser.search",
        "parameters": {
            "type": "object",
            "properties": {"query": {"type": "string"}},
            "required": ["query"],
        },
    }


def dump_tag(model: str, tools: list[dict[str, Any]], tool_choice: Any) -> dict[str, Any]:
    return get_model_structural_tag(
        model,
        tools=tools,
        tool_choice=tool_choice,
        reasoning=False,
    ).model_dump()


def build_cases(model: str) -> dict[str, Any]:
    one_tool = [function_tool("search")]
    two_tools = [function_tool("search"), function_tool("alt")]
    cases = {
        "auto_no_tools": dump_tag(model, [], "auto"),
        "auto_one_tool": dump_tag(model, one_tool, "auto"),
        "required_two_tools": dump_tag(model, two_tools, "required"),
        "forced_search": dump_tag(
            model,
            two_tools,
            {"type": "function", "function": {"name": "search"}},
        ),
        "allowed_required_alt": dump_tag(
            model,
            two_tools,
            {
                "type": "allowed_tools",
                "allowed_tools": {
                    "mode": "required",
                    "tools": [
                        {"type": "function", "function": {"name": "alt"}},
                    ],
                },
            },
        ),
        "strict_false": dump_tag(model, [strict_false_tool("loose")], "auto"),
        "missing_parameters": dump_tag(model, [missing_parameters_tool("missing")], "auto"),
    }
    if model == "harmony":
        cases["builtin_auto"] = dump_tag(model, [builtin_tool()], "auto")
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


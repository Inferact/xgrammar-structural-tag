#!/usr/bin/env python3
# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "xgrammar==0.2.8",
# ]
# ///
"""Compile Rust fixtures and exercise their constraints with the real XGrammar runtime.

Run after `cargo run --example write_golden -- --write`. All inputs are checked-in
Rust-generated JSON, so this test checks the crate/runtime boundary directly.
"""

import json
from pathlib import Path

import xgrammar as xgr
from generate_python_golden import MODELS, SPEC, build_cases

ROOT = Path(__file__).resolve().parents[1] / "tests" / "golden"


def read(name):
    return json.loads((ROOT / f"{name}.json").read_text(encoding="utf-8"))


def check_models():
    # A byte vocabulary exercises compilation without downloading model assets.
    vocab = [bytes([i]) for i in range(256)] + [b"<eos>"]
    compiler = xgr.GrammarCompiler(xgr.TokenizerInfo(vocab, stop_token_ids=[256]))
    fixtures = {}
    compiled_count = 0
    expected_rejections = 0
    for model in MODELS + SPEC["models"]["rust_only"]:
        cases = read(model)
        if model in MODELS:
            assert cases == build_cases(model), f"upstream parity: {model}"
        fixtures[model] = cases
        for name, value in cases.items():
            # MiniMax M3's fixed-name XML converter explicitly rejects Any schemas.
            rejects_any = model == "minimax_m3" and name in {
                "strict_false",
                "missing_parameters",
            }
            try:
                compiler.compile_structural_tag(value)
            except (ValueError, RuntimeError) as error:
                assert rejects_any, f"{model}/{name}: {error}"
                assert "schema" in str(error).lower(), str(error)
                expected_rejections += 1
            else:
                assert not rejects_any, f"update documented limitation: {model}/{name}"
                compiled_count += 1

    def accepts(model, case, output):
        compiled = compiler.compile_structural_tag(fixtures[model][case])
        matcher = xgr.GrammarMatcher(compiled)
        return matcher.accept_string(output) and matcher.is_completed()

    # Nested containers must limit both calls per container and repeated containers.
    invoke = '<｜DSML｜ invoke name="search">\n<｜DSML｜ parameter name="q" string="true">x</｜DSML｜ parameter>\n</｜DSML｜ invoke>\n'
    call = "<｜DSML｜ calls>\n" + invoke + "</｜DSML｜ calls>"
    reasoning = "done</think>"
    assert accepts("deepseek_v4_1", "single_call_auto_two_tools", reasoning + call)
    assert not accepts(
        "deepseek_v4_1", "single_call_auto_two_tools", reasoning + call + call
    )
    assert not accepts(
        "deepseek_v4_1",
        "single_call_auto_two_tools",
        reasoning + "<｜DSML｜ calls>\n" + invoke * 2 + "</｜DSML｜ calls>",
    )
    assert accepts("deepseek_v4_1", "auto_one_tool", call + call)

    # Harmony's single-call setting keeps analysis messages ahead of the call.
    analysis = "<|channel|>analysis<|message|>thinking<|end|><|start|>assistant"
    call = '<|channel|>commentary to=functions.search<|constrain|>json<|message|>{"q":"x"}<|call|>'
    assert accepts("harmony", "single_call_required_two_tools", analysis + call)
    assert not accepts(
        "harmony",
        "single_call_required_two_tools",
        analysis + call + "<|start|>assistant" + call,
    )

    # MiniMax M3 adaptive output may skip thinking with a lone close marker.
    call = ']<]minimax[>[<tool_call>\n]<]minimax[>[<invoke name="search">]<]minimax[>[<q>x]<]minimax[>[</q>]<]minimax[>[</invoke>\n]<]minimax[>[</tool_call>'
    for prefix in ["", "</mm:think>", "<mm:think>thinking</mm:think>"]:
        assert accepts("minimax_m3", "adaptive_required_two_tools", prefix + call), (
            prefix
        )
    # MiMo uses the compact Qwen XML layout; Qwen3-Coder's newline layout must not match.
    call = "<tool_call><function=search><parameter=q>x</parameter></function></tool_call>"
    newline_call = "<tool_call>\n<function=search>\n<parameter=q>\nx\n</parameter>\n</function>\n</tool_call>"
    assert accepts("mimo", "required_two_tools", call)
    assert not accepts("mimo", "required_two_tools", newline_call)
    assert accepts("mimo", "reasoning_required_one_tool", "<think>t</think>" + call)
    assert not accepts("mimo", "reasoning_required_one_tool", call)
    assert accepts("mimo", "adaptive_required_two_tools", call)
    assert accepts("mimo", "adaptive_required_two_tools", "<think>t</think>" + call)

    # Kimi K3 argument strings may not contain its channel markers.
    kimi_call = (
        'r<|close|>response<|sep|><|open|>tools<|sep|><|open|>call tool="search" index="1"<|sep|>'
        '<|open|>argument key="q" type="string"<|sep|>%s<|close|>argument<|sep|>'
        "<|close|>call<|sep|><|close|>tools<|sep|><|close|>message<|sep|>"
    )
    assert accepts("kimi_k3", "required_two_tools", kimi_call % "x")
    assert not accepts("kimi_k3", "required_two_tools", kimi_call % "a<|sep|>b")
    print(
        f"Compiled {compiled_count} model cases; confirmed {expected_rejections} upstream schema rejections and single-call/adaptive behavior."
    )


def check_budgets():
    vocab = ["<think>", "a", "b", "c", "de", "中", "</", "think>", "</think>", "<eos>"]
    compiler = xgr.GrammarCompiler(xgr.TokenizerInfo(vocab, stop_token_ids=[9]))
    compiled = {
        name: compiler.compile_structural_tag(tag)
        for name, tag in read("formats").items()
    }

    def allowed(matcher, token):
        mask = xgr.allocate_token_bitmask(1, len(vocab))
        matcher.fill_next_token_bitmask(mask)
        index = vocab.index(token)
        return bool((int(mask[0, index // 32]) >> (index % 32)) & 1)

    def accepts(name, tokens):
        matcher = xgr.GrammarMatcher(compiled[name])
        for token in tokens:
            if not allowed(matcher, token) or not matcher.accept_token(
                vocab.index(token)
            ):
                return False
        return matcher.is_completed()

    for name in ["any_text_tokens", "any_text_both"]:
        assert accepts(name, ["<think>", "a", "b", "</", "think>"])
        assert not accepts(name, ["<think>", "a", "b", "c", "</", "think>"])
    assert accepts("any_text_chars", ["<think>", "中", "中", "中", "</think>"])
    assert not accepts("any_text_chars", ["<think>", "de", "de", "</think>"])
    assert not accepts("any_text_both", ["<think>", "de", "de", "</think>"])
    assert accepts("any_tokens", ["<think>", "a", "b", "</think>"])
    assert not accepts("any_tokens", ["<think>", "a", "b", "c", "</think>"])
    for name in ["any_text_zero", "any_tokens_zero"]:
        assert accepts(name, ["<think>", "</think>"])
        assert not accepts(name, ["<think>", "a", "</think>"])

    # Speculative rollback restores the region budget as well as grammar state.
    matcher = xgr.GrammarMatcher(compiled["any_text_tokens"])
    for token in ["<think>", "a", "b"]:
        assert allowed(matcher, token)
        assert matcher.accept_token(vocab.index(token))
    assert not allowed(matcher, "c")
    assert not allowed(matcher, "<eos>")
    assert allowed(matcher, "</")
    matcher.rollback(1)
    assert allowed(matcher, "c")
    print(
        "Region budgets enforce token/codepoint limits, zero budgets, multi-token closes, and rollback."
    )


if __name__ == "__main__":
    check_models()
    check_budgets()

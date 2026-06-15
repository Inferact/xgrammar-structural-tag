//! Model keys supported by the structural tag registry.

/// One supported model structural-tag template.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Model {
    /// Llama JSON function calling.
    Llama,
    /// Kimi K2 tool calling.
    Kimi,
    /// DeepSeek R1 tool calling.
    DeepSeekR1,
    /// DeepSeek V3.1 tool calling.
    DeepSeekV31,
    /// Qwen 3.5 / Qwen 3 Coder XML tool calling.
    Qwen35,
    /// Qwen 3 Coder alias for Qwen 3.5 XML tool calling.
    Qwen3Coder,
    /// Qwen 3 JSON-in-tag tool calling.
    Qwen3,
    /// OpenAI Harmony / GPT-OSS tool calling.
    Harmony,
    /// DeepSeek V3.2 DSML tool calling.
    DeepSeekV32,
    /// MiniMax XML tool calling.
    Minimax,
    /// GLM 4.7 / GLM 5 XML tool calling.
    Glm47,
    /// DeepSeek V4 DSML tool calling.
    DeepSeekV4,
    /// vLLM Hermes extension.
    Hermes,
    /// Rust frontend HY3 XML extension.
    HyV3,
}

impl Model {
    /// Parse a model key.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "llama" => Some(Self::Llama),
            "kimi" => Some(Self::Kimi),
            "deepseek_r1" => Some(Self::DeepSeekR1),
            "deepseek_v3_1" => Some(Self::DeepSeekV31),
            "qwen_3_5" => Some(Self::Qwen35),
            "qwen_3_coder" => Some(Self::Qwen3Coder),
            "qwen_3" => Some(Self::Qwen3),
            "harmony" => Some(Self::Harmony),
            "deepseek_v3_2" => Some(Self::DeepSeekV32),
            "minimax" => Some(Self::Minimax),
            "glm_4_7" => Some(Self::Glm47),
            "deepseek_v4" => Some(Self::DeepSeekV4),
            "hermes" => Some(Self::Hermes),
            "hy_v3" => Some(Self::HyV3),
            _ => None,
        }
    }

    /// Return the public model key.
    pub fn key(self) -> &'static str {
        match self {
            Self::Llama => "llama",
            Self::Kimi => "kimi",
            Self::DeepSeekR1 => "deepseek_r1",
            Self::DeepSeekV31 => "deepseek_v3_1",
            Self::Qwen35 => "qwen_3_5",
            Self::Qwen3Coder => "qwen_3_coder",
            Self::Qwen3 => "qwen_3",
            Self::Harmony => "harmony",
            Self::DeepSeekV32 => "deepseek_v3_2",
            Self::Minimax => "minimax",
            Self::Glm47 => "glm_4_7",
            Self::DeepSeekV4 => "deepseek_v4",
            Self::Hermes => "hermes",
            Self::HyV3 => "hy_v3",
        }
    }
}

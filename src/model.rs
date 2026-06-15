//! Model keys supported by the structural tag registry.

use std::{fmt, str::FromStr};

use strum::VariantArray;
use thiserror::Error as ThisError;

/// Error returned when parsing an unknown model key.
#[derive(Debug, Clone, PartialEq, Eq, ThisError)]
#[error("unknown structural tag model '{value}'")]
pub struct ParseModelError {
    value: String,
}

impl ParseModelError {
    /// Return the unknown model key.
    pub fn value(&self) -> &str {
        &self.value
    }
}

/// One supported model structural-tag template.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, VariantArray)]
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
    /// Hermes tool calling.
    Hermes,
    /// HY3 XML tool calling.
    HyV3,
}

impl Model {
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

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.key())
    }
}

impl FromStr for Model {
    type Err = ParseModelError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let model = match value {
            "llama" => Self::Llama,
            "kimi" => Self::Kimi,
            "deepseek_r1" => Self::DeepSeekR1,
            "deepseek_v3_1" => Self::DeepSeekV31,
            "qwen_3_5" => Self::Qwen35,
            "qwen_3_coder" => Self::Qwen3Coder,
            "qwen_3" => Self::Qwen3,
            "harmony" => Self::Harmony,
            "deepseek_v3_2" => Self::DeepSeekV32,
            "minimax" => Self::Minimax,
            "glm_4_7" => Self::Glm47,
            "deepseek_v4" => Self::DeepSeekV4,
            "hermes" => Self::Hermes,
            "hy_v3" => Self::HyV3,
            _ => {
                return Err(ParseModelError {
                    value: value.to_string(),
                });
            }
        };
        Ok(model)
    }
}

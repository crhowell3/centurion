use std::path::PathBuf;

use base64::Engine;
use iced_core::Color;
use palette::rgb::{Rgb, Rgba};
use palette::{FromColor, Hsva, Okhsl, Srgb, Srgba};
use rand::prelude::*;
use rand_chacha::ChaChaRng;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::fs;

const DEFAULT_THEME_NAME: &str = "Phalanx";
const DEFAULT_THEME_CONTENT: &str = include_str!("../../../assets/themes/phalanx.toml");

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub colors: Colors,
}

impl Theme {
    pub fn new(name: String, colors: Colors) -> Self {
        Theme { name, colors }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Colors {
    #[serde(default)]
    pub general: General,
    #[serde(default)]
    pub text: Text,
    #[serde(default)]
    pub buffer: Buffer,
    #[serde(default)]
    pub buttons: Buttons,
}

impl Colors {
    pub async fn save(self, path: PathBuf) -> Result<(), Error> {
        let content = toml::to_string(&self)?;

        fs::write(path, &content).await?;

        Ok(())
    }

    pub fn encode_base64(&self) -> String {
        let bytes = binary::encode(self);

        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&bytes)
    }

    pub fn decode_base64(content: &str) -> Result<Self, Error> {
        let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(content)?;

        Ok(binary::decode(&bytes));
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to serialize theme to toml: {0}")]
    Encode(#[from] toml::ser::Error),
    #[error("Failed to write theme file: {0}")]
    Write(#[from] std::io::Error),
    #[error("Failed to encode base64 theme string: {0}")]
    Base64Decode(#[from] base64::DecodeError),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Buttons {
    #[serde(default)]
    pub primary: Button,
    #[serde(default)]
    pub secondary: Button,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Button {
    #[serde(default = "default_transparent", with = "color_serde")]
    pub background: Color,
    #[serde(default = "default_transparent", with = "color_serde")]
    pub background_hover: Color,
    #[serde(default = "default_transparent", with = "color_serde")]
    pub background_selected: Color,
    #[serde(default = "default_transparent", with = "color_serde")]
    pub background_selected_hover: Color,
}

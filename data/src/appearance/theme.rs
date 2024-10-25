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

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: DEFAULT_THEME_NAME.to_string(),
            colors: Colors::default(),
        }
    }
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

        Ok(binary::decode(&bytes))
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct General {
    #[serde(default = "default_transparent", with = "color_serde")]
    pub background: Color,
    #[serde(default = "default_transparent", with = "color_serde")]
    pub border: Color,
    #[serde(default = "default_transparent", with = "color_serde")]
    pub horizontal_rule: Color,
    #[serde(default = "default_transparent", with = "color_serde")]
    pub unread_indicator: Color,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Text {
    #[serde(default = "default_transparent", with = "color_serde")]
    pub primary: Color,
    #[serde(default = "default_transparent", with = "color_serde")]
    pub secondary: Color,
    #[serde(default = "default_transparent", with = "color_serde")]
    pub tertiary: Color,
    #[serde(default = "default_transparent", with = "color_serde")]
    pub success: Color,
    #[serde(default = "default_transparent", with = "color_serde")]
    pub error: Color,
}

impl Default for Colors {
    fn default() -> Self {
        toml::from_str(DEFAULT_THEME_CONTENT).expect("parse default theme")
    }
}

pub fn hex_to_color(hex: &str) -> Option<Color> {
    if hex.len() == 7 || hex.len() == 9 {
        let hash = &hex[0..1];
        let r = u8::from_str_radix(&hex[1..3], 16);
        let g = u8::from_str_radix(&hex[3..5], 16);
        let b = u8::from_str_radix(&hex[5..7], 16);
        let a = (hex.len() == 9)
            .then(|| u8::from_str_radix(&hex[7..9], 16).ok())
            .flatten();

        return match (hash, r, g, b, a) {
            ("#", Ok(r), Ok(g), Ok(b), None) => Some(Color {
                r: r as f32 / 255.0,
                g: g as f32 / 255.0,
                b: b as f32 / 255.0,
                a: 1.0,
            }),
            ("#", Ok(r), Ok(g), Ok(b), Some(a)) => Some(Color {
                r: r as f32 / 255.0,
                g: g as f32 / 255.0,
                b: b as f32 / 255.0,
                a: a as f32 / 255.0,
            }),
            _ => None,
        };
    }

    None
}

pub fn color_to_hex(color: Color) -> String {
    use std::fmt::Write;

    let mut hex = String::with_capacity(9);

    let [r, g, b, a] = color.into_rgba8();

    let _ = write!(&mut hex, "#");
    let _ = write!(&mut hex, "{:02X}", r);
    let _ = write!(&mut hex, "{:02X}", g);
    let _ = write!(&mut hex, "{:02X}", b);

    if a < u8::MAX {
        let _ = write!(&mut hex, "{:02X}", a);
    }

    hex
}

/// Adjusts the transparency of the foreground color based on the background color's lightness.
pub fn alpha_color(min_alpha: f32, max_alpha: f32, background: Color, foreground: Color) -> Color {
    alpha(
        foreground,
        min_alpha + to_hsl(background).lightness * (max_alpha - min_alpha),
    )
}

/// Randomizes the hue value of an `iced::Color` based on a seed.
pub fn randomize_color(original_color: Color, seed: &str) -> Color {
    // Generate a 64-bit hash from the seed string
    let seed_hash = seahash::hash(seed.as_bytes());

    // Create a random number generator from the seed
    let mut rng = ChaChaRng::seed_from_u64(seed_hash);

    // Convert the original color to HSL
    let original_hsl = to_hsl(original_color);

    // Randomize the hue value using the random number generator
    let randomized_hue: f32 = rng.gen_range(0.0..=360.0);
    let randomized_hsl = Okhsl::new(
        randomized_hue,
        original_hsl.saturation,
        original_hsl.lightness,
    );

    // Convert the randomized HSL color back to Color
    from_hsl(randomized_hsl)
}

pub fn to_hsl(color: Color) -> Okhsl {
    let mut hsl = Okhsl::from_color(Rgb::from(color));
    if hsl.saturation.is_nan() {
        hsl.saturation = Okhsl::max_saturation();
    }

    hsl
}

pub fn to_hsva(color: Color) -> Hsva {
    Hsva::from_color(Rgba::from(color))
}

pub fn from_hsva(color: Hsva) -> Color {
    Srgba::from_color(color).into()
}

pub fn from_hsl(hsl: Okhsl) -> Color {
    Srgb::from_color(hsl).into()
}

pub fn alpha(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}

fn default_transparent() -> Color {
    Color::TRANSPARENT
}

mod color_serde {
    use iced_core::Color;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(String::deserialize(deserializer)
            .map(|hex| super::hex_to_color(&hex))?
            .unwrap_or(Color::TRANSPARENT))
    }

    pub fn serialize<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        super::color_to_hex(*color).serialize(serializer)
    }
}

mod color_serde_maybe {
    use iced_core::Color;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Color>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Option::<String>::deserialize(deserializer)?.and_then(|hex| super::hex_to_color(&hex)))
    }

    pub fn serialize<S>(color: &Option<Color>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        color.map(super::color_to_hex).serialize(serializer)
    }
}

mod binary {
    use iced_core::Color;
    use strum::{IntoEnumIterator, VariantArray};

    use super::{Buttons, Colors, General, Text};

    pub fn encode(colors: &Colors) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Tag::VARIANTS.len() * (1 + 4));

        for tag in Tag::iter() {
            if let Some(color) = tag.encode(colors) {
                bytes.push(tag as u8);
                bytes.extend(color);
            }
        }

        bytes
    }

    pub fn decode(bytes: &[u8]) -> Colors {
        let mut colors = Colors {
            general: General::default(),
            text: Text::default(),
            buttons: Buttons::default(),
        };

        for chunk in bytes.chunks(5) {
            if chunk.len() == 5 {
                if let Ok(tag) = Tag::try_from(chunk[0]) {
                    let color =
                        Color::from_rgba8(chunk[1], chunk[2], chunk[3], chunk[4] as f32 / 255.0);

                    tag.update_colors(&mut colors, color);
                }
            }
        }

        colors
    }

    // IMPORTANT: Tags cannot be rearranged or deleted to preserve
    // backwards compatability. Only append new items in the future
    #[derive(Debug, Clone, Copy, strum::EnumIter, strum::VariantArray, derive_more::TryFrom)]
    #[try_from(repr)]
    #[repr(u8)]
    pub enum Tag {
        GeneralBackground = 0,
        GeneralBorder = 1,
        GeneralHorizontalRule = 2,
        GeneralUnreadIndicator = 3,
        TextPrimary = 4,
        TextSecondary = 5,
        TextTertiary = 6,
        TextSuccess = 7,
        TextError = 8,
        ButtonsPrimaryBackground = 30,
        ButtonsPrimaryBackgroundHover = 31,
        ButtonsPrimaryBackgroundSelected = 32,
        ButtonsPrimaryBackgroundSelectedHover = 33,
        ButtonsSecondaryBackground = 34,
        ButtonsSecondaryBackgroundHover = 35,
        ButtonsSecondaryBackgroundSelected = 36,
        ButtonsSecondaryBackgroundSelectedHover = 37,
    }

    impl Tag {
        pub fn encode(&self, colors: &Colors) -> Option<[u8; 4]> {
            let color = match self {
                Tag::GeneralBackground => colors.general.background,
                Tag::GeneralBorder => colors.general.border,
                Tag::GeneralHorizontalRule => colors.general.horizontal_rule,
                Tag::GeneralUnreadIndicator => colors.general.unread_indicator,
                Tag::TextPrimary => colors.text.primary,
                Tag::TextSecondary => colors.text.secondary,
                Tag::TextTertiary => colors.text.tertiary,
                Tag::TextSuccess => colors.text.success,
                Tag::TextError => colors.text.error,
                Tag::ButtonsPrimaryBackground => colors.buttons.primary.background,
                Tag::ButtonsPrimaryBackgroundHover => colors.buttons.primary.background_hover,
                Tag::ButtonsPrimaryBackgroundSelected => colors.buttons.primary.background_selected,
                Tag::ButtonsPrimaryBackgroundSelectedHover => {
                    colors.buttons.primary.background_selected_hover
                }
                Tag::ButtonsSecondaryBackground => colors.buttons.secondary.background,
                Tag::ButtonsSecondaryBackgroundHover => colors.buttons.secondary.background_hover,
                Tag::ButtonsSecondaryBackgroundSelected => {
                    colors.buttons.secondary.background_selected
                }
                Tag::ButtonsSecondaryBackgroundSelectedHover => {
                    colors.buttons.secondary.background_selected_hover
                }
            };

            Some(color.into_rgba8())
        }

        pub fn update_colors(&self, colors: &mut Colors, color: Color) {
            match self {
                Tag::GeneralBackground => colors.general.background = color,
                Tag::GeneralBorder => colors.general.border = color,
                Tag::GeneralHorizontalRule => colors.general.horizontal_rule = color,
                Tag::GeneralUnreadIndicator => colors.general.unread_indicator = color,
                Tag::TextPrimary => colors.text.primary = color,
                Tag::TextSecondary => colors.text.secondary = color,
                Tag::TextTertiary => colors.text.tertiary = color,
                Tag::TextSuccess => colors.text.success = color,
                Tag::TextError => colors.text.error = color,
                Tag::ButtonsPrimaryBackground => colors.buttons.primary.background = color,
                Tag::ButtonsPrimaryBackgroundHover => {
                    colors.buttons.primary.background_hover = color
                }
                Tag::ButtonsPrimaryBackgroundSelected => {
                    colors.buttons.primary.background_selected = color
                }
                Tag::ButtonsPrimaryBackgroundSelectedHover => {
                    colors.buttons.primary.background_selected_hover = color;
                }
                Tag::ButtonsSecondaryBackground => colors.buttons.secondary.background = color,
                Tag::ButtonsSecondaryBackgroundHover => {
                    colors.buttons.secondary.background_hover = color
                }
                Tag::ButtonsSecondaryBackgroundSelected => {
                    colors.buttons.secondary.background_selected = color;
                }
                Tag::ButtonsSecondaryBackgroundSelectedHover => {
                    colors.buttons.secondary.background_selected_hover = color;
                }
            }
        }
    }
}

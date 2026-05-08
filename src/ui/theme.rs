//! Muted dark-fantasy palette and typography for readable, grounded UI.
//!
//! ## Typography scale (px)
//!
//! | Constant | px | Typical use |
//! |----------|-----|-------------|
//! | `FONT_DISPLAY_HERO` | 38 | Running playback clock |
//! | `FONT_DISPLAY_SUB` | 34 | Alternate large numerics |
//! | `FONT_HEADLINE` | 26 | Screen titles |
//! | `FONT_TITLE` | 22 | Brand / top bar title |
//! | `FONT_STRONG` | 20 | High-emphasis labels (log boss, purchase) |
//! | `FONT_SECTION` | 17 | Section headers |
//! | `FONT_SKILL_ACTIVE` | 18 | Unlocked skill slot label |
//! | `FONT_SKILL_DIM` | 16 | Locked skill slot / secondary buttons |
//! | `FONT_BODY` | 15 | Default reading text |
//! | `FONT_SUBLINE` | 16 | Log lines, treasure/shrine emphasis |
//! | `FONT_COMPACT` | 14 | Dense lists, chips |
//! | `FONT_CAPTION` | 13 | Captions, tooltips |
//! | `FONT_LABEL` | 12 | Table/sort hints |
//! | `FONT_MICRO` | 11 | Fine print |
//!
//! ## Spacing (px, suggested)
//!
//! | Constant | Value | Typical use |
//! |----------|-------|-------------|
//! | `PAD_ROOT` | 20 | Outer mockup inset, top bar horizontal |
//! | `PAD_BAR_Y` | 10 | Top bar vertical padding |
//! | `PANEL_INSET_LG` | 14–16 | Primary panel body padding |
//! | `PANEL_INSET` | 12 | Cards, scroll regions |
//! | `PANEL_INSET_SM` | 8 | Tight stacks, chips |
//! | `PAD_TOOLTIP` | 10 | Tooltip panel padding |
//!

use bevy::prelude::*;

pub struct UiTheme;

impl UiTheme {
    pub const FONT_DISPLAY_HERO: f32 = 38.0;
    pub const FONT_DISPLAY_SUB: f32 = 34.0;
    pub const FONT_HEADLINE: f32 = 26.0;
    pub const FONT_TITLE: f32 = 22.0;
    pub const FONT_STRONG: f32 = 20.0;
    pub const FONT_SECTION: f32 = 17.0;
    pub const FONT_SKILL_ACTIVE: f32 = 18.0;
    pub const FONT_SKILL_DIM: f32 = 16.0;
    pub const FONT_BODY: f32 = 15.0;
    pub const FONT_SUBLINE: f32 = 16.0;
    pub const FONT_COMPACT: f32 = 14.0;
    pub const FONT_CAPTION: f32 = 13.0;
    pub const FONT_LABEL: f32 = 12.0;
    pub const FONT_MICRO: f32 = 11.0;

    pub const PAD_ROOT: f32 = 20.0;
    pub const PAD_BAR_Y: f32 = 10.0;
    pub const PANEL_INSET: f32 = 12.0;
    pub const PANEL_INSET_SM: f32 = 8.0;
    pub const PANEL_INSET_LG: f32 = 14.0;
    pub const PAD_TOOLTIP: f32 = 10.0;
    pub fn void_black() -> Color {
        Color::srgb(0.04, 0.035, 0.042)
    }

    /// Base stone wash for full-screen backdrops.
    pub fn stone_deep() -> Color {
        Color::srgb(0.1, 0.09, 0.1)
    }

    pub fn stone_mid() -> Color {
        Color::srgb(0.12, 0.11, 0.115)
    }

    pub fn stone_highlight() -> Color {
        Color::srgb(0.16, 0.14, 0.13)
    }

    /// Ember / torch tint (very subtle overlays).
    pub fn torch_glow() -> Color {
        Color::srgba(0.35, 0.18, 0.08, 0.12)
    }

    pub fn panel_bg() -> Color {
        Color::srgb(0.11, 0.1, 0.105)
    }

    pub fn panel_bg_deep() -> Color {
        Color::srgb(0.085, 0.078, 0.082)
    }

    pub fn panel_border() -> Color {
        Color::srgb(0.22, 0.19, 0.18)
    }

    pub fn panel_border_inner() -> Color {
        Color::srgb(0.14, 0.12, 0.13)
    }

    pub fn muted_gold() -> Color {
        Color::srgb(0.78, 0.64, 0.38)
    }

    /// Ornamental bronze / gold for mockup-style panel frames.
    pub fn ornate_gold() -> Color {
        Color::srgb(0.62, 0.48, 0.22)
    }

    pub fn muted_cream() -> Color {
        Color::srgb(0.82, 0.78, 0.72)
    }

    pub fn body() -> Color {
        Color::srgb(0.72, 0.7, 0.68)
    }

    pub fn body_dim() -> Color {
        // Slightly lifted vs older stone grays so captions stay legible on `panel_bg_deep`.
        Color::srgb(0.56, 0.54, 0.51)
    }

    pub fn muted_red() -> Color {
        Color::srgb(0.55, 0.22, 0.2)
    }

    pub fn accent_red() -> Color {
        Color::srgb(0.72, 0.32, 0.28)
    }

    pub fn danger() -> Color {
        Color::srgb(0.75, 0.35, 0.32)
    }

    pub fn healing() -> Color {
        Color::srgb(0.45, 0.62, 0.48)
    }

    /// Toxic / debuff accent (poison chips, poison lines in the log). Distinct from [`Self::healing`].
    pub fn status_poison() -> Color {
        Color::srgb(0.52, 0.64, 0.38)
    }

    pub fn treasure() -> Color {
        Color::srgb(0.85, 0.7, 0.42)
    }

    pub fn elite() -> Color {
        Color::srgb(0.8, 0.38, 0.32)
    }
}

pub fn rarity_color(rarity: crate::domain::items::ItemRarity) -> Color {
    use crate::domain::items::ItemRarity;
    match rarity {
        ItemRarity::Common => UiTheme::body(),
        ItemRarity::Uncommon => Color::srgb(0.48, 0.62, 0.52),
        ItemRarity::Rare => Color::srgb(0.5, 0.55, 0.78),
        ItemRarity::Epic => Color::srgb(0.65, 0.42, 0.78),
        ItemRarity::Legendary => Color::srgb(0.92, 0.72, 0.38),
    }
}

/// Visual + typographic emphasis for run log / summary lines.
pub fn log_line_present(line: &str) -> (Color, f32) {
    let lower = line.to_lowercase();
    if line.contains("Warden") || line.contains("Gate Warden") {
        return (UiTheme::muted_gold(), UiTheme::FONT_STRONG);
    }
    if lower.contains("defeated by") {
        return (UiTheme::danger(), UiTheme::FONT_SUBLINE);
    }
    if lower.contains("elite") {
        return (UiTheme::elite(), UiTheme::FONT_SECTION);
    }
    if lower.contains("poison") {
        return (UiTheme::status_poison(), UiTheme::FONT_SUBLINE);
    }
    if lower.contains("shrine") {
        return (UiTheme::healing(), UiTheme::FONT_SUBLINE);
    }
    if lower.contains("found ") || lower.contains("treasure") {
        return (UiTheme::treasure(), UiTheme::FONT_SUBLINE);
    }
    if lower.contains("defeated") {
        return (UiTheme::body(), UiTheme::FONT_BODY);
    }
    (UiTheme::body_dim(), UiTheme::FONT_COMPACT)
}

/// Separator between status chips on playback hero/enemy debuff lines (must match domain/UI join).
pub const PLAYBACK_DEBUFF_SLOT_SEP: &str = "  ·  ";

/// Rich caption for a single debuff line: poison chips use [`UiTheme::status_poison`], empty `—` slots stay dim.
pub fn playback_debuff_status_text(line: &str) -> Text {
    let style_dim = TextStyle {
        font_size: UiTheme::FONT_CAPTION,
        color: UiTheme::body_dim(),
        ..default()
    };
    let style_poison = TextStyle {
        font_size: UiTheme::FONT_CAPTION,
        color: UiTheme::status_poison(),
        ..default()
    };
    let parts: Vec<&str> = line.split(PLAYBACK_DEBUFF_SLOT_SEP).collect();
    let mut sections = Vec::with_capacity(parts.len().max(1));
    for (i, part) in parts.iter().enumerate() {
        let prefix: &str = if i == 0 { "" } else { PLAYBACK_DEBUFF_SLOT_SEP };
        let style = if part.trim_start().starts_with("Poison") {
            style_poison.clone()
        } else {
            style_dim.clone()
        };
        sections.push(TextSection::new(format!("{prefix}{part}"), style));
    }
    if sections.is_empty() {
        Text::from_section("", style_dim)
    } else {
        Text::from_sections(sections)
    }
}

pub fn playback_debuff_text_bundle(line: &str) -> TextBundle {
    TextBundle {
        text: playback_debuff_status_text(line),
        ..default()
    }
}

/// Combat log body for playback: one styled section per line (newlines between), using [`log_line_present`].
pub fn playback_log_rich_text(lines: &[String]) -> Text {
    if lines.is_empty() {
        return Text::from_section(
            "",
            TextStyle {
                font_size: UiTheme::FONT_COMPACT,
                color: UiTheme::body_dim(),
                ..default()
            },
        );
    }
    let mut sections = Vec::with_capacity(lines.len());
    for (i, line) in lines.iter().enumerate() {
        let (color, size) = log_line_present(line);
        let mut value = line.clone();
        if i + 1 < lines.len() {
            value.push('\n');
        }
        sections.push(TextSection::new(
            value,
            TextStyle {
                font_size: size,
                color,
                ..default()
            },
        ));
    }
    Text::from_sections(sections)
}

/// Flatten multi-section [`Text`] for comparing against a joined log string.
pub fn text_flatten(text: &Text) -> String {
    text.sections.iter().map(|s| s.value.as_str()).collect()
}

pub fn headline_text(text: impl Into<String>) -> TextBundle {
    TextBundle::from_section(
        text,
        TextStyle {
            font_size: UiTheme::FONT_HEADLINE,
            color: UiTheme::muted_gold(),
            ..default()
        },
    )
}

pub fn section_title(text: impl Into<String>) -> TextBundle {
    TextBundle::from_section(
        text,
        TextStyle {
            font_size: UiTheme::FONT_SECTION,
            color: UiTheme::muted_cream(),
            ..default()
        },
    )
}

pub fn body_text(text: impl Into<String>) -> TextBundle {
    TextBundle::from_section(
        text,
        TextStyle {
            font_size: UiTheme::FONT_BODY,
            color: UiTheme::body(),
            ..default()
        },
    )
}

pub fn caption_text(text: impl Into<String>) -> TextBundle {
    TextBundle::from_section(
        text,
        TextStyle {
            font_size: UiTheme::FONT_CAPTION,
            color: UiTheme::body_dim(),
            ..default()
        },
    )
}

/// Combined stat total from base item + [`crate::domain::items::ItemInstance::affix_stats`].
pub fn format_item_stat_summary(item: &crate::domain::items::ItemInstance) -> String {
    let s = item.stats + item.affix_stats();
    let mut parts = Vec::new();
    if s.max_health != 0 {
        parts.push(format!("HP {:+}", s.max_health));
    }
    if s.damage != 0 {
        parts.push(format!("DMG {:+}", s.damage));
    }
    if s.armor != 0 {
        parts.push(format!("ARM {:+}", s.armor));
    }
    if s.healing_power != 0 {
        parts.push(format!("HEAL {:+}", s.healing_power));
    }
    if s.attack_speed != 0.0 {
        parts.push(format!("ATK SPD {:+}", s.attack_speed));
    }
    if parts.is_empty() {
        "No stat modifiers.".to_string()
    } else {
        parts.join(" · ")
    }
}

/// One bullet per affix with gameplay wording (see [`ItemAffix::effect_description`]).
pub fn format_item_affix_lines(item: &crate::domain::items::ItemInstance) -> String {
    if item.affixes.is_empty() {
        return String::new();
    }
    item.affixes
        .iter()
        .map(|a| format!("• {}", a.effect_description()))
        .collect::<Vec<_>>()
        .join("\n")
}

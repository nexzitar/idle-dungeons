//! Muted dark-fantasy palette and typography for readable, grounded UI.

use bevy::prelude::*;

pub struct UiTheme;

impl UiTheme {
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
        Color::srgb(0.5, 0.48, 0.46)
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
    }
}

/// Visual + typographic emphasis for run log / summary lines.
pub fn log_line_present(line: &str) -> (Color, f32) {
    let lower = line.to_lowercase();
    if line.contains("Warden") || line.contains("Gate Warden") {
        return (UiTheme::muted_gold(), 20.0);
    }
    if lower.contains("defeated by") {
        return (UiTheme::danger(), 16.0);
    }
    if lower.contains("elite") {
        return (UiTheme::elite(), 17.0);
    }
    if lower.contains("shrine") {
        return (UiTheme::healing(), 16.0);
    }
    if lower.contains("found ") || lower.contains("treasure") {
        return (UiTheme::treasure(), 16.0);
    }
    if lower.contains("defeated") {
        return (UiTheme::body(), 15.0);
    }
    (UiTheme::body_dim(), 14.0)
}

pub fn headline_text(text: impl Into<String>) -> TextBundle {
    TextBundle::from_section(
        text,
        TextStyle {
            font_size: 26.0,
            color: UiTheme::muted_gold(),
            ..default()
        },
    )
}

pub fn section_title(text: impl Into<String>) -> TextBundle {
    TextBundle::from_section(
        text,
        TextStyle {
            font_size: 17.0,
            color: UiTheme::muted_cream(),
            ..default()
        },
    )
}

pub fn body_text(text: impl Into<String>) -> TextBundle {
    TextBundle::from_section(
        text,
        TextStyle {
            font_size: 15.0,
            color: UiTheme::body(),
            ..default()
        },
    )
}

pub fn caption_text(text: impl Into<String>) -> TextBundle {
    TextBundle::from_section(
        text,
        TextStyle {
            font_size: 13.0,
            color: UiTheme::body_dim(),
            ..default()
        },
    )
}

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
    if !item.affixes.is_empty() {
        parts.push(
            item.affixes
                .iter()
                .map(|a| format!("{a:?}"))
                .collect::<Vec<_>>()
                .join(", "),
        );
    }
    if parts.is_empty() {
        "No stat modifiers.".to_string()
    } else {
        parts.join(" · ")
    }
}

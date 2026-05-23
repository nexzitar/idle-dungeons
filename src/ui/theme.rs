//! Muted dark-fantasy palette and typography for readable, grounded UI.
//!
//! Typography and spacing constants are unchanged from Bevy 0.14 layout; spawning uses Bevy 0.18 UI
//! components (`Node`, `Text`, `TextFont`, `TextColor`).
//!
//! ## Typography scale (px)
//!
//! See module docs in git history for the reference table (`FONT_*`, `PAD_*`).
//!

use bevy::prelude::*;
use bevy::text::{TextColor, TextFont};

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

    pub fn stone_deep() -> Color {
        Color::srgb(0.1, 0.09, 0.1)
    }

    pub fn stone_mid() -> Color {
        Color::srgb(0.12, 0.11, 0.115)
    }

    pub fn stone_highlight() -> Color {
        Color::srgb(0.16, 0.14, 0.13)
    }

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

/// Shared panel chrome (background, border, inset) for cards and shell columns.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiPanelStyle {
    pub background: Color,
    pub border: Color,
    pub padding_px: f32,
    pub row_gap_px: f32,
    pub border_px: f32,
}

impl UiPanelStyle {
    pub fn framed() -> Self {
        Self {
            background: UiTheme::panel_bg(),
            border: UiTheme::panel_border(),
            padding_px: UiTheme::PANEL_INSET_LG,
            row_gap_px: 10.0,
            border_px: 1.0,
        }
    }

    pub fn bottom_strip() -> Self {
        Self {
            background: UiTheme::panel_bg(),
            border: UiTheme::panel_border(),
            padding_px: UiTheme::PANEL_INSET_LG,
            row_gap_px: 12.0,
            border_px: 1.0,
        }
    }

    /// Stash / loot item rows (gear hub, summary rewards).
    pub fn deep_card() -> Self {
        Self {
            background: UiTheme::panel_bg_deep(),
            border: UiTheme::panel_border(),
            padding_px: UiTheme::PANEL_INSET,
            row_gap_px: 8.0,
            border_px: 1.0,
        }
    }

    /// Presentation editor hierarchy / inspector columns.
    pub fn editor_sidebar() -> Self {
        Self {
            background: UiTheme::panel_bg(),
            border: UiTheme::ornate_gold(),
            padding_px: UiTheme::PANEL_INSET_SM,
            row_gap_px: 8.0,
            border_px: 1.0,
        }
    }

    /// Centered modal dialog surface (settings, skill book).
    pub fn dialog() -> Self {
        Self {
            background: UiTheme::panel_bg_deep(),
            border: UiTheme::ornate_gold(),
            padding_px: UiTheme::PAD_ROOT,
            row_gap_px: UiTheme::PANEL_INSET,
            border_px: 2.0,
        }
    }

    pub fn with_border(self, border: Color) -> Self {
        Self { border, ..self }
    }

    pub fn inset_column_node(self) -> Node {
        Node {
            box_sizing: BoxSizing::BorderBox,
            width: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(self.padding_px)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            row_gap: Val::Px(self.row_gap_px),
            border: UiRect::all(Val::Px(self.border_px)),
            ..default()
        }
    }
}

/// Modal backdrop dimming colors (maps to [`crate::ui::components::UiButtonPalette`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiModalStyle {
    pub backdrop_idle_bg: Color,
    pub backdrop_hover_bg: Color,
    pub backdrop_pressed_bg: Color,
}

impl UiModalStyle {
    pub fn standard() -> Self {
        Self {
            backdrop_idle_bg: Color::srgba(0.02, 0.02, 0.04, 0.58),
            backdrop_hover_bg: Color::srgba(0.04, 0.04, 0.06, 0.65),
            backdrop_pressed_bg: Color::srgba(0.06, 0.06, 0.08, 0.72),
        }
    }

    pub fn backdrop_palette(self) -> crate::ui::components::UiButtonPalette {
        crate::ui::components::UiButtonPalette {
            idle_bg: self.backdrop_idle_bg,
            hover_bg: self.backdrop_hover_bg,
            pressed_bg: self.backdrop_pressed_bg,
            idle_border: Color::NONE,
            hover_border: Color::NONE,
            pressed_border: Color::NONE,
        }
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

pub const PLAYBACK_DEBUFF_SLOT_SEP: &str = "  ·  ";

fn debuff_plain_line(line: &str) -> String {
    line.split(PLAYBACK_DEBUFF_SLOT_SEP)
        .map(|part| {
            let part = part.trim_start();
            if part.starts_with("Poison") {
                format!(
                    "[Poison] {}",
                    part.strip_prefix("Poison").unwrap_or("").trim()
                )
            } else {
                part.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(PLAYBACK_DEBUFF_SLOT_SEP)
}

pub fn playback_debuff_line_string(line: &str) -> String {
    debuff_plain_line(line)
}

/// Combat log plain body (colors from multi-span text dropped; use [`log_line_present`] offline if needed).
pub fn playback_log_plain(lines: &[String]) -> String {
    lines.join("\n")
}

pub fn headline_text(text: impl Into<String>) -> impl Bundle {
    (
        Text::new(text.into()),
        TextFont::from_font_size(UiTheme::FONT_HEADLINE),
        TextColor(UiTheme::muted_gold()),
    )
}

pub fn section_title(text: impl Into<String>) -> impl Bundle {
    (
        Text::new(text.into()),
        TextFont::from_font_size(UiTheme::FONT_SECTION),
        TextColor(UiTheme::muted_cream()),
    )
}

pub fn body_text(text: impl Into<String>) -> impl Bundle {
    (
        Text::new(text.into()),
        TextFont::from_font_size(UiTheme::FONT_BODY),
        TextColor(UiTheme::body()),
    )
}

pub fn caption_text(text: impl Into<String>) -> impl Bundle {
    (
        Text::new(text.into()),
        TextFont::from_font_size(UiTheme::FONT_CAPTION),
        TextColor(UiTheme::body_dim()),
    )
}

pub fn playback_debuff_line_bundle(line: &str) -> impl Bundle {
    let s = debuff_plain_line(line);
    let color = if line.contains("Poison") {
        UiTheme::status_poison()
    } else {
        UiTheme::body_dim()
    };
    (
        Text::new(s),
        TextFont::from_font_size(UiTheme::FONT_CAPTION),
        TextColor(color),
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
    if parts.is_empty() {
        "No stat modifiers.".to_string()
    } else {
        parts.join(" · ")
    }
}

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

pub fn skill_category_chip_colors(cat: crate::domain::skills::SkillCategory) -> (Color, Color) {
    use crate::domain::skills::SkillCategory;
    match cat {
        SkillCategory::BasicAttack => (UiTheme::stone_mid(), UiTheme::muted_cream()),
        SkillCategory::AttackSkill => (
            UiTheme::elite().mix(&UiTheme::void_black(), 0.5),
            UiTheme::muted_gold(),
        ),
        SkillCategory::Buff => (
            Color::srgb(0.22, 0.38, 0.42).mix(&UiTheme::void_black(), 0.35),
            UiTheme::healing(),
        ),
        SkillCategory::Reactive => (
            Color::srgb(0.28, 0.36, 0.55).mix(&UiTheme::void_black(), 0.35),
            Color::srgb(0.65, 0.74, 0.92),
        ),
        SkillCategory::Passive => (UiTheme::stone_deep(), UiTheme::body_dim()),
        SkillCategory::Channel => (
            Color::srgb(0.42, 0.3, 0.52).mix(&UiTheme::void_black(), 0.35),
            Color::srgb(0.82, 0.72, 0.92),
        ),
        SkillCategory::Proc => (
            UiTheme::treasure().mix(&UiTheme::void_black(), 0.55),
            UiTheme::treasure(),
        ),
    }
}

pub fn playback_float_text_color(
    caption: &str,
    anchor: crate::domain::combat::CombatSfxAnchor,
) -> Color {
    let lower = caption.to_ascii_lowercase();
    if lower.contains("recover") || lower.contains("recovers") {
        return UiTheme::healing();
    }

    match anchor {
        crate::domain::combat::CombatSfxAnchor::Enemy => {
            if lower.contains("poison deals") || lower.contains("poison corrodes") {
                return UiTheme::status_poison();
            }
            if lower.contains("thorns bite") {
                return UiTheme::muted_gold();
            }
            if lower.contains("defeated") {
                return UiTheme::treasure();
            }
            if lower.contains("ability") || lower.contains("cleave hits") {
                return UiTheme::muted_gold();
            }
            if lower.contains("strike for") || lower.contains("hits foe") {
                return UiTheme::muted_cream();
            }
            if lower.contains("strike") && lower.contains("white") {
                return UiTheme::muted_cream();
            }
            UiTheme::muted_cream()
        }
        crate::domain::combat::CombatSfxAnchor::Player0
        | crate::domain::combat::CombatSfxAnchor::Player1 => {
            if lower.contains("the foe hits")
                || lower.contains("collapse")
                || lower.contains(" is down")
            {
                return UiTheme::danger();
            }
            if lower.contains("ability") || lower.contains(" white and ") {
                return UiTheme::muted_gold();
            }
            if lower.contains("strike")
                || lower.contains("hits you")
                || lower.contains("hits your")
                || lower.contains("thorns bite")
            {
                return UiTheme::muted_cream();
            }
            if lower.contains("poison") {
                return UiTheme::status_poison();
            }
            if lower.contains("damage") || lower.contains("hits") {
                return UiTheme::danger();
            }
            UiTheme::muted_cream()
        }
        crate::domain::combat::CombatSfxAnchor::Neutral => UiTheme::muted_cream(),
    }
}

#[cfg(test)]
mod panel_style_tests {
    use super::*;

    #[test]
    fn deep_card_uses_deep_panel_background() {
        let style = UiPanelStyle::deep_card();
        assert_eq!(style.background, UiTheme::panel_bg_deep());
        assert_eq!(style.padding_px, UiTheme::PANEL_INSET);
    }
}

#[cfg(test)]
mod playback_float_tests {
    use super::*;
    use crate::domain::combat::CombatSfxAnchor;

    #[test]
    fn float_poison_tick_uses_venom_green_on_foe_anchor() {
        let c =
            playback_float_text_color("Poison deals 3 damage (2 stacks).", CombatSfxAnchor::Enemy);
        assert_eq!(c, UiTheme::status_poison());
    }

    #[test]
    fn float_cleave_splash_is_ability_gold_on_foe_anchor() {
        let c = playback_float_text_color("… Cleave hits foe 2 for 5.", CombatSfxAnchor::Enemy);
        assert_eq!(c, UiTheme::muted_gold());
    }
}

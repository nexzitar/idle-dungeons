use bevy::prelude::*;

use crate::domain::run::{RunSummary, DEFAULT_RUN_MAX_DEPTH};
use crate::ui::theme::{caption_text, UiTheme};

pub fn empty_run_summary() -> RunSummary {
    RunSummary {
        outcome: crate::domain::run::RunOutcome::HeroDied,
        deepest_depth: 0,
        floors_cleared: 0,
        dungeon_depth_cap: DEFAULT_RUN_MAX_DEPTH,
        gold_earned: 0,
        salvage_earned: 0,
        loot: Vec::new(),
        death_reason: Some("No chronicle available.".to_string()),
        log: Vec::new(),
        peak_risk_note: String::new(),
        guided_early_combat_drop_granted: false,
        encounter_score: 0,
        party_strike_damage_white: 0,
        party_strike_damage_yellow: 0,
    }
}

pub fn summary_panel_text(summary: &RunSummary) -> String {
    let mut out = String::new();
    out.push_str(&outcome_headline(summary));
    out.push('\n');
    if !summary.peak_risk_note.is_empty() {
        out.push_str(&summary.peak_risk_note);
        out.push('\n');
    }
    out.push_str(&reward_digest(summary));
    out.push_str("\n\n- Chronicle -\n");
    for line in narrative_highlights(summary) {
        out.push_str(&line);
        out.push('\n');
    }
    if summary.log.len() > 8 {
        out.push_str("\n… full log on the scroll …\n");
    }
    out.push_str("\n- Full log -\n");
    for line in summary.log.iter().take(12) {
        out.push_str(line);
        out.push('\n');
    }
    out
}

pub fn outcome_headline(summary: &RunSummary) -> String {
    match summary.outcome {
        crate::domain::run::RunOutcome::BossDefeated => format!(
            "Victory - reached depth {} before sealing the gate.",
            summary.deepest_depth
        ),
        crate::domain::run::RunOutcome::HeroDied => {
            let reason = summary
                .death_reason
                .clone()
                .unwrap_or_else(|| "The delve ends in darkness.".to_string());
            format!("Fallen - depth {}. {}", summary.deepest_depth, reason)
        }
    }
}

/// Horizontal treasure chips for summary column and rewards modal.
pub fn spawn_treasure_stat_row(parent: &mut ChildSpawnerCommands<'_>, summary: &RunSummary) {
    parent
        .spawn(Node {
            box_sizing: BoxSizing::BorderBox,
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(8.0),
            row_gap: Val::Px(6.0),
            ..default()
        })
        .with_children(|row| {
            spawn_treasure_chip(
                row,
                "Gold",
                &format!("+{}", summary.gold_earned),
                UiTheme::treasure(),
            );
            spawn_treasure_chip(
                row,
                "Salvage",
                &format!("+{}", summary.salvage_earned),
                UiTheme::muted_gold(),
            );
            spawn_treasure_chip(
                row,
                "Score",
                &summary.encounter_score.to_string(),
                UiTheme::body(),
            );
            spawn_treasure_chip(
                row,
                "Depth",
                &summary.deepest_depth.to_string(),
                UiTheme::body_dim(),
            );
        });
    if let Some(pct) = summary.strike_ability_share_percent() {
        parent.spawn(caption_text(format!(
            "Strikes: {}% ability · {} weapon / {} ability",
            pct, summary.party_strike_damage_white, summary.party_strike_damage_yellow
        )));
    }
}

fn spawn_treasure_chip(
    parent: &mut ChildSpawnerCommands<'_>,
    label: &str,
    value: &str,
    accent: Color,
) {
    parent
        .spawn((
            Node {
                box_sizing: BoxSizing::BorderBox,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: Val::Px(2.0),
                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(UiTheme::panel_bg()),
            BorderColor::from(UiTheme::panel_border_inner()),
        ))
        .with_children(|chip| {
            chip.spawn((
                Text::new(label.to_uppercase()),
                TextFont::from_font_size(UiTheme::FONT_MICRO),
                TextColor(UiTheme::body_dim()),
            ));
            chip.spawn((
                Text::new(value),
                TextFont::from_font_size(UiTheme::FONT_SECTION),
                TextColor(accent),
            ));
        });
}

pub fn reward_digest(summary: &RunSummary) -> String {
    let mut out = format!(
        "Rewards pending - Gold +{} · Salvage +{} · Encounter score {} · Loot pieces: {}",
        summary.gold_earned,
        summary.salvage_earned,
        summary.encounter_score,
        summary.loot.len()
    );
    if let Some(pct) = summary.strike_ability_share_percent() {
        use std::fmt::Write;
        let _ = write!(
            &mut out,
            "\nStrike damage: {}% abilities · {} white · {} yellow",
            pct, summary.party_strike_damage_white, summary.party_strike_damage_yellow
        );
    }
    out
}

/// Curated beats for scan-friendly storytelling (subset of the run log).
pub fn narrative_highlights(summary: &RunSummary) -> Vec<String> {
    let mut picks = Vec::new();
    for line in &summary.log {
        let l = line.to_lowercase();
        if l.contains("warden")
            || l.contains("shrine")
            || l.contains("found ")
            || l.contains("elite")
            || l.contains("defeated by")
        {
            picks.push(line.clone());
        }
    }
    if picks.is_empty() {
        summary.log.first().cloned().into_iter().collect()
    } else {
        picks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::run::{RunOutcome, RunSummary, DEFAULT_RUN_MAX_DEPTH};

    #[test]
    fn summary_text_reports_depth_gold_and_outcome() {
        let summary = RunSummary {
            outcome: RunOutcome::HeroDied,
            deepest_depth: 8,
            floors_cleared: 7,
            dungeon_depth_cap: DEFAULT_RUN_MAX_DEPTH,
            gold_earned: 30,
            salvage_earned: 5,
            loot: Vec::new(),
            death_reason: Some("Defeated by Hollow".into()),
            log: vec!["Depth 8: defeated by Hollow".into()],
            peak_risk_note: "Peak room risk: moderate (standard combat).".into(),
            guided_early_combat_drop_granted: false,
            encounter_score: 0,
            party_strike_damage_white: 60,
            party_strike_damage_yellow: 40,
        };

        let text = summary_panel_text(&summary);

        assert!(text.contains("depth 8"));
        assert!(text.contains("Gold +30"));
        assert!(text.contains("Encounter score"));
        assert!(text.contains("40% abilities"));
        assert!(text.contains("60 white"));
        assert!(text.contains("40 yellow"));
        assert!(text.contains("Defeated by Hollow"));
        assert!(text.contains("Peak room risk"));
    }
}

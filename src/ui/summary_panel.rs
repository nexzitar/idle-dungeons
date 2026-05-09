use crate::domain::run::{RunSummary, DEFAULT_RUN_MAX_DEPTH};

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

pub fn reward_digest(summary: &RunSummary) -> String {
    format!(
        "Rewards pending - Gold +{} · Salvage +{} · Loot pieces: {}",
        summary.gold_earned,
        summary.salvage_earned,
        summary.loot.len()
    )
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
        };

        let text = summary_panel_text(&summary);

        assert!(text.contains("depth 8"));
        assert!(text.contains("Gold +30"));
        assert!(text.contains("Defeated by Hollow"));
        assert!(text.contains("Peak room risk"));
    }
}

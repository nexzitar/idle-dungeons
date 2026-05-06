use crate::domain::run::RunSummary;

pub fn summary_panel_text(summary: &RunSummary) -> String {
    let log_preview = if summary.log.is_empty() {
        "No run log.".to_string()
    } else {
        summary
            .log
            .iter()
            .take(6)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "Outcome: {:?}\nDepth {}\nGold {}\nSalvage {}\nLoot {}\n{}\n\nRun Log\n{}",
        summary.outcome,
        summary.deepest_depth,
        summary.gold_earned,
        summary.salvage_earned,
        summary.loot.len(),
        summary
            .death_reason
            .clone()
            .unwrap_or_else(|| "Victory".to_string()),
        log_preview
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::run::{RunOutcome, RunSummary};

    #[test]
    fn summary_text_reports_depth_gold_and_outcome() {
        let summary = RunSummary {
            outcome: RunOutcome::HeroDied,
            deepest_depth: 8,
            gold_earned: 30,
            salvage_earned: 5,
            loot: Vec::new(),
            death_reason: Some("Defeated by Hollow".into()),
            log: vec!["Depth 8: defeated by Hollow".into()],
        };

        let text = summary_panel_text(&summary);

        assert!(text.contains("Depth 8"));
        assert!(text.contains("Gold 30"));
        assert!(text.contains("Defeated by Hollow"));
        assert!(text.contains("Run Log"));
    }
}

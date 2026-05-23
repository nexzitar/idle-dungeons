//! Running-state playback theater sync (bars, floats, meters, log).

use bevy::prelude::*;

use crate::app::ActiveRunPlayback;
use crate::domain::combat::COMBAT_TICK_DISPLAY_SECS;
use crate::domain::run::RunPlaybackFrameKind;
use crate::ui::components::{
    FloatingCombatPopup, PlaybackAggroArrowLine, PlaybackAggroArrowText, PlaybackCaptionText,
    PlaybackCombatLogPanel, PlaybackCombatLogToggleLabel, PlaybackDepthText,
    PlaybackDmgMeterEnemyFill, PlaybackDmgMeterEnemyValue, PlaybackDmgMeterPlayer0Fill,
    PlaybackDmgMeterPlayer0Value, PlaybackDmgMeterPlayer1Fill, PlaybackDmgMeterPlayer1Row,
    PlaybackDmgMeterPlayer1Value, PlaybackEnemyBarFill, PlaybackEnemyDebuffLine,
    PlaybackEnemyNameText, PlaybackEnemyPortraitBlock, PlaybackFoeAltCastFill,
    PlaybackFoeAltCdFill, PlaybackFoeAltTimingRow, PlaybackFoeCastFill, PlaybackFoeCdFill,
    PlaybackLogText, PlaybackPlayer0BarFill, PlaybackPlayer0CastFill, PlaybackPlayer0CdFill,
    PlaybackPlayer0DebuffLine, PlaybackPlayer0InstantRechargeFill, PlaybackPlayer0SkillGcdFill,
    PlaybackPlayer1BarFill, PlaybackPlayer1CastFill, PlaybackPlayer1CdFill,
    PlaybackPlayer1InstantRechargeFill, PlaybackPlayer1PortraitBlock, PlaybackPlayer1SkillGcdFill,
    PlaybackProgressBarFill, PlaybackProgressLabel, PlaybackRoomKindText,
    PlaybackTheaterFloatLayer,
};
use crate::ui::components::RunPlaybackScreen;
use crate::ui::primitives::skill_icon::SkillIconGcdOverlay;
use crate::domain::party::PartyHeroKind;
use crate::ui::theme::UiTheme;
use crate::ui::PlaybackCombatLogVisible;
use crate::ui::FloatingCombatPopupSeq;

pub(crate) fn sync_run_playback_ui(
    playback: Res<ActiveRunPlayback>,
    mut params: ParamSet<(
        Query<&mut Text, With<PlaybackDepthText>>,
        Query<&mut Text, With<PlaybackRoomKindText>>,
        Query<&mut Text, With<PlaybackEnemyNameText>>,
        Query<&mut Text, With<PlaybackCaptionText>>,
        Query<&mut Text, With<PlaybackLogText>>,
        Query<&mut Node, With<PlaybackPlayer0BarFill>>,
        Query<&mut Node, With<PlaybackEnemyBarFill>>,
    )>,
) {
    if playback.frames.is_empty() {
        return;
    }
    let idx = playback.display_index.min(playback.frames.len() - 1);
    let frame = &playback.frames[idx];

    let depth_s = format!("Depth: {}", frame.depth);
    let kind_s = format!(
        "Type: {} · Risk: {}",
        crate::ui::shell::room_kind_label(frame.room_kind),
        frame.risk_hint
    );

    let hero_max_snap = frame.hero_snapshot_max_hp.max(1) as f32;
    let hero_f = (frame.hero_snapshot_hp as f32 / hero_max_snap).clamp(0.0, 1.0);
    let (enemy_f, enemy_name, caption) = match &frame.kind {
        RunPlaybackFrameKind::Narration { text } => (0.0, "—".to_string(), text.clone()),
        RunPlaybackFrameKind::Combat(c) => (
            c.enemy_hp as f32 / c.enemy_max_hp.max(1) as f32,
            c.enemy_name.clone(),
            c.caption.clone(),
        ),
    };

    let log_plain = crate::ui::theme::playback_log_plain(&playback.log_lines);

    for mut text in params.p0().iter_mut() {
        if text.0 != depth_s {
            text.0 = depth_s.clone();
        }
    }
    for mut text in params.p1().iter_mut() {
        if text.0 != kind_s {
            text.0 = kind_s.clone();
        }
    }
    for mut text in params.p2().iter_mut() {
        if text.0 != enemy_name {
            text.0 = enemy_name.clone();
        }
    }
    for mut text in params.p3().iter_mut() {
        if text.0 != caption {
            text.0 = caption.clone();
        }
    }
    for mut text in params.p4().iter_mut() {
        if text.0 != log_plain {
            text.0.clone_from(&log_plain);
        }
    }

    let hero_w = Val::Percent((hero_f * 100.0).clamp(0.0, 100.0));
    let enemy_w = Val::Percent((enemy_f * 100.0).clamp(0.0, 100.0));
    for mut style in params.p5().iter_mut() {
        style.width = hero_w;
    }
    for mut style in params.p6().iter_mut() {
        style.width = enemy_w;
    }
}

pub(crate) fn sync_playback_cast_bars_party(
    playback: Res<ActiveRunPlayback>,
    mut params: ParamSet<(
        Query<&mut Node, With<PlaybackPlayer0CastFill>>,
        Query<&mut Node, With<PlaybackPlayer0CdFill>>,
        Query<&mut Node, With<PlaybackPlayer0SkillGcdFill>>,
        Query<&mut Node, With<PlaybackPlayer0InstantRechargeFill>>,
        Query<&mut Node, With<PlaybackPlayer1CastFill>>,
        Query<&mut Node, With<PlaybackPlayer1CdFill>>,
        Query<&mut Node, With<PlaybackPlayer1SkillGcdFill>>,
        Query<&mut Node, With<PlaybackPlayer1InstantRechargeFill>>,
    )>,
) {
    if playback.frames.is_empty() {
        return;
    }
    let idx = playback.display_index.min(playback.frames.len() - 1);
    let f = &playback.frames[idx];
    let crate::domain::run::RunPlaybackFrameKind::Combat(c) = &f.kind else {
        return;
    };
    let lc = Val::Percent((c.player0_cast * 100.0).clamp(0.0, 100.0));
    let lcdn = Val::Percent((c.player0_cd * 100.0).clamp(0.0, 100.0));
    let lsg = Val::Percent((c.player0_skill_gcd * 100.0).clamp(0.0, 100.0));
    let lir = Val::Percent((c.player0_instant_recharge * 100.0).clamp(0.0, 100.0));
    let ac = Val::Percent((c.player1_cast * 100.0).clamp(0.0, 100.0));
    let acdn = Val::Percent((c.player1_cd * 100.0).clamp(0.0, 100.0));
    let asg = Val::Percent((c.player1_skill_gcd * 100.0).clamp(0.0, 100.0));
    let air = Val::Percent((c.player1_instant_recharge * 100.0).clamp(0.0, 100.0));
    for mut n in params.p0().iter_mut() {
        n.width = lc;
    }
    for mut n in params.p1().iter_mut() {
        n.width = lcdn;
    }
    for mut n in params.p2().iter_mut() {
        n.width = lsg;
    }
    for mut n in params.p3().iter_mut() {
        n.width = lir;
    }
    for mut n in params.p4().iter_mut() {
        n.width = ac;
    }
    for mut n in params.p5().iter_mut() {
        n.width = acdn;
    }
    for mut n in params.p6().iter_mut() {
        n.width = asg;
    }
    for mut n in params.p7().iter_mut() {
        n.width = air;
    }
}

pub(crate) fn sync_playback_cast_bars_foe(
    playback: Res<ActiveRunPlayback>,
    mut params: ParamSet<(
        Query<&mut Node, With<PlaybackFoeCastFill>>,
        Query<&mut Node, With<PlaybackFoeCdFill>>,
        Query<&mut Node, With<PlaybackFoeAltCastFill>>,
        Query<&mut Node, With<PlaybackFoeAltCdFill>>,
        Query<&mut Visibility, With<PlaybackFoeAltTimingRow>>,
    )>,
) {
    if playback.frames.is_empty() {
        return;
    }
    let idx = playback.display_index.min(playback.frames.len() - 1);
    let f = &playback.frames[idx];
    let crate::domain::run::RunPlaybackFrameKind::Combat(c) = &f.kind else {
        return;
    };
    let fc = Val::Percent((c.foe_cast * 100.0).clamp(0.0, 100.0));
    let fcdn = Val::Percent((c.foe_cd * 100.0).clamp(0.0, 100.0));
    for mut n in params.p0().iter_mut() {
        n.width = fc;
    }
    for mut n in params.p1().iter_mut() {
        n.width = fcdn;
    }
    let fca = Val::Percent((c.foe_alt_cast * 100.0).clamp(0.0, 100.0));
    let fcda = Val::Percent((c.foe_alt_cd * 100.0).clamp(0.0, 100.0));
    for mut n in params.p2().iter_mut() {
        n.width = fca;
    }
    for mut n in params.p3().iter_mut() {
        n.width = fcda;
    }
    let show = c.enemy_name.contains(" · ");
    let v = if show {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut vis in params.p4().iter_mut() {
        if *vis != v {
            *vis = v;
        }
    }
}

pub(crate) fn sync_run_playback_party_bars(
    playback: Res<ActiveRunPlayback>,
    mut player1_bar: Query<&mut Node, With<PlaybackPlayer1BarFill>>,
) {
    if playback.frames.is_empty() {
        return;
    }
    let idx = playback.display_index.min(playback.frames.len() - 1);
    let frame = &playback.frames[idx];

    let ally_w = match (frame.partner_snapshot_hp, frame.partner_snapshot_max_hp) {
        (Some(h), Some(m)) if m > 0 => {
            Val::Percent(((h as f32 / m as f32).clamp(0.0, 1.0) * 100.0).clamp(0.0, 100.0))
        }
        _ => Val::Percent(0.0),
    };
    for mut style in &mut player1_bar {
        style.width = ally_w;
    }
}

pub(crate) fn sync_playback_aggro_arrow(
    playback: Res<ActiveRunPlayback>,
    mut aggro: Query<&mut Text, With<PlaybackAggroArrowText>>,
) {
    if playback.frames.is_empty() {
        return;
    }
    let idx = playback.display_index.min(playback.frames.len() - 1);
    let frame = &playback.frames[idx];
    let arrow_s = match &frame.kind {
        RunPlaybackFrameKind::Narration { .. } => "\u{2014}".to_string(),
        RunPlaybackFrameKind::Combat(c) => {
            let has_partner = frame.partner_snapshot_max_hp.is_some();
            let t0 = c.threat_slot0.unwrap_or(0);
            let t1 = c.threat_slot1.unwrap_or(0);
            let label = crate::domain::combat::aggro_arrow_target_label(
                [t0, t1],
                frame.hero_snapshot_hp,
                frame.partner_snapshot_hp.unwrap_or(0),
                has_partner,
                c.foe_last_target,
            );
            format!("\u{2192} {label}")
        }
    };
    for mut text in &mut aggro {
        if text.0 != arrow_s {
            text.0 = arrow_s.clone();
        }
    }
}

pub(crate) fn sync_playback_theater_slot_visibility(
    playback: Res<ActiveRunPlayback>,
    mut ally: Query<
        &mut Visibility,
        (
            With<PlaybackPlayer1PortraitBlock>,
            Without<PlaybackEnemyPortraitBlock>,
        ),
    >,
    mut enemy: Query<
        &mut Visibility,
        (
            With<PlaybackEnemyPortraitBlock>,
            Without<PlaybackPlayer1PortraitBlock>,
        ),
    >,
) {
    if playback.frames.is_empty() {
        return;
    }
    let idx = playback.display_index.min(playback.frames.len() - 1);
    let frame = &playback.frames[idx];
    let show_ally = frame.partner_snapshot_max_hp.is_some();
    let show_enemy = matches!(frame.kind, RunPlaybackFrameKind::Combat(_));
    for mut v in &mut ally {
        *v = if show_ally {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for mut v in &mut enemy {
        *v = if show_enemy {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub(crate) fn sync_playback_aggro_arrow_line(
    playback: Res<ActiveRunPlayback>,
    mut q: Query<(&mut Node, &mut Visibility), With<PlaybackAggroArrowLine>>,
) {
    if playback.frames.is_empty() {
        return;
    }
    let idx = playback.display_index.min(playback.frames.len() - 1);
    let frame = &playback.frames[idx];
    let Ok((mut style, mut vis)) = q.single_mut() else {
        return;
    };
    match &frame.kind {
        RunPlaybackFrameKind::Narration { .. } => {
            *vis = Visibility::Hidden;
        }
        RunPlaybackFrameKind::Combat(c) => {
            let has_partner = frame.partner_snapshot_max_hp.is_some();
            let t0 = c.threat_slot0.unwrap_or(0);
            let t1 = c.threat_slot1.unwrap_or(0);
            let slot = crate::domain::combat::pick_party_enemy_target(
                0,
                [t0, t1],
                frame.hero_snapshot_hp,
                frame.partner_snapshot_hp.unwrap_or(0),
                has_partner,
                c.foe_last_target,
            );
            let top = if slot == 0 { 30.0 } else { 58.0 };
            *vis = Visibility::Visible;
            style.position_type = PositionType::Absolute;
            style.top = Val::Percent(top);
            style.right = Val::Percent(14.0);
            style.width = Val::Percent(44.0);
            style.height = Val::Px(4.0);
            style.left = Val::Auto;
            style.bottom = Val::Auto;
        }
    }
}

pub(crate) fn sync_playback_damage_meters(
    playback: Res<ActiveRunPlayback>,
    mut player1_row: Query<&mut Visibility, With<PlaybackDmgMeterPlayer1Row>>,
    mut fills: ParamSet<(
        Query<&mut Node, With<PlaybackDmgMeterPlayer0Fill>>,
        Query<&mut Node, With<PlaybackDmgMeterPlayer1Fill>>,
        Query<&mut Node, With<PlaybackDmgMeterEnemyFill>>,
    )>,
    mut vals: ParamSet<(
        Query<&mut Text, With<PlaybackDmgMeterPlayer0Value>>,
        Query<&mut Text, With<PlaybackDmgMeterPlayer1Value>>,
        Query<&mut Text, With<PlaybackDmgMeterEnemyValue>>,
    )>,
) {
    if playback.frames.is_empty() {
        return;
    }
    let idx = playback.display_index.min(playback.frames.len() - 1);
    let frames = playback.frames.as_slice();
    let mut combat: Option<&crate::domain::combat::CombatPlaybackFrame> = None;
    for f in frames[..=idx].iter().rev() {
        if let RunPlaybackFrameKind::Combat(c) = &f.kind {
            combat = Some(c);
            break;
        }
    }
    let has_partner = frames[idx].partner_snapshot_max_hp.is_some();
    let secs = COMBAT_TICK_DISPLAY_SECS.max(0.001);
    let fmt = |dmg: u32, ticks: u32| -> String {
        let t = ticks.max(1) as f32 * secs;
        let dps = dmg as f32 / t;
        format!("{} · {:.1}/s", dmg, dps)
    };
    let (p0, p1, fe, tk) = match combat {
        Some(c) => (
            c.damage_meter_party_0,
            c.damage_meter_party_1,
            c.damage_meter_foe,
            c.run_sim_ticks,
        ),
        None => (0u32, 0u32, 0u32, 1u32),
    };
    let max = p0.max(p1).max(fe).max(1) as f32;
    let w0 = (p0 as f32 / max * 100.0).clamp(0.0, 100.0);
    let w1 = (p1 as f32 / max * 100.0).clamp(0.0, 100.0);
    let wf = (fe as f32 / max * 100.0).clamp(0.0, 100.0);
    for mut s in fills.p0().iter_mut() {
        s.width = Val::Percent(w0);
    }
    for mut s in fills.p1().iter_mut() {
        s.width = Val::Percent(w1);
    }
    for mut s in fills.p2().iter_mut() {
        s.width = Val::Percent(wf);
    }
    let s0 = fmt(p0, tk);
    let s1 = fmt(p1, tk);
    let sf = fmt(fe, tk);
    for mut t in vals.p0().iter_mut() {
        if t.0 != s0 {
            t.0.clone_from(&s0);
        }
    }
    for mut t in vals.p1().iter_mut() {
        if t.0 != s1 {
            t.0.clone_from(&s1);
        }
    }
    for mut t in vals.p2().iter_mut() {
        if t.0 != sf {
            t.0.clone_from(&sf);
        }
    }
    for mut v in &mut player1_row {
        *v = if has_partner {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub(crate) fn sync_playback_combat_log_panel_visibility(
    vis: Res<PlaybackCombatLogVisible>,
    mut panel: Query<&mut Visibility, With<PlaybackCombatLogPanel>>,
) {
    let v = if vis.0 {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut pv in &mut panel {
        *pv = v;
    }
}

pub(crate) fn sync_combat_log_toggle_label(
    vis: Res<PlaybackCombatLogVisible>,
    mut labels: Query<&mut Text, With<PlaybackCombatLogToggleLabel>>,
) {
    let s = if vis.0 { "Hide log" } else { "Show log" };
    for mut text in &mut labels {
        if text.0 != s {
            text.0 = s.to_string();
        }
    }
}

pub(crate) fn spawn_playback_floating_combat_text(
    playback: Res<ActiveRunPlayback>,
    mut last_idx: Local<Option<usize>>,
    float_layer: Query<Entity, With<PlaybackTheaterFloatLayer>>,
    mut seq: ResMut<FloatingCombatPopupSeq>,
    mut commands: Commands,
) {
    if playback.frames.is_empty() {
        return;
    }
    let idx = playback.display_index.min(playback.frames.len() - 1);
    if *last_idx == Some(idx) {
        return;
    }
    *last_idx = Some(idx);

    let frame = &playback.frames[idx];
    let (caption, anchor) = match &frame.kind {
        RunPlaybackFrameKind::Combat(c) => (c.caption.clone(), c.sfx_anchor),
        _ => return,
    };
    if matches!(anchor, crate::domain::combat::CombatSfxAnchor::Neutral) {
        return;
    }
    let Ok(parent) = float_layer.single() else {
        return;
    };
    let color = crate::ui::theme::playback_float_text_color(&caption, anchor);
    let lower = caption.to_ascii_lowercase();
    let font_size = if lower.contains("ability damage") || lower.contains(" white and ") {
        UiTheme::FONT_SUBLINE
    } else {
        UiTheme::FONT_COMPACT
    };
    let mut pos = Node {
        box_sizing: BoxSizing::BorderBox,
        position_type: PositionType::Absolute,
        max_width: Val::Px(200.0),
        padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    match anchor {
        crate::domain::combat::CombatSfxAnchor::Player0 => {
            pos.left = Val::Percent(4.0);
            pos.right = Val::Auto;
            pos.top = Val::Percent(10.0);
        }
        crate::domain::combat::CombatSfxAnchor::Player1 => {
            pos.left = Val::Percent(4.0);
            pos.right = Val::Auto;
            pos.top = Val::Percent(52.0);
        }
        crate::domain::combat::CombatSfxAnchor::Enemy => {
            pos.right = Val::Percent(4.0);
            pos.left = Val::Auto;
            pos.top = Val::Percent(28.0);
        }
        crate::domain::combat::CombatSfxAnchor::Neutral => return,
    }

    seq.0 = seq.0.wrapping_add(1);
    let my_seq = seq.0;

    commands.entity(parent).with_children(|layer| {
        layer
            .spawn((
                pos,
                FloatingCombatPopup {
                    ttl: 0.88,
                    seq: my_seq,
                },
            ))
            .with_children(|pop| {
                pop.spawn((
                    Text::new(caption),
                    TextFont::from_font_size(font_size),
                    TextColor(color),
                ));
            });
    });
}

pub(crate) fn tick_floating_combat_popups(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut FloatingCombatPopup)>,
) {
    let dt = time.delta_secs();
    let mut v: Vec<(u32, Entity, f32)> = Vec::new();
    for (entity, mut pop) in &mut q {
        pop.ttl -= dt;
        v.push((pop.seq, entity, pop.ttl));
    }
    v.sort_by_key(|(s, _, _)| *s);
    let remove = v.len().saturating_sub(8);
    for i in 0..remove {
        commands.entity(v[i].1).despawn();
    }
    for (_, entity, ttl) in v.into_iter().skip(remove) {
        if ttl <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub(crate) fn sync_run_playback_debuff_slots(
    playback: Res<ActiveRunPlayback>,
    mut hero: Query<
        &mut Text,
        (
            With<PlaybackPlayer0DebuffLine>,
            Without<PlaybackEnemyDebuffLine>,
        ),
    >,
    mut foe: Query<
        &mut Text,
        (
            With<PlaybackEnemyDebuffLine>,
            Without<PlaybackPlayer0DebuffLine>,
        ),
    >,
) {
    if playback.frames.is_empty() {
        return;
    }
    let idx = playback.display_index.min(playback.frames.len() - 1);
    let frame = &playback.frames[idx];
    const EMPTY_DEBUFF: &str = "—  ·  —  ·  —  ·  —";
    let (hero_line, foe_line) = match &frame.kind {
        RunPlaybackFrameKind::Narration { .. } => {
            (EMPTY_DEBUFF.to_string(), EMPTY_DEBUFF.to_string())
        }
        RunPlaybackFrameKind::Combat(c) => (
            c.hero_debuff_slots.join("  ·  "),
            c.enemy_debuff_slots.join("  ·  "),
        ),
    };
    let hs = crate::ui::theme::playback_debuff_line_string(&hero_line);
    let fs = crate::ui::theme::playback_debuff_line_string(&foe_line);
    for mut text in &mut hero {
        if text.0 != hs {
            text.0.clone_from(&hs);
        }
    }
    for mut text in &mut foe {
        if text.0 != fs {
            text.0.clone_from(&fs);
        }
    }
}

pub(crate) fn sync_playback_delve_progress_bar(
    playback: Res<ActiveRunPlayback>,
    mut fill: Query<&mut Node, With<PlaybackProgressBarFill>>,
    mut label: Query<&mut Text, With<PlaybackProgressLabel>>,
) {
    if playback.frames.is_empty() {
        return;
    }
    let idx = playback.display_index.min(playback.frames.len() - 1);
    let frame = &playback.frames[idx];
    let cap = frame.delve_floors_cap.max(1);
    let cleared = frame.delve_floors_cleared;
    let frac = (cleared as f32 / cap as f32).clamp(0.0, 1.0);
    for mut style in &mut fill {
        style.width = Val::Percent(frac * 100.0);
    }
    let line = format!("Floors cleared: {cleared} / {cap}");
    for mut text in &mut label {
        if text.0 != line {
            text.0 = line.clone();
        }
    }
}

/// Maps aggregate hero GCD from playback frames onto theater skill icon overlays.
pub(crate) fn sync_playback_skill_icon_overlays(
    playback: Res<ActiveRunPlayback>,
    running: Query<(), With<RunPlaybackScreen>>,
    mut overlays: Query<(&SkillIconGcdOverlay, &mut Node, &mut Visibility)>,
) {
    if running.is_empty() {
        return;
    }
    let mut hide_all = || {
        for (_, mut node, mut vis) in &mut overlays {
            *vis = Visibility::Hidden;
            node.height = Val::Percent(0.0);
        }
    };
    if playback.frames.is_empty() {
        hide_all();
        return;
    }
    let idx = playback.display_index.min(playback.frames.len() - 1);
    let frame = &playback.frames[idx];
    let crate::domain::run::RunPlaybackFrameKind::Combat(c) = &frame.kind else {
        hide_all();
        return;
    };
    for (slot, mut node, mut vis) in &mut overlays {
        let frac = match slot.hero {
            PartyHeroKind::Player1 => c.player0_skill_gcd,
            PartyHeroKind::Player2 => c.player1_skill_gcd,
        };
        if frac <= 0.001 {
            *vis = Visibility::Hidden;
            node.height = Val::Percent(0.0);
        } else {
            *vis = Visibility::Visible;
            node.height = Val::Percent((frac * 100.0).clamp(0.0, 100.0));
        }
    }
}

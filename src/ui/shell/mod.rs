//! Game shell scaffolding: ornate panels, header, dungeon columns, playback theater, footer dock.
//!
//! Game shell layout (header, theater columns, footer). Spawn API re-exported from submodules.

mod footer;
mod hero_column;
mod layout;
mod playback_bars;
mod theater;

pub use footer::{
    spawn_mockup_footer, spawn_stash_filters_and_sort_row, spawn_summary_rewards_modal,
    FooterMode,
};
pub use hero_column::{spawn_hero_column, HeroColumnConfig};
pub use layout::{
    fmt_speed_label, mockup_gear_cards, room_kind_label, spawn_dungeon_briefing_column,
    spawn_dungeon_camp_column, spawn_dungeon_summary_column, spawn_mockup_header,
    spawn_ornate_column, spawn_settings_modal, spawn_three_column_row,
    title_settings_menu_button,
};
pub use theater::spawn_run_playback_middle_column;

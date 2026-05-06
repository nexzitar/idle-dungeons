pub mod build_panel;
pub mod inventory_panel;
pub mod log_panel;
pub mod run_panel;
pub mod summary_panel;
pub mod upgrade_panel;

use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, _app: &mut App) {}
}

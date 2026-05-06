use bevy::prelude::*;

pub struct IdleDungeonsPlugin;

impl Plugin for IdleDungeonsPlugin {
    fn build(&self, _app: &mut App) {}
}

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(IdleDungeonsPlugin)
        .run();
}

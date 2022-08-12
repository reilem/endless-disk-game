use bevy::prelude::*;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, _app: &mut App) {
        #[cfg(debug_assertions)]
        {
            use crate::{player::Player, world::TileMap};
            use bevy_inspector_egui::{RegisterInspectable, WorldInspectorPlugin};
            _app.add_plugin(WorldInspectorPlugin::new())
                .register_inspectable::<Player>()
                .register_inspectable::<TileMap>();
        }
    }
}

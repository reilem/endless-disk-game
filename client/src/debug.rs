use bevy::prelude::*;

use crate::{player::Player, world::TileMap};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, _app: &mut App) {
        // TODO: Currently doesn't work as an optional
        // IDEA: Use profiles instead of optional features
        #[cfg(feature = "bevy-inspector-egui")]
        {
            use bevy_inspector_egui::{RegisterInspectable, WorldInspectorPlugin};
            _app.add_plugin(WorldInspectorPlugin::new())
                .register_inspectable::<Player>()
                .register_inspectable::<TileMap>();
        }
    }
}

use bevy::prelude::*;
use bevy_inspector_egui::RegisterInspectable;

use crate::{player::Player, world::TileMap};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, _app: &mut App) {
        #[cfg(feature = "bevy-inspector-egui")]
        {
            use bevy_inspector_egui::WorldInspectorPlugin;
            _app.add_plugin(WorldInspectorPlugin::new())
                .register_inspectable::<Player>()
                .register_inspectable::<TileMap>();
        }
    }
}

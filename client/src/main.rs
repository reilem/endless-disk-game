use bevy::prelude::*;

pub fn main() {
    bevy::log::info!("Hello");
    App::new().add_plugins(DefaultPlugins).run();
}

use bevy::{
    prelude::*,
    window::{PresentMode, WindowMode},
};
use endless_game::{
    debug::DebugPlugin, player::PlayerPlugin, texture::TexturePlugin, world::WorldPlugin,
};

const BACKGROUND: Color = Color::rgb(0.2, 0.2, 0.2);

pub fn main() {
    bevy::log::info!("Hello");
    App::new()
        .insert_resource(ClearColor(BACKGROUND))
        .insert_resource(init_window())
        .add_startup_system(init_camera)
        .add_plugin(TexturePlugin)
        .add_plugin(WorldPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugPlugin)
        .run();
}

fn init_window() -> WindowDescriptor {
    WindowDescriptor {
        width: 1024.0,
        height: 768.0,
        title: "Endless".to_string(),
        present_mode: PresentMode::AutoVsync,
        resizable: true,
        decorations: true,
        cursor_visible: true,
        cursor_locked: false,
        mode: WindowMode::Windowed,
        transparent: false,
        canvas: None,
        fit_canvas_to_parent: true,
        ..default()
    }
}

fn init_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

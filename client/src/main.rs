use bevy::{
    prelude::*,
    render::{
        render_resource::{FilterMode, SamplerDescriptor},
        texture::ImageSampler,
    },
    window::{PresentMode, WindowMode},
};
use endless_game::{player::PlayerPlugin, TextureSheet};

const BACKGROUND: Color = Color::rgb(0.2, 0.2, 0.2);

pub fn main() {
    bevy::log::info!("Hello");
    App::new()
        .insert_resource(ClearColor(BACKGROUND))
        .insert_resource(init_window())
        .add_startup_system_to_stage(StartupStage::PreStartup, init_textures)
        .add_startup_system(init_camera)
        .add_system(set_texture_filters_to_nearest)
        .add_plugin(PlayerPlugin)
        .add_plugins(DefaultPlugins)
        .run();
}

fn init_window() -> WindowDescriptor {
    WindowDescriptor {
        width: 1024.0,
        height: 768.0,
        position: WindowPosition::Centered(MonitorSelection::Current),
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
        ..Default::default()
    }
}

fn init_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

fn init_textures(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlasses: ResMut<Assets<TextureAtlas>>,
) {
    let image: Handle<Image> = assets.load("atlas-1.png"); // TODO: Pad the tiles if there is bleeding
    let atlas = TextureAtlas::from_grid(image, Vec2::splat(32.0), 3, 1);
    let atlas_handle = texture_atlasses.add(atlas);
    commands.insert_resource(TextureSheet { atlas_handle });
}

fn set_texture_filters_to_nearest(
    mut texture_events: EventReader<AssetEvent<Image>>,
    mut textures: ResMut<Assets<Image>>,
) {
    for event in texture_events.iter() {
        if let AssetEvent::Created { handle } = event {
            if let Some(mut texture) = textures.get_mut(handle) {
                println!("Change texture filter!");

                texture.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
                    min_filter: FilterMode::Nearest,
                    mag_filter: FilterMode::Nearest,
                    mipmap_filter: FilterMode::Nearest,
                    ..default()
                })
            }
        }
    }
}

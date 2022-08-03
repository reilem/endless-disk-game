use bevy::{
    prelude::*,
    window::{PresentMode, WindowMode},
};

struct TextureSheet {
    atlas_handle: Handle<TextureAtlas>,
}

const BACKGROUND: Color = Color::rgb(0.2, 0.2, 0.2);

pub fn main() {
    bevy::log::info!("Hello");
    App::new()
        .insert_resource(ClearColor(BACKGROUND))
        .insert_resource(init_window())
        .add_startup_system_to_stage(StartupStage::PreStartup, init_textures)
        .add_startup_system(init_camera)
        .add_startup_system(spawn_player)
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

fn spawn_player(mut commands: Commands, textures: Res<TextureSheet>) {
    let mut sprite = TextureAtlasSprite::new(2);
    sprite.custom_size = Some(Vec2 { x: 64.0, y: 64.0 });
    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite,
            texture_atlas: textures.atlas_handle.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 900.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Name::new("Player"));
}

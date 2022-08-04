use bevy::prelude::*;

use crate::TextureSheet;

#[derive(Component)]
pub struct Player;

pub struct PlayerPlugin;

const SPEED: f32 = 100.0;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_player);
        app.add_system(player_movement);
    }
}

fn player_movement(
    mut player_query: Query<(&Player, &mut Transform)>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (_player, mut transform) = player_query
        .get_single_mut()
        .unwrap_or_else(|err| panic!("Failed to get player {:?}", err));

    if keyboard.any_pressed([KeyCode::W, KeyCode::Up]) {
        transform.translation.y += SPEED * time.delta_seconds();
    }
    if keyboard.any_pressed([KeyCode::S, KeyCode::Down]) {
        transform.translation.y -= SPEED * time.delta_seconds();
    }
    if keyboard.any_pressed([KeyCode::A, KeyCode::Left]) {
        transform.translation.x -= SPEED * time.delta_seconds();
    }
    if keyboard.any_pressed([KeyCode::D, KeyCode::Right]) {
        transform.translation.x += SPEED * time.delta_seconds();
    }
}

fn spawn_player(mut commands: Commands, textures: Res<TextureSheet>) {
    let sprite = TextureAtlasSprite::new(2);

    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite,
            texture_atlas: textures.atlas_handle.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 900.0),
                scale: Vec3::splat(4.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Name::new("Player"))
        .insert(Player);
}

use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;

use crate::{sprite::spawn_sprite, texture::TextureSheet, TILE_SIZE};

#[derive(Component, Inspectable)]
pub struct Player {
    speed: f32,
}

pub struct PlayerPlugin;

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
    // TODO: Track player with camera
    let (player, mut transform) = player_query
        .get_single_mut()
        .unwrap_or_else(|err| panic!("Failed to get player {:?}", err));

    if keyboard.any_pressed([KeyCode::W, KeyCode::Up]) {
        transform.translation.y += player.speed * TILE_SIZE * time.delta_seconds();
    }
    if keyboard.any_pressed([KeyCode::S, KeyCode::Down]) {
        transform.translation.y -= player.speed * TILE_SIZE * time.delta_seconds();
    }
    if keyboard.any_pressed([KeyCode::A, KeyCode::Left]) {
        transform.translation.x -= player.speed * TILE_SIZE * time.delta_seconds();
    }
    if keyboard.any_pressed([KeyCode::D, KeyCode::Right]) {
        transform.translation.x += player.speed * TILE_SIZE * time.delta_seconds();
    }
}

fn spawn_player(mut commands: Commands, texture_sheet: Res<TextureSheet>) {
    let player_sprite = spawn_sprite(&mut commands, &texture_sheet, 2, Vec3::new(0.0, 0.0, 900.0));
    commands
        .entity(player_sprite)
        .insert(Name::new("Player"))
        .insert(Player { speed: 1.5 });
}

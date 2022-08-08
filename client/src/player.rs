use bevy::{prelude::*, sprite::collide_aabb::collide};
use bevy_inspector_egui::Inspectable;

use crate::{
    sprite::{spawn_sprite, sprite_size},
    texture::TextureSheet,
    world::{update_tile_background, TileCollider, TileMap, WorldTile},
    TILE_SIZE,
};

#[derive(Component, Inspectable)]
pub struct Player {
    speed: f32,
    just_moved: bool,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_player);
        app.add_system(player_movement);
        app.add_system(camera_movement.after(player_movement));
        app.add_system(movement_side_effects.after(camera_movement));
    }
}

type WallQuery<'world, 'state, 't> =
    Query<'world, 'state, (&'t Transform, (With<TileCollider>, Without<Player>))>;

fn camera_movement(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    let player_transform = player_query
        .get_single()
        .unwrap_or_else(|err| panic!("Failed to get player in camera_movement: {:?}", err));
    let mut camera_transform = camera_query
        .get_single_mut()
        .unwrap_or_else(|err| panic!("Failed to get camera in camera_movement: {:?}", err));

    camera_transform.translation.x = player_transform.translation.x;
    camera_transform.translation.y = player_transform.translation.y;
}

fn player_movement(
    mut player_query: Query<(&mut Player, &mut Transform)>,
    wall_query: WallQuery, // Cannot request same entity from two queries, so we must exclude player component
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (mut player, mut transform) = player_query
        .get_single_mut()
        .unwrap_or_else(|err| panic!("Failed to get player {:?}", err));

    let mut y = 0.0;
    if keyboard.any_pressed([KeyCode::W, KeyCode::Up]) {
        y = player.speed * TILE_SIZE * time.delta_seconds();
    }
    if keyboard.any_pressed([KeyCode::S, KeyCode::Down]) {
        y = -1.0 * player.speed * TILE_SIZE * time.delta_seconds();
    }
    let mut x = 0.0;
    if keyboard.any_pressed([KeyCode::A, KeyCode::Left]) {
        x = -1.0 * player.speed * TILE_SIZE * time.delta_seconds();
    }
    if keyboard.any_pressed([KeyCode::D, KeyCode::Right]) {
        x = player.speed * TILE_SIZE * time.delta_seconds();
    }
    if x != 0.0 || y != 0.0 {
        let next_x_position = transform.translation + Vec3 { x, y: 0.0, z: 0.0 };
        if !will_collide(next_x_position, &wall_query) {
            transform.translation.x = next_x_position.x;
            player.just_moved = true;
        }
        let next_y_position = transform.translation + Vec3 { x: 0.0, y, z: 0.0 };
        if !will_collide(next_y_position, &wall_query) {
            transform.translation.y = next_y_position.y;
            player.just_moved = true;
        }
    }
}

fn will_collide(position: Vec3, wall_query: &WallQuery) -> bool {
    for (wall_transform, _) in wall_query.iter() {
        if collide(
            position,
            sprite_size(2),
            wall_transform.translation,
            sprite_size(1),
        )
        .is_some()
        {
            return true;
        }
    }
    false
}

fn spawn_player(mut commands: Commands, texture_sheet: Res<TextureSheet>) {
    let player_sprite = spawn_sprite(&mut commands, &texture_sheet, 2, Vec3::new(0.0, 0.0, 900.0));
    commands
        .entity(player_sprite)
        .insert(Name::new("Player"))
        .insert(Player {
            speed: 1.5,
            just_moved: false,
        });
}

fn movement_side_effects(
    mut player_query: Query<&mut Player>,
    camera_query: Query<&Transform, With<Camera>>,
    map_query: Query<&TileMap>,
    tile_query: Query<&mut Transform, (With<WorldTile>, Without<Camera>)>,
) {
    let mut player = player_query
        .get_single_mut()
        .unwrap_or_else(|err| panic!("Failed to get player {:?}", err));
    if player.just_moved {
        update_tile_background(camera_query, map_query, tile_query);
        player.just_moved = false;
    }
}

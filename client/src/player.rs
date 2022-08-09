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
        app.add_system(mouse_movement);
        app.add_system(keyboard_movement);
        app.add_system(
            camera_movement
                .after(keyboard_movement)
                .after(mouse_movement),
        );
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

fn keyboard_movement(
    mut player_query: Query<(&mut Player, &mut Transform)>,
    wall_query: WallQuery, // Cannot request same entity from two queries, so we must exclude player component
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let ds = time.delta_seconds();
    let (mut player, mut transform) = player_query
        .get_single_mut()
        .unwrap_or_else(|err| panic!("Failed to get player {:?}", err));
    // TODO: Ensure diagonal movement speed is same as horizontal/vertical
    let mut y = 0.0;
    if keyboard.any_pressed([KeyCode::W, KeyCode::Up]) {
        y = displacement(player.speed, ds);
    }
    if keyboard.any_pressed([KeyCode::S, KeyCode::Down]) {
        y = -1.0 * displacement(player.speed, ds);
    }
    let mut x = 0.0;
    if keyboard.any_pressed([KeyCode::A, KeyCode::Left]) {
        x = -1.0 * displacement(player.speed, ds);
    }
    if keyboard.any_pressed([KeyCode::D, KeyCode::Right]) {
        x = displacement(player.speed, ds);
    }
    move_player(x, y, &mut player, &mut transform, wall_query);
}

fn mouse_movement(
    mut player_query: Query<(&mut Player, &mut Transform)>,
    wall_query: WallQuery,
    buttons: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    time: Res<Time>,
) {
    if !buttons.pressed(MouseButton::Left) {
        return;
    }

    let (mut player, mut transform) = player_query
        .get_single_mut()
        .unwrap_or_else(|err| panic!("Failed to get player {:?}", err));

    let window = windows.get_primary().unwrap();
    let width = window.width();
    let height = window.height();
    let mid_x = width / 2.0;
    let mid_y = height / 2.0;

    if let Some(position) = window.cursor_position() {
        let delta_space = displacement(player.speed, time.delta_seconds());
        // Project the coordinates into normal space
        // Old space: bottom-left: (0,0), top-right: (width, height)
        // Normal space: bottom-left (-1,-1), top-right: (1,1)
        let normal_position = Vec2 {
            x: (position.x - mid_x) / mid_x,
            y: (position.y - mid_y) / mid_y,
        };
        // Cursor deadzone is at -0.25 to 0.25
        // If the cursor is in this deadzone the sprite will move slower the closer the cursor is
        // and faster the further away the cursor is. Beyond the deadzone the sprite will move at max speed.
        let deadzone_percentage = 0.25;
        // Weights will either be 1 or -1 when mouse is beyond deadzone, or between 0 and 1(or -1) when within deadzone
        // Closer to player = smaller weight
        let weight_x = (normal_position.x / deadzone_percentage).min(1.0).max(-1.0);
        let weight_y = (normal_position.y / deadzone_percentage).min(1.0).max(-1.0);
        // Calculate the strength of the x and y movement
        // Diagonal: both = 0.5. Horizontal: strength_x = 1. Vertical: strength_y = 1
        let alpha = normal_position.y / normal_position.x;
        let strength_x = 1.0 / (1.0 + alpha.abs());
        let strength_y = 1.0 - strength_x;
        // Multiply the weights by the strengths to find the true x and y movement components
        let x_component = strength_x * weight_x;
        let y_component = strength_y * weight_y;
        // Multiple the vector components by the maximum displacement to find the x and y displacement
        let x = x_component * delta_space;
        let y = y_component * delta_space;
        move_player(x, y, &mut player, &mut transform, wall_query);
    }
}

fn displacement(speed: f32, time: f32) -> f32 {
    speed * TILE_SIZE * time
}

fn move_player(
    x: f32,
    y: f32,
    player: &mut Player,
    player_transform: &mut Transform,
    wall_query: WallQuery,
) {
    if x == 0.0 && y == 0.0 {
        return;
    }
    let next_x_position = player_transform.translation + Vec3 { x, y: 0.0, z: 0.0 };
    if !will_collide(next_x_position, &wall_query) {
        player_transform.translation.x = next_x_position.x;
        player.just_moved = true;
    }
    let next_y_position = player_transform.translation + Vec3 { x: 0.0, y, z: 0.0 };
    if !will_collide(next_y_position, &wall_query) {
        player_transform.translation.y = next_y_position.y;
        player.just_moved = true;
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
            speed: 2.0,
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

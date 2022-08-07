use bevy::prelude::*;

use crate::{
    sprite::{create_sprite_with_size, spawn_sprite},
    texture::TextureSheet,
    TILE_SIZE,
};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(create_tile_background);
    }
}

const WIDTH: f32 = 11.0;
const HEIGHT: f32 = 10.0;
const MIDDLE_X: u32 = (WIDTH / 2.0) as u32; // Rounds towards zero (floor)
const MIDDLE_Y: u32 = (HEIGHT / 2.0) as u32;
const TRANSLATE_X: f32 = TILE_SIZE * (MIDDLE_X as f32);
const TRANSLATE_Y: f32 = TILE_SIZE * (MIDDLE_Y as f32);

fn create_tile_background(mut commands: Commands, texture_sheet: Res<TextureSheet>) {
    // TODO: Make tile background dynamic based on screen size

    let mut tiles = Vec::new();

    for y in 0..(HEIGHT as u32) {
        for x in 0..(WIDTH as u32) {
            let tile = spawn_sprite(&mut commands, &texture_sheet, 0, tile_location(x, y));
            tiles.push(tile);
        }
    }

    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: create_sprite_with_size(0, 0.0), // Set size to zero to hide parent tile texture
            texture_atlas: texture_sheet.atlas_handle.clone(),
            transform: Transform {
                translation: tile_location(0, 0),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Map"))
        .insert(Transform::default())
        .insert(GlobalTransform::default())
        .push_children(&tiles);
}

fn tile_location(x: u32, y: u32) -> Vec3 {
    Vec3 {
        x: (x as f32) * TILE_SIZE - TRANSLATE_X,
        y: (y as f32) * TILE_SIZE - TRANSLATE_Y,
        z: 0.0,
    }
}

use bevy::prelude::*;

use crate::{
    sprite::{create_sprite, spawn_sprite},
    texture::TextureSheet,
    TILE_SIZE,
};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(create_tile_background);
    }
}

const WIDTH: f32 = 9.0;
const HEIGHT: f32 = 8.0;

fn create_tile_background(mut commands: Commands, texture_sheet: Res<TextureSheet>) {
    // TODO: Make tile background dynamic based on screen size

    let mut tiles = Vec::new();

    for y in 0..(HEIGHT as u32) {
        for x in 0..(WIDTH as u32) {
            let tile = spawn_sprite(&mut commands, &texture_sheet, 0, tile_location(x, y));
            tiles.push(tile);
        }
    }

    let mut parent_tile = TextureAtlasSprite::new(0);
    parent_tile.custom_size = Some(Vec2::splat(TILE_SIZE));

    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: create_sprite(0),
            texture_atlas: texture_sheet.atlas_handle.clone(),
            transform: Transform {
                translation: tile_location(0, 0), // TODO: Remove duplicate parent tile somehow
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
    let translation_x = TILE_SIZE * (WIDTH / 2.0).floor();
    let translation_y = TILE_SIZE * (HEIGHT / 2.0).floor();
    Vec3 {
        x: (x as f32) * TILE_SIZE - translation_x,
        y: (y as f32) * TILE_SIZE - translation_y,
        z: 0.0,
    }
}

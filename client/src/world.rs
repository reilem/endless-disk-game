use bevy::prelude::*;

use crate::{sprite::spawn_sprite, texture::TextureSheet, TILE_SIZE};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(create_tile_background);
    }
}

const WIDTH: f32 = 20.0;
const HEIGHT: f32 = 15.0;

fn create_tile_background(mut commands: Commands, texture_sheet: Res<TextureSheet>) {
    let translation_x = TILE_SIZE * (WIDTH / 2.0).floor();
    let translation_y = TILE_SIZE * (HEIGHT / 2.0).floor();
    for y in 0..(HEIGHT as u32) {
        for x in 0..(WIDTH as u32) {
            spawn_sprite(
                &mut commands,
                &texture_sheet,
                0,
                Vec3 {
                    x: (x as f32) * TILE_SIZE - translation_x,
                    y: (y as f32) * TILE_SIZE - translation_y,
                    z: 0.0,
                },
            );
        }
    }
}

use bevy::prelude::*;

use crate::{texture::TextureSheet, TILE_SIZE};

pub fn create_sprite(index: usize) -> TextureAtlasSprite {
    let mut sprite = TextureAtlasSprite::new(index);
    sprite.custom_size = Some(Vec2::splat(TILE_SIZE));
    sprite
}

pub fn spawn_sprite(
    commands: &mut Commands,
    texture_sheet: &TextureSheet,
    index: usize,
    translation: Vec3,
) -> Entity {
    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: create_sprite(index),
            texture_atlas: texture_sheet.atlas_handle.clone(),
            transform: Transform {
                translation,
                ..default()
            },
            ..default()
        })
        .id()
}

use bevy::prelude::*;

use crate::{texture::TextureSheet, TILE_SIZE};

pub fn spawn_sprite(
    commands: &mut Commands,
    texture_sheet: &TextureSheet,
    index: usize,
    translation: Vec3,
) -> Entity {
    let mut sprite = TextureAtlasSprite::new(index);
    sprite.custom_size = Some(Vec2::splat(TILE_SIZE));

    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite,
            texture_atlas: texture_sheet.atlas_handle.clone(),
            transform: Transform {
                translation,
                ..Default::default()
            },
            ..Default::default()
        })
        .id()
}

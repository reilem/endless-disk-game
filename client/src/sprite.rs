use bevy::{ecs::system::EntityCommands, prelude::*};

use crate::{texture::TextureSheet, TILE_SIZE};

pub fn create_sprite(index: usize) -> TextureAtlasSprite {
    create_sprite_with_size(SpriteInfo {
        index,
        size: TILE_SIZE,
    })
}

pub struct SpriteInfo {
    pub index: usize,
    pub size: f32,
}

pub fn create_sprite_with_size(info: SpriteInfo) -> TextureAtlasSprite {
    let mut sprite = TextureAtlasSprite::new(info.index);
    sprite.custom_size = Some(Vec2::splat(info.size));
    sprite
}

pub fn spawn_bundle<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    texture_sheet: &TextureSheet,
    sprite: TextureAtlasSprite,
    translation: Vec3,
) -> EntityCommands<'w, 's, 'a> {
    commands.spawn_bundle(SpriteSheetBundle {
        sprite,
        texture_atlas: texture_sheet.atlas_handle.clone(),
        transform: Transform {
            translation,
            ..default()
        },
        ..default()
    })
}

pub fn spawn_sprite(
    commands: &mut Commands,
    texture_sheet: &TextureSheet,
    index: usize,
    translation: Vec3,
) -> Entity {
    spawn_bundle(commands, texture_sheet, create_sprite(index), translation).id()
}

pub fn spawn_sprite_with_size(
    commands: &mut Commands,
    texture_sheet: &TextureSheet,
    index: usize,
    size: f32,
    translation: Vec3,
) -> Entity {
    spawn_bundle(
        commands,
        texture_sheet,
        create_sprite_with_size(SpriteInfo { index, size }),
        translation,
    )
    .id()
}

use bevy::prelude::{Handle, TextureAtlas};

pub mod debug;
pub mod player;

pub struct TextureSheet {
    pub atlas_handle: Handle<TextureAtlas>,
}

pub const TILE_SIZE: f32 = 96.0;
pub const TEXTURE_SIZE: f32 = 32.0;
pub const SCALE: f32 = TILE_SIZE / TEXTURE_SIZE;

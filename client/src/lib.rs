use bevy::prelude::{Handle, TextureAtlas};

pub mod player;

pub struct TextureSheet {
    pub atlas_handle: Handle<TextureAtlas>,
}

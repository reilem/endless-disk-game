use bevy::{
    prelude::*,
    render::{
        render_resource::{FilterMode, SamplerDescriptor},
        texture::ImageSampler,
    },
};

use crate::TEXTURE_SIZE;

pub struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, init_textures);
        app.add_system(set_texture_filters_to_nearest);
    }
}

pub struct TextureSheet {
    pub atlas_handle: Handle<TextureAtlas>,
}

fn init_textures(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlasses: ResMut<Assets<TextureAtlas>>,
) {
    let image: Handle<Image> = assets.load("atlas-1.png"); // NOTE: Pad the tiles if there is bleeding
    let atlas = TextureAtlas::from_grid(image, Vec2::splat(TEXTURE_SIZE), 3, 1);
    let atlas_handle = texture_atlasses.add(atlas);
    commands.insert_resource(TextureSheet { atlas_handle });
}

fn set_texture_filters_to_nearest(
    mut texture_events: EventReader<AssetEvent<Image>>,
    mut textures: ResMut<Assets<Image>>,
) {
    for event in texture_events.iter() {
        if let AssetEvent::Created { handle } = event {
            if let Some(mut texture) = textures.get_mut(handle) {
                texture.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
                    min_filter: FilterMode::Nearest,
                    mag_filter: FilterMode::Nearest,
                    mipmap_filter: FilterMode::Nearest,
                    ..default()
                })
            }
        }
    }
}

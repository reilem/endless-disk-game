use bevy::prelude::*;

use crate::{sprite::*, texture::TextureSheet, TILE_SIZE};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(create_tile_background);
    }
}

fn create_tile_background(
    mut commands: Commands,
    texture_sheet: Res<TextureSheet>,
    window: Res<WindowDescriptor>,
) {
    let window_size = Size {
        width: window.width as u32,
        height: window.height as u32,
    };

    let x_square_count = number_of_squares_horizontally(&window_size) as f32;
    let y_square_count = number_of_squares_vertically(&window_size) as f32;

    let mut tiles = Vec::new();

    for y in grid_range_start(y_square_count, 0.0)..grid_range_end(y_square_count, 0.0) {
        for x in grid_range_start(x_square_count, 0.0)..grid_range_end(x_square_count, 0.0) {
            let tile = spawn_sprite(
                &mut commands,
                &texture_sheet,
                0,
                tile_location(IVec2 { x, y }),
            );
            tiles.push(tile);
        }
    }

    spawn_bundle(
        &mut commands,
        &texture_sheet,
        create_sprite_with_size(SpriteInfo {
            index: 0,
            size: 0.0,
        }),
        tile_location(IVec2 { x: 0, y: 0 }),
    )
    .insert(Name::new("Map"))
    .insert(Transform::default())
    .insert(GlobalTransform::default())
    .push_children(&tiles);
}

/**
 * Calculates the tile location transformation required to put the tile in the correct position in the world.
 * Does this by simply multiplying the given tile index position by tile size.
 */
fn tile_location(index_position: IVec2) -> Vec3 {
    Vec3 {
        x: (index_position.x as f32) * TILE_SIZE,
        y: (index_position.y as f32) * TILE_SIZE,
        z: 0.0,
    }
}

/**
 * Always returns an uneven number, adds one if even to ensure full grid coverage.
 */
fn keep_uneven(value: u32) -> u32 {
    if value % 2 == 0 {
        value + 1
    } else {
        value
    }
}

/**
 * Returns the number of squares that will be needed to fill the window in squares
 * with size SQUARE_SIZE based on given parameter.
 */
fn number_of_squares(parameter: u32) -> u32 {
    let divided = (parameter as f32) / TILE_SIZE;
    let ceiled = divided.ceil();
    keep_uneven(ceiled as u32)
}

/**
 * Returns the number of squares that will be needed to fill the window horizontally
 */
fn number_of_squares_horizontally(size: &Size<u32>) -> u32 {
    number_of_squares(size.width)
}

/**
 * Returns the number of squares that will be needed to fill the window vertically
 */
fn number_of_squares_vertically(size: &Size<u32>) -> u32 {
    number_of_squares(size.height)
}

/**
 * Takes a square count of the world and an offset (player position) and calculates the lowest index in the range which will
 * be used to calculate the world grid.
 *
 * By dividing and ceiling the negation of the size by two we get the lowest index of the left-most (or bottom-most) square in the grid.
 * By adding the floor of the offset we ensure that negative numbers are rounded to their lowest value
 * and an extra square is always prepared in the grid before it comes into view.
 */
fn grid_range_start(square_count: f32, offset: f32) -> i32 {
    ((-1.0 * square_count) / 2.0).ceil() as i32 + offset.floor() as i32
}

/**
 * Take square count of the world and an offset (player position) and calculates the highest index in the range which will
 * be used during calculation of the world grid.
 *
 * We divide the square count by two and floor it to get the bottom-left coordinate of the right most square in the grid. This
 * could cause glitches when even numbers are passed to this calculation. But we fix this by ensuring the sizes
 * of the grid are always uneven numbers. At the end we add one because range calculations are not inclusive in rust.
 */
fn grid_range_end(square_count: f32, offset: f32) -> i32 {
    (square_count / 2.0).floor() as i32 + offset.ceil() as i32 + 1
}

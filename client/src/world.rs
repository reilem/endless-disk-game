use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;

use crate::{sprite::*, texture::TextureSheet, TILE_SIZE};

pub struct WorldPlugin;

#[derive(Component)]
pub struct TileCollider;

#[derive(Component)]
pub struct WorldTile;

#[derive(Component, Inspectable)]
pub struct TileMap {
    pub tiles_x: u32,
    pub tiles_y: u32,
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(create_tile_background);
        app.add_system(update_tile_background);
    }
}

fn create_tile_background(
    mut commands: Commands,
    texture_sheet: Res<TextureSheet>,
    window: Res<WindowDescriptor>,
) {
    // NOTE: Discussion on larger worlds and f32 vs f64 https://github.com/bevyengine/bevy/issues/1680
    //       We will likely need to have our own f64 coordinate system and for rendering simply use the screen bounds as coordinates
    let window_size = Size {
        width: window.width as u32,
        height: window.height as u32,
    };

    let x_square_count = number_of_squares_horizontally(&window_size);
    let y_square_count = number_of_squares_vertically(&window_size);

    let mut tiles = Vec::new();

    for y in grid_range_start(y_square_count)..grid_range_end(y_square_count) {
        for x in grid_range_start(x_square_count)..grid_range_end(x_square_count) {
            let tile = spawn_sprite(
                &mut commands,
                &texture_sheet,
                0,
                tile_location(IVec2 { x, y }),
            );
            commands.entity(tile).insert(WorldTile);
            tiles.push(tile);
        }
    }

    tiles.push(add_fire(
        &mut commands,
        &texture_sheet,
        IVec2 { x: 2, y: 4 },
    ));
    tiles.push(add_fire(
        &mut commands,
        &texture_sheet,
        IVec2 { x: -3, y: -2 },
    ));
    tiles.push(add_fire(
        &mut commands,
        &texture_sheet,
        IVec2 { x: 4, y: -1 },
    ));

    commands
        .spawn()
        .insert(Name::new("Map"))
        .insert(ComputedVisibility::default())
        .insert(Visibility::default())
        .insert(Transform::default())
        .insert(GlobalTransform::default())
        .insert(TileMap {
            tiles_x: x_square_count,
            tiles_y: y_square_count,
        })
        .push_children(&tiles);
}

fn update_tile_background(
    camera_query: Query<&Transform, With<Camera>>,
    map_query: Query<&TileMap>,
    mut tile_query: Query<&mut Transform, (With<WorldTile>, Without<Camera>)>,
) {
    // TODO: Only do this after player moves
    let tile_map = map_query.get_single().unwrap_or_else(|err| {
        panic!(
            "Failed to get tile map in update_tile_background: {:?}",
            err
        )
    });
    let tile_map_height = tile_map.tiles_y as f32 * TILE_SIZE; // Extract calculation
    let tile_map_width = tile_map.tiles_x as f32 * TILE_SIZE;

    // Lowest tolerated x value for a tile before it is reset to the right side of the tile map
    let threshold_left = tile_index_to_world_coord(grid_range_start(tile_map.tiles_x)) - TILE_SIZE;
    // Highest tolerated x value for a tile before it is reset to the left side of the tile map
    let threshold_right = tile_index_to_world_coord(grid_range_end(tile_map.tiles_x));
    // Highest tolerated y value for a tile before it is reset to the bottom of the tile map
    let threshold_top = tile_index_to_world_coord(grid_range_end(tile_map.tiles_y));
    // Lowest tolerated y value for a tile before it is reset to the top of the tile map
    let threshold_bottom =
        tile_index_to_world_coord(grid_range_start(tile_map.tiles_y)) - TILE_SIZE;

    let camera_transform = camera_query
        .get_single()
        .unwrap_or_else(|err| panic!("Failed to get camera in update_tile_background {:?}", err));

    for mut tile_transform in tile_query.iter_mut() {
        let normalised_tile_vec = tile_transform.translation - camera_transform.translation;
        if normalised_tile_vec.x < threshold_left {
            tile_transform.translation.x += tile_map_width;
        }
        if normalised_tile_vec.x > threshold_right {
            tile_transform.translation.x -= tile_map_width;
        }
        if normalised_tile_vec.y < threshold_bottom {
            tile_transform.translation.y += tile_map_height;
        }
        if normalised_tile_vec.y > threshold_top {
            tile_transform.translation.y -= tile_map_height;
        }
    }
}

fn add_fire(
    commands: &mut Commands,
    texture_sheet: &TextureSheet,
    index_position: IVec2,
) -> Entity {
    let fire = spawn_sprite(
        commands,
        texture_sheet,
        1,
        tile_location_3(IVec3 {
            x: index_position.x,
            y: index_position.y,
            z: 10,
        }),
    );
    commands.entity(fire).insert(TileCollider);
    fire
}

/**
 * Calculates the tile location transformation required to put the tile in the correct position in the world.
 * Does this by simply multiplying the given tile index position by tile size.
 */
fn tile_location(index_position: IVec2) -> Vec3 {
    tile_location_3(IVec3 {
        x: index_position.x,
        y: index_position.y,
        z: 0,
    })
}

/**
 * Calculates the tile location transformation required to put the tile in the correct position in the world.
 * Does this by simply multiplying the given tile index position by tile size.
 */
fn tile_location_3(index_position: IVec3) -> Vec3 {
    Vec3 {
        x: tile_index_to_world_coord(index_position.x),
        y: tile_index_to_world_coord(index_position.y),
        z: index_position.z as f32,
    }
}

fn tile_index_to_world_coord(index: i32) -> f32 {
    index as f32 * TILE_SIZE
}

/**
 * Always returns an uneven number, adds one if even to ensure full grid coverage.
 */
fn next_highest_uneven_number(value: u32) -> u32 {
    if value % 2 == 0 {
        value + 3
    } else {
        value + 2
    }
}

/**
 * Returns the number of squares that will be needed to fill the window in squares
 * with size SQUARE_SIZE based on given parameter.
 */
fn number_of_squares(parameter: u32) -> u32 {
    let divided = (parameter as f32) / TILE_SIZE;
    let ceiled = divided.ceil();
    next_highest_uneven_number(ceiled as u32)
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
 * Takes a square count of the world and calculates the lowest index in the range which will
 * be used to calculate the world grid.
 *
 * By dividing and ceiling the negation of the size by two we get the lowest index of the left-most (or bottom-most) square in the grid.
 */
fn grid_range_start(square_count: u32) -> i32 {
    ((-1.0 * square_count as f32) / 2.0).ceil() as i32
}

/**
 * Take square count of the world and calculates the highest index in the range which will
 * be used during calculation of the world grid.
 *
 * We divide the square count by two and floor it to get the bottom-left coordinate of the right most square in the grid. This
 * could cause glitches when even numbers are passed to this calculation. But we fix this by ensuring the sizes
 * of the grid are always uneven numbers. At the end we add one because range calculations are not inclusive in rust.
 */
fn grid_range_end(square_count: u32) -> i32 {
    (square_count as f32 / 2.0).floor() as i32 + 1
}

/* World Generation
 *
 * Goals of this file:
 *    To create unique fun interesting worlds that the player will want to explore and enjoy
 *    To keep functions clean and reusable if necessary under 30 locs per function if possible
 *
 */
use bracket_noise::prelude::*;
use rand::Rng;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use iyes_loopless::prelude::*;

use crate::{assets::SpriteAssets, constants::world_obj_sprites::*, interact::*, AppState};

pub const MAP_SIZE_X: u32 = 128; // Size of map currently only supports square maps
pub const MAP_SIZE_Y: u32 = 128; // Size of map currently only supports square maps
pub const TILE_PIXELS_X: f32 = 8f32;
pub const TILE_PIXELS_Y: f32 = 8f32;
pub const FLOOR_Z: f32 = 0f32; // Generally the lowest depth in terms of sprites
pub const OBJECT_Z: f32 = 10f32; // Height for objects such as trees or rocks to exist in the world

pub struct WorldGenerationPlugin;

impl Plugin for WorldGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(AppState::GameLoading, create_world.label("map"))
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(AppState::Running)
                    .with_system(regenerate_world)
                    .with_system(stretch_tree)
                    .into(),
            );
    }
}

// pub struct OverWorld {
//     floor_tiles: TileStorage,
//     floor_tilemap: Entity,
//     objs_tiles: TileStorage,
//     objes_tilemap: Entity
// }

fn create_world(mut commands: Commands, tiles: Res<SpriteAssets>) {
    let tilemap_size = world_size();

    // Floor tiles are the underlying tiles to everything in the overworld, should NEVER be empty,
    // limited to the terrain atlas
    let mut floor_tiles = TileStorage::empty(tilemap_size);
    let floor_tilemap = commands.spawn_empty().id();

    // Objs tiles are the items, world objects, special terrain features that appear on top of the floor
    // limited to the world_objs atlas
    let mut objs_tiles = TileStorage::empty(tilemap_size);
    let objs_tilemap = commands.spawn_empty().id();

    // TODO: allow user to input a seed, maybe using a config file?
    let seed = rand::random::<u64>();

    // Spawn the elements of the tilemaps.
    spawn_terrain(&mut commands, &mut floor_tiles, floor_tilemap, seed);
    spawn_trees(&mut commands, &mut objs_tiles, objs_tilemap, seed);

    commands.entity(floor_tilemap).insert(TilemapBundle {
        grid_size: tilegridsize_pixels(),
        map_type: TilemapType::default(),
        size: tilemap_size,
        storage: floor_tiles,
        texture: TilemapTexture::Single(tiles.terrain.clone()),
        tile_size: tilemaptilesize_pixels(),
        transform: Transform::from_translation(Vec3::new(0f32, 0f32, FLOOR_Z)),
        ..Default::default()
    });
    commands.entity(objs_tilemap).insert((
        TilemapBundle {
            grid_size: tilegridsize_pixels(),
            map_type: TilemapType::default(),
            size: tilemap_size,
            storage: objs_tiles,
            texture: TilemapTexture::Single(tiles.world_objs.clone()),
            tile_size: tilemaptilesize_pixels(),
            transform: Transform::from_translation(Vec3::new(0f32, 0f32, OBJECT_Z)),
            ..Default::default()
        },
        Blocking,
    ));
    println!("World Created succesfully");
}

fn regenerate_world(
    mut commands: Commands,
    mut tile_storage_q: Query<(&mut TileStorage, Entity)>,
    keeb: Res<Input<KeyCode>>,
    sprites: Res<SpriteAssets>,
) {
    if !keeb.just_pressed(KeyCode::Grave) {
        return;
    }

    for (mut tile_storage, tilemap_entity) in tile_storage_q.iter_mut() {
        // Despawn existing world
        for x in 0..MAP_SIZE_X {
            for y in 0..MAP_SIZE_Y {
                let tile_pos = TilePos { x, y };
                if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                    commands.entity(tile_entity).despawn_recursive();
                    tile_storage.remove(&tile_pos);
                }
            }
        }
        commands.entity(tilemap_entity).despawn_recursive();
    }

    // Create world
    create_world(commands, sprites);
}

/// Fills walkable_tiles with terrain and fills blocked_tiles with water
fn spawn_terrain(
    commands: &mut Commands,
    floor_tiles: &mut TileStorage,
    map_entity: Entity,
    seed: u64,
) {
    let noise = terrain_perlin(seed);
    let mut rng = rand::thread_rng();
    for x in 0..MAP_SIZE_X {
        for y in 0..MAP_SIZE_Y {
            let tile_pos = TilePos { x, y };

            let mut perlin_value = noise.get_noise((x as f32) / 160.0, (y as f32) / 100.0);
            perlin_value = (perlin_value + 1.0) * 0.5;

            let tile_entity = commands.spawn_empty().id();
            let texture_index = if perlin_value > 0.05f32 && perlin_value < 0.2f32 {
                // Water
                commands.entity(tile_entity).insert(Blocking);
                TileTextureIndex(13)
            } else {
                let foilage_percent = rng.gen::<u32>() % 100;
                let foilage_type = rng.gen_range(1..5);
                if foilage_percent >= 20 {
                    TileTextureIndex(0)
                } else {
                    TileTextureIndex(foilage_type)
                }
            };
            commands.entity(tile_entity).insert( TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(map_entity),
                    texture_index,
                    ..Default::default()
                });

            floor_tiles.set(&tile_pos, tile_entity);
        }
    }
}

fn spawn_trees(commands: &mut Commands, objs_tiles: &mut TileStorage, objs_tilemap: Entity, seed: u64) {
    let noise = tree_perlin(seed);

    for x in 0..MAP_SIZE_X {
        for y in (0..MAP_SIZE_Y).step_by(2) {
            let tree_base_pos = TilePos { x, y };
            let tree_top_pos = TilePos { x, y: y + 1 };

            if let Some(_) = objs_tiles.checked_get(&tree_top_pos) {
                continue;
            }
            if let Some(_) = objs_tiles.checked_get(&tree_base_pos) {
                continue;
            }

            let mut perlin_value = noise.get_noise((x as f32) / 160.0, (y as f32) / 100.0);
            perlin_value = (perlin_value + 1.0) * 0.5;

            if perlin_value < 0.2f32 || perlin_value > 0.6f32 {
                //spawn object
                let (base_entity, top_entity) = place_medium_tree(commands, &objs_tilemap, &tree_base_pos);
                objs_tiles.set(&tree_base_pos, base_entity);
                objs_tiles.set(&tree_top_pos, top_entity);
            }
        }
    }
}

fn place_medium_tree(commands: &mut Commands, blocked_tilemap: &Entity, tree_base_pos: &TilePos) -> (Entity, Entity) {
    let base_entity = commands
        .spawn((
            TileBundle {
                position: *tree_base_pos,
                tilemap_id: TilemapId(*blocked_tilemap),
                texture_index: TileTextureIndex(TREE_BASE),
                ..default()
            },
            Tree,
            Interact::Harvest(Health::new(5)),
            Blocking
        ))
        .id();
    let top_entity = commands
        .spawn((
            TileBundle {
                position: TilePos {
                    x: tree_base_pos.x,
                    y: tree_base_pos.y + 1,
                },
                tilemap_id: TilemapId(*blocked_tilemap),
                texture_index: TileTextureIndex(TREE_TOP),
                ..default()
            },
            Tree,
            Blocking
        ))
        .id();

    (base_entity, top_entity)
}

fn stretch_tree(mut tree_q: Query<(&mut Transform, &TilePos), With<Tree>>, keeb: Res<Input<KeyCode>>) {
    if keeb.pressed(KeyCode::K) {
        for (mut transform, _) in tree_q.iter_mut() {
            transform.scale.x += 0.06;
        }
    } else if keeb.pressed(KeyCode::J) {
        for (mut transform, _) in tree_q.iter_mut() {
            transform.scale.x -= 0.06;
        }
    }
}

// Perlin example
// let mut perlin_value = noise.get_noise((x as f32) / 160.0, (y as f32) / 100.0);
// perlin_value = (perlin_value + 1.0) * 0.5;
//
// let tile_index: u32 = match perlin_value {
//       x if x > 0.5 => 5,
//       x if x > 0.3 => 3,
//       x if x > 0.25 => 4,
//       x if x > 0.15 => 2,
//       x if x > 0.1 => 1,
//       _ => 0,
//     };

/// Returns the tile postion in the world with respect to tile size
#[allow(dead_code)]
fn tile_pos_to_world_pos(tile_pos: TilePos) -> Vec2 {
    tile_pos.center_in_world(&tilegridsize_pixels(), &TilemapType::Square)
        + Vec2::new(TILE_PIXELS_X / 2f32, TILE_PIXELS_Y / 2f32)
}

pub fn within_bounds(tile: Vec2) -> bool {
    tile.x >= 0.0 && tile.x < MAP_SIZE_X as f32 && tile.y >= 0.0 && tile.y < MAP_SIZE_Y as f32
}

pub fn world_size() -> TilemapSize {
    TilemapSize {
        x: MAP_SIZE_X,
        y: MAP_SIZE_Y,
    }
}

fn tilegridsize_pixels() -> TilemapGridSize {
    TilemapGridSize {
        x: TILE_PIXELS_X,
        y: TILE_PIXELS_Y,
    }
}

fn tilemaptilesize_pixels() -> TilemapTileSize {
    TilemapTileSize {
        x: TILE_PIXELS_X,
        y: TILE_PIXELS_Y,
    }
}

// Marks the tile_entity to denote it is unpassable
#[derive(Component)]
pub struct Blocking;

//=====> Terrain Components
#[derive(Component)]
struct Tree;

//=====> Perlin generators and settings

fn terrain_perlin(seed: u64) -> FastNoise {
    let mut noise = FastNoise::seeded(seed);
    noise.set_noise_type(NoiseType::SimplexFractal);
    noise.set_fractal_type(FractalType::FBM);
    noise.set_fractal_octaves(6);
    noise.set_fractal_gain(0.1);
    noise.set_fractal_lacunarity(2.0);
    noise.set_frequency(1.5);
    noise
}

fn tree_perlin(seed: u64) -> FastNoise {
    let mut noise = FastNoise::seeded(seed);
    noise.set_noise_type(NoiseType::SimplexFractal);
    noise.set_fractal_type(FractalType::FBM);
    noise.set_fractal_octaves(6);
    noise.set_fractal_gain(0.1);
    noise.set_fractal_lacunarity(2.0);
    noise.set_frequency(1.5);
    noise
}

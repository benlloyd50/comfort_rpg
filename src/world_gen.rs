use bracket_noise::prelude::*;
use rand::Rng;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use iyes_loopless::prelude::*;

use crate::{assets::SpriteAssets, AppState};

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
            .add_system(stretch_tree.run_in_state(AppState::Running));
    }
}

fn create_world(mut commands: Commands, tiles: Res<SpriteAssets>) {
    let tilemap_size = TilemapSize {
        x: MAP_SIZE_X,
        y: MAP_SIZE_Y,
    };

    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(tilemap_size);

    let seed = rand::random::<u64>();

    let mut blocked_tiles = Vec::new();

    // Spawn the elements of the tilemaps.
    spawn_terrain(&mut commands, &mut tile_storage, &mut blocked_tiles, tilemap_entity, seed);
    spawn_trees(&mut commands, &mut tile_storage, &mut blocked_tiles, &tiles, seed);

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size: tilegridsize_pixels(),
        map_type: TilemapType::default(),
        size: tilemap_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(tiles.terrain.clone()),
        tile_size: tilemaptilesize_pixels(),
        transform: Transform::from_translation(Vec3::new(0f32, 0f32, FLOOR_Z)),
        ..Default::default()
    });
    println!("World Created succesfully");
}

fn spawn_terrain(
    commands: &mut Commands,
    storage: &mut TileStorage,
    blocked_tiles: &mut Vec<TilePos>,
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

            let tile_index: u32;
            if perlin_value > 0.05f32 && perlin_value < 0.2f32 {
                tile_index = 13;
                blocked_tiles.insert(0, tile_pos);
                commands.entity(tile_entity).insert(Blocking);
            }
            else {
                let grass_chance = rng.gen::<u32>() % 100;
                let grass_type = rng.gen_range(1..5);
                tile_index = if grass_chance >= 20 { 0 } else { grass_type };
            }

            commands.entity(tile_entity).insert(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(map_entity),
                    texture_index: TileTextureIndex(tile_index),
                    ..Default::default()
                });
            storage.set(&tile_pos, tile_entity);
        }
    }
}

fn spawn_trees(
    commands: &mut Commands,
    tile_storage: &mut TileStorage,
    blocked_tiles: &mut Vec<TilePos>,
    tiles: &Res<SpriteAssets>,
    seed: u64,
) {
    let noise = tree_perlin(seed);

    for x in 0..MAP_SIZE_X {
        for y in (0..MAP_SIZE_Y).step_by(2) {
            let tree_base_pos = TilePos { x, y };
            let tree_top_pos = TilePos { x, y: y + 1 };
            let world_pos = tile_pos_to_world_pos(tree_base_pos) - Vec2::new(TILE_PIXELS_X / 2f32, 0f32);

            if blocked_tiles.contains(&tree_base_pos) || blocked_tiles.contains(&tree_top_pos) {
                continue;
            }
            else {
                blocked_tiles.insert(0, tree_base_pos);
                blocked_tiles.insert(0, tree_top_pos);
            }

            let mut perlin_value = noise.get_noise((x as f32) / 160.0, (y as f32) / 100.0);
            perlin_value = (perlin_value + 1.0) * 0.5;

            if perlin_value < 0.2f32 || perlin_value > 0.6f32 {
                //spawn object
                commands.spawn((
                    SpriteSheetBundle {
                        texture_atlas: tiles.tree.clone(),
                        sprite: TextureAtlasSprite { index: 0, ..default() },
                        transform: Transform::from_translation(Vec3 {
                            x: world_pos.x,
                            y: world_pos.y,
                            z: OBJECT_Z,
                        }),
                        ..default()
                    },
                    Tree,
                ));

                // update the tilemap entities with a marker component to block
                let tile1 = tile_storage.get(&tree_base_pos).unwrap();
                let tile2 = tile_storage.get(&tree_top_pos).unwrap();
                commands.entity(tile1).insert(Blocking);
                commands.entity(tile2).insert(Blocking);
            }
        }
    }
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
fn tile_pos_to_world_pos(tile_pos: TilePos) -> Vec2 {
    tile_pos.center_in_world(&tilegridsize_pixels(), &TilemapType::Square)
        + Vec2::new(TILE_PIXELS_X / 2f32, TILE_PIXELS_Y / 2f32)
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

// Marks a tile as blocking
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

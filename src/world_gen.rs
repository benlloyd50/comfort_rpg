/* World Generation
 *
 * Goals of this file:
 *    To create unique fun interesting worlds that the player will want to explore and enjoy
 *    To keep functions clean and reusable with the builder pattern
 *
 */
use bracket_noise::prelude::*;
use rand::Rng;
use std::time::Instant;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    assets::SpriteAssets, comfort_config::load_settings, constants::world_obj_sprites::*,
    interact::*, AppState,
};

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

pub struct GameWorld {
    // Floor tiles are the underlying tiles to everything in the overworld, should NEVER be empty,
    // limited to the terrain atlas
    floor_tiles: TileStorage,
    floor_tilemap: Entity,
    // Objs tiles are the items, world objects, special terrain features that appear on top of the floor
    // limited to the world_objs atlas
    objs_tiles: TileStorage,
    objs_tilemap: Entity,
    // TilePos that cannot have anything else placed ontop of them
    blocked_tiles: Vec<TilePos>,
    seed: u64,
}

fn create_world(mut commands: Commands, tiles: Res<SpriteAssets>) {
    let start = Instant::now();
    let tilemap_size = world_size();

    let mut overworld = GameWorld {
        floor_tiles: TileStorage::empty(tilemap_size),
        floor_tilemap: commands.spawn_empty().id(),
        objs_tiles: TileStorage::empty(tilemap_size),
        objs_tilemap: commands.spawn_empty().id(),
        blocked_tiles: Vec::new(),
        // TODO: allow user to input a seed, maybe using a config file?
        seed: rand::random::<u64>(),
    };

    // Spawn the elements of the tilemaps.
    overworld
        .spawn_terrain(&mut commands)
        .spawn_trees(&mut commands)
        .spawn_flowers(&mut commands);

    commands
        .entity(overworld.floor_tilemap)
        .insert(TilemapBundle {
            grid_size: tilegridsize_pixels(),
            map_type: TilemapType::default(),
            size: tilemap_size,
            storage: overworld.floor_tiles,
            texture: TilemapTexture::Single(tiles.terrain.clone()),
            tile_size: tilemaptilesize_pixels(),
            transform: Transform::from_translation(Vec3::new(0f32, 0f32, FLOOR_Z)),
            ..Default::default()
        });
    commands.entity(overworld.objs_tilemap).insert((
        TilemapBundle {
            grid_size: tilegridsize_pixels(),
            map_type: TilemapType::default(),
            size: tilemap_size,
            storage: overworld.objs_tiles,
            texture: TilemapTexture::Single(tiles.world_objs.clone()),
            tile_size: tilemaptilesize_pixels(),
            transform: Transform::from_translation(Vec3::new(0f32, 0f32, OBJECT_Z)),
            ..Default::default()
        },
        Blocking,
    ));
    let duration = start.elapsed();
    println!("World created succesfully in {:?}", duration);
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

impl GameWorld {
    /// Fills walkable_tiles with terrain and fills blocked_tiles with water
    fn spawn_terrain(&mut self, commands: &mut Commands) -> &mut GameWorld {
        let noise = terrain_perlin(self.seed);
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
                    self.blocked_tiles.push(tile_pos);
                    TileTextureIndex(13)
                } else {
                    let foilage_percent = rng.gen_range(0..100);
                    let foilage_type = rng.gen_range(1..5);
                    if foilage_percent >= 20 {
                        TileTextureIndex(0)
                    } else {
                        TileTextureIndex(foilage_type)
                    }
                };
                commands.entity(tile_entity).insert(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(self.floor_tilemap),
                    texture_index,
                    ..Default::default()
                });

                self.floor_tiles.set(&tile_pos, tile_entity);
            }
        }

        self
    }

    /// Spawns trees inside the world
    fn spawn_trees(&mut self, commands: &mut Commands) -> &mut GameWorld {
        let noise = tree_perlin(self.seed);

        for x in 0..MAP_SIZE_X {
            for y in (0..MAP_SIZE_Y).step_by(2) {
                let tree_base_pos = TilePos { x, y };
                let tree_top_pos = TilePos { x, y: y + 1 };

                if self.blocked_tiles.contains(&tree_base_pos)
                    || self.blocked_tiles.contains(&tree_top_pos)
                {
                    continue;
                }

                let mut perlin_value = noise.get_noise((x as f32) / 160.0, (y as f32) / 100.0);
                perlin_value = (perlin_value + 1.0) * 0.5;

                if perlin_value < 0.2f32 || perlin_value > 0.6f32 {
                    //spawn object
                    let (base_entity, top_entity) =
                        place_medium_tree(commands, &self.objs_tilemap, &tree_base_pos);
                    self.objs_tiles.set(&tree_base_pos, base_entity);
                    self.objs_tiles.set(&tree_top_pos, top_entity);
                }
            }
        }

        self
    }

    /// Spawns flowers on tiles that do not block
    fn spawn_flowers(&mut self, commands: &mut Commands) -> &mut GameWorld {
        let mut rng = rand::thread_rng();
        for x in 0..MAP_SIZE_X {
            for y in 0..MAP_SIZE_Y {
                let tile_pos = TilePos { x, y };
                if self.blocked_tiles.contains(&tile_pos) {
                    continue;
                }

                let foilage_percent = rng.gen_range(0..100);
                if foilage_percent <= 2 {
                    commands.spawn(TileBundle {
                        position: tile_pos,
                        texture_index: TileTextureIndex(rng.gen_range(2..8)),
                        tilemap_id: TilemapId(self.objs_tilemap),
                        ..default()
                    });
                }
            }
        }

        self
    }
}

fn place_medium_tree(
    commands: &mut Commands,
    blocked_tilemap: &Entity,
    tree_base_pos: &TilePos,
) -> (Entity, Entity) {
    let base_entity = commands.spawn_empty().id();
    let top_entity = commands.spawn_empty().id();
    let obj_size = ObjectSize::Multi(base_entity);

    commands.entity(base_entity).insert((
        TileBundle {
            position: *tree_base_pos,
            tilemap_id: TilemapId(*blocked_tilemap),
            texture_index: TileTextureIndex(TREE_BASE),
            ..default()
        },
        Tree,
        Interact::Harvest(Health::new(5)),
        Blocking,
        obj_size.clone(),
    ));
    commands.entity(top_entity).insert((
        TileBundle {
            position: TilePos {
                x: tree_base_pos.x,
                y: tree_base_pos.y + 1,
            },
            tilemap_id: TilemapId(*blocked_tilemap),
            texture_index: TileTextureIndex(TREE_TOP),
            ..default()
        },
        obj_size.clone(),
    ));

    (base_entity, top_entity)
}

fn stretch_tree(
    mut tree_q: Query<(&mut Transform, &TilePos), With<Tree>>,
    keeb: Res<Input<KeyCode>>,
) {
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

//====> World Data Components
// Marks a tile as being part of an object, the Entity will contain the data for the object
#[derive(Clone, Copy)]
#[derive(Component)]
pub enum ObjectSize {
    Single,
    Multi(Entity),
}

// Marks the tile_entity to denote it is unpassable
#[derive(Component)]
pub struct Blocking;

//=====> Terrain Components
#[derive(Component)]
struct Tree;

//=====> Perlin generators and settings
fn terrain_perlin(seed: u64) -> FastNoise {
    let config = match load_settings("terrainperlin") {
        Ok(config) => config,
        Err(_) => panic!("Could not load terrainperlin settings"),
    };
    let mut noise = FastNoise::seeded(seed);
    noise.set_noise_type(NoiseType::SimplexFractal);
    noise.set_fractal_type(FractalType::FBM);
    noise.set_fractal_octaves(config.octaves);
    noise.set_fractal_gain(config.gain);
    noise.set_fractal_lacunarity(config.lacunarity);
    noise.set_frequency(config.frequency);
    noise
}

fn tree_perlin(seed: u64) -> FastNoise {
    let config = match load_settings("treeperlin") {
        Ok(config) => config,
        Err(_) => panic!("Could not load treeperlin settings"),
    };
    let mut noise = FastNoise::seeded(seed);
    noise.set_noise_type(NoiseType::SimplexFractal);
    noise.set_fractal_type(FractalType::FBM);
    noise.set_fractal_octaves(config.octaves);
    noise.set_fractal_gain(config.gain);
    noise.set_fractal_lacunarity(config.lacunarity);
    noise.set_frequency(config.frequency);
    noise
}

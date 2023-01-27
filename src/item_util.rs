/// Item Utilities
///
/// Includes systems to spawn items
/// Contains the database for items
use bevy::{prelude::*, utils::HashMap};
use bevy_ecs_tilemap::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    assets::SpriteAssets,
    world_gen::{tilegridsize_pixels, tilemaptilesize_pixels, world_size, ItemStorage},
    AppState,
};
use serde::Deserialize;
use std::{error::Error, fs};

pub const ITEM_Z: f32 = 5f32; // Height for items

pub struct ItemUtilPlugin;

impl Plugin for ItemUtilPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnItemEvent>()
            .add_enter_system(AppState::GameLoading, init_item_database)
            .add_enter_system(AppState::GameLoading, create_item_tilestorage)
            .add_system(
                spawn_item_at_xy
                    .run_in_state(AppState::Running)
                    .run_on_event::<SpawnItemEvent>(),
            );
    }
}

// Maps all items to a unique u32, loaded on startup and should not be mutated at runtime
#[derive(Resource)]
pub struct ItemDatabase {
    items: HashMap<u32, Item>,
}

// Static information about the item that is the same across all of its kind
#[derive(Deserialize, Debug, Component, Clone)]
pub struct Item {
    pub id: u32,          // unique identifier for the item
    pub name: String,     // name of item
    pub atlas_index: u32, // sprite index for the atlas
}

#[derive(Component)]
pub struct ItemQuantity(u32);

fn init_item_database(mut commands: Commands) {
    let items = match load_items_from_json() {
        Ok(items) => items,
        Err(err) => panic!("Could not load items from json, {}", err),
    };

    let mut item_db = HashMap::new();
    for item in items {
        // println!("{:#?}", item);
        item_db.insert(item.id, item);
    }

    commands.insert_resource(ItemDatabase { items: item_db });
}

/// Attempts to load item definitions from a json file
fn load_items_from_json() -> Result<Vec<Item>, Box<dyn Error>> {
    let contents = fs::read_to_string("assets/items/comfort_items.json")?;
    let items: Vec<Item> = serde_json::from_str(&contents)?;
    Ok(items)
}

pub struct SpawnItemEvent {
    x: u32,
    y: u32,
    item_id: u32,
}

impl SpawnItemEvent {
    pub fn from(x: u32, y: u32, item_id: u32) -> SpawnItemEvent {
        SpawnItemEvent { x, y, item_id }
    }
}

fn spawn_item_at_xy(
    mut commands: Commands,
    mut tile_storage_q: Query<(Entity, &mut TileStorage), With<ItemStorage>>,
    item_db: Res<ItemDatabase>,
    mut ev_spawnitem: EventReader<SpawnItemEvent>,
) {
    for ev in ev_spawnitem.iter() {
        if let Ok((tiles_entity, mut item_tiles)) = tile_storage_q.get_single_mut() {
            let tile_pos = TilePos { x: ev.x, y: ev.y };
            if let Some(item) = item_db.items.get(&ev.item_id) {
                let item_entity = commands
                    .spawn((TileBundle {
                        position: tile_pos,
                        texture_index: TileTextureIndex(item.atlas_index),
                        tilemap_id: TilemapId(tiles_entity),
                        ..default()
                    }, Item::from(item.clone())
                    ))
                    .id();
                item_tiles.set(&tile_pos, item_entity);
            }
        }
    }
}

// Creates entity to hold all item entities when they exist in the overworld
fn create_item_tilestorage(mut commands: Commands, tiles: Res<SpriteAssets>) {
    let item_tilemap = commands.spawn_empty().id();
    let tilemap_size = world_size();
    let item_tiles = TileStorage::empty(tilemap_size);
    commands.entity(item_tilemap).insert((
        TilemapBundle {
            grid_size: tilegridsize_pixels(),
            map_type: TilemapType::Square,
            size: tilemap_size,
            storage: item_tiles,
            texture: TilemapTexture::Single(tiles.items.clone()),
            tile_size: tilemaptilesize_pixels(),
            transform: Transform::from_translation(Vec3::new(0f32, 0f32, ITEM_Z)),
            ..Default::default()
        },
        ItemStorage,
    ));
}

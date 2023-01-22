/// Item Utilities
///
/// Includes systems to spawn items
/// Contains the database for items
use bevy::{prelude::*, utils::HashMap};
use iyes_loopless::prelude::*;
use serde::Deserialize;
use std::{fs, error::Error};
use crate::AppState;

pub struct ItemUtilPlugin;

impl Plugin for ItemUtilPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(AppState::GameLoading, init_item_database);
            // .add_enter_system(AppState::GameLoading, create_item_tilestorage);
    }
}

#[derive(Resource)]
pub struct ItemDatabase {
    items: HashMap<u32, Item>,
}

// Static information about the item that is the same across all of its kind
#[derive(Deserialize, Debug)]
#[derive(Component)]
pub struct Item {
    id: u32,           // unique identifier for the item
    name: String,      // name of item
    atlas_index: u32,  // sprite index for the atlas
}

#[derive(Component)]
pub struct ItemQuantity(u32);

fn init_item_database(mut commands: Commands) {
    let items = match load_items_from_json() {
        Ok(items) => items,
        Err(_) => panic!("Could not load items from json"),
    };

    let mut item_db = HashMap::new();
    for item in items {
        println!("{:#?}", item);
        item_db.insert(item.id, item);
    }

    commands.insert_resource(ItemDatabase { items: item_db });
}

pub fn load_items_from_json() -> Result<Vec<Item>, Box<dyn Error>> {
    let contents = fs::read_to_string("assets/items/comfort_items.json")?;
    let items: Vec<Item> = serde_json::from_str(&contents)?;
    Ok(items)
}

fn create_item_tilestorage() {
    todo!("Create item tilestorage here")
}

use bevy::{prelude::*, utils::HashMap};
use bevy_ecs_tilemap::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    entity_tile_pos::EntityTilePos,
    item_util::{Item, ItemId, ItemQuantity},
    player::{Direction, Player, SystemOrder},
    world_gen::ItemStorage,
    AppState,
};

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        // app.add_enter_system(AppState::GameLoading, create_inventory_ui)
        app.add_system(
            take_item
                .run_in_state(AppState::Running)
                .label(SystemOrder::Input)
                .before(SystemOrder::Logic),
        );
    }
}

/// The max capacity of items able to be held
#[derive(Component)]
struct InventoryCapacity(u32);

// u32 is the id of the item
#[derive(Component)]
pub struct Inventory {
    items: HashMap<ItemId, ItemQuantity>,
}

impl Inventory {
    pub fn new() -> Self {
        Inventory { items: HashMap::new() }
    }

    pub fn add_item(&mut self, id: ItemId, amt: &ItemQuantity) {
        match self.items.entry(id) {
            bevy::utils::hashbrown::hash_map::Entry::Occupied(o) => {
                o.into_mut().0 += amt.0;
            }
            bevy::utils::hashbrown::hash_map::Entry::Vacant(v) => {
                v.insert(*amt);
            }
        }
    }
}

/// Marks an item as being carried by the owner Entity
#[derive(Component)]
struct Carried(Entity);

/// When the player presses the pickup key it will attempt to pickup the item under the player or
/// in the direction they face, priority is given to underneath self
fn take_item(
    mut commands: Commands,
    mut player_q: Query<(&EntityTilePos, &Direction, &mut Inventory), With<Player>>,
    mut tilestorage_q: Query<&mut TileStorage, With<ItemStorage>>,
    items_q: Query<(Entity, &Item, &ItemQuantity), With<TilePos>>,
    keeb: Res<Input<KeyCode>>,
) {
    // T for Take, this action may be held
    if !keeb.pressed(KeyCode::T) {
        return;
    }

    let mut tile_storage = match tilestorage_q.get_single_mut() {
        Ok(t) => t,
        Err(_) => panic!("found more than one itemstorage"),
    };
    let (position, direction, mut inventory) = match player_q.get_single_mut() {
        Ok(e) => e,
        Err(_) => panic!("found more than one player in harvest fn"),
    };

    let dest_tile = TilePos::new(position.x, position.y);

    if let Some(tile_entity) = tile_storage.get(&dest_tile) {
        if let Ok((entity, item_info, qty)) = items_q.get(tile_entity) {
            tile_storage.remove(&dest_tile);
            commands.entity(entity).despawn_recursive();
            inventory.add_item(item_info.id, qty);
            println!("Picked up {}", item_info.name);
        }
    }

    
    let dest_tile = match *direction {
        Direction::Up => { TilePos::new(position.x, position.y + 1) },
        Direction::Down => { TilePos::new(position.x, position.y - 1) },
        Direction::Left => { TilePos::new(position.x - 1, position.y) },
        Direction::Right => { TilePos::new(position.x + 1,position.y) },
    };
    
    if let Some(tile_entity) = tile_storage.get(&dest_tile) {
        if let Ok((entity, item_info, qty)) = items_q.get(tile_entity) {
            tile_storage.remove(&dest_tile);
            commands.entity(entity).despawn_recursive();
            inventory.add_item(item_info.id, qty);
            println!("Picked up {}", item_info.name);
        }
    }
}

#[allow(dead_code)]
fn create_inventory_ui() {
    todo!()
}

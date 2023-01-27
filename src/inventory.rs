use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use iyes_loopless::prelude::*;

use crate::{AppState, player::{SystemOrder, Player, Direction}, entity_tile_pos::EntityTilePos, item_util::Item, world_gen::ItemStorage};

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        // app.add_enter_system(AppState::GameLoading, create_inventory_ui)
        app.add_system(
            take_item
                .run_in_state(AppState::Running)
                .label(SystemOrder::Input)
                .before(SystemOrder::Logic)
        );
    }
}

/// The max capacity of items able to be held
#[derive(Component)]
struct InventoryCapacity(u32);

/// Marks an item as being carried by the owner Entity
#[derive(Component)]
struct Carried(Entity);


/// When the player presses the pickup key it will attempt to pickup the item under the player or
/// in the direction they face, priority is given to underneath self
fn take_item(
    mut commands: Commands,
    player_q: Query<(Entity, &EntityTilePos, &Direction), With<Player>>,
    items_q: Query<(Entity, &Item), With<TilePos>>,
    mut tilestorage_q: Query<&mut TileStorage, With<ItemStorage>>,
    keeb: Res<Input<KeyCode>>, 
) {
    // T for Take
    if !keeb.just_pressed(KeyCode::T) {
        return;
    }
    
    let mut tile_storage = match tilestorage_q.get_single_mut() {
        Ok(t) => t,
        Err(_) => panic!("found more than one itemstorage")
    };
    let (player, player_pos, dir) = match player_q.get_single() {
        Ok(e) => e,
        Err(_) => panic!("found more than one player in harvest fn")
    };

    let dest_tile = TilePos::new(player_pos.x, player_pos.y);

    if let Some(tile_entity) = tile_storage.get(&dest_tile) {
        if let Ok((entity, item_info)) = items_q.get(tile_entity) {
            // Remove tile from storage
            tile_storage.remove(&dest_tile);
            // Make the item carried by the player
            // Remove the components that make it exist in the world since it is now in your inventory
            commands.entity(entity).insert(Carried(player)).remove::<Transform>().remove::<TilePos>();
            println!("Picked up {}", item_info.name);
            return;
        }
    }

    let dest_tile = match *dir {
        Direction::Up => {TilePos{x: player_pos.x, y: player_pos.y + 1}},
        Direction::Down => {TilePos{x: player_pos.x, y: player_pos.y - 1}},
        Direction::Left => {TilePos{x: player_pos.x - 1, y: player_pos.y }},
        Direction::Right => {TilePos{x: player_pos.x + 1, y: player_pos.y }},
    };

    if let Some(tile_entity) = tile_storage.get(&dest_tile) {
        if let Ok((entity, item_info)) = items_q.get(tile_entity) {
            // Remove tile from storage
            tile_storage.remove(&dest_tile);
            // Make the item carried by the player
            commands.entity(entity).insert(Carried(player)).remove::<Transform>().remove::<TilePos>();
            println!("Picked up {}", item_info.name);
        }
    }

}

#[allow(dead_code)]
fn create_inventory_ui() {
    todo!()
}

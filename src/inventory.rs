use bevy::{prelude::*, ui::widget::ImageMode};
use bevy_ecs_tilemap::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    assets::{FontAssets, UiAssets},
    entity_tile_pos::EntityTilePos,
    item_util::{Item, ItemDatabase, ItemId, ItemQuantity},
    player::{Direction, Player, SystemOrder},
    world_gen::ItemStorage,
    GameState,
};

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::GameLoading, create_inventory_ui)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Running)
                    .label(SystemOrder::Input)
                    .before(SystemOrder::Logic)
                    .with_system(take_item)
                    .with_system(toggle_inventory)
                    .into(),
            )
            .add_event::<InventoryUpdate>()
            .add_system(
                ui_inventory_update
                    .run_in_state(GameState::Running)
                    .run_on_event::<InventoryUpdate>(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Menu)
                    .with_system(toggle_inventory)
                    .with_system(move_inventory_cursor)
                    .into(),
            );
    }
}

/// The max capacity of items able to be held
#[derive(Component)]
struct InventoryCapacity(u32);

// u32 is the id of the item
#[derive(Component)]
pub struct Inventory {
    items: Vec<InventoryItem>,
    // items: HashMap<ItemId, ItemQuantity>,
    max_size: usize,
}

struct InventoryItem {
    id: u32,
    amt: u32,
}

impl Inventory {
    pub fn new() -> Self {
        Inventory {
            // items: HashMap::new(),
            items: Vec::new(),
            max_size: 15,
        }
    }

    pub fn add_item(&mut self, id: ItemId, amt: &ItemQuantity) {
        match self.items.iter().position(|i| i.id == id.0) {
            Some(idx) => { self.items[idx].amt += amt.0;},
            None => if self.items.len() < self.max_size {
                self.items.push(InventoryItem { id: id.0, amt: amt.0});
            }
        }
    }

    // Attempts to remove items from an inventory
    // Will fail if the quantity in the inventory is less than what is trying to be removed
    pub fn remove_item(&mut self, id: ItemId, amt: &ItemQuantity) -> bool {
        match self.items.iter().position(|i| i.id == id.0) {
            Some(idx) => { 
                if self.items[idx].amt >= amt.0 {
                    self.items[idx].amt -= amt.0;
                    if self.items[idx].amt == 0 {
                        self.items.remove(idx);
                    }
                    return true;
                }   
                return false; 
            },
            None => false
        }


        // match self.items.entry(id) {
        //     Entry::Occupied(mut o) => {
        //         if o.get().0 >= amt.0 {
        //             o.get_mut().0 -= amt.0;
        //             if o.get().0 == 0 {
        //                 o.remove_entry();
        //             }
        //             return true;
        //         }
        //         false
        //     }
        //     Entry::Vacant(_v) => false,
        // }
    }

    // Checks the inventory to see if there is the specified quantity and item inside
    pub fn contains_item(&self, id: ItemId, amt: &ItemQuantity) -> bool {
        match self.items.iter().position(|i| i.id == id.0) {
            Some(idx) => self.items[idx].amt >= amt.0,
            None => false,
        }
    }
}

/// When the player presses the pickup key it will attempt to pickup the item under the player or
/// in the direction they face, priority is given to underneath self
fn take_item(
    mut commands: Commands,
    mut player_q: Query<(&EntityTilePos, &Direction, &mut Inventory), With<Player>>,
    mut tilestorage_q: Query<&mut TileStorage, With<ItemStorage>>,
    mut ev_invopen: EventWriter<InventoryUpdate>,
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
        }
    }

    let dest_tile = match *direction {
        Direction::Up => TilePos::new(position.x, position.y + 1),
        Direction::Down => TilePos::new(position.x, position.y - 1),
        Direction::Left => TilePos::new(position.x - 1, position.y),
        Direction::Right => TilePos::new(position.x + 1, position.y),
    };

    if let Some(tile_entity) = tile_storage.get(&dest_tile) {
        if let Ok((entity, item_info, qty)) = items_q.get(tile_entity) {
            tile_storage.remove(&dest_tile);
            commands.entity(entity).despawn_recursive();
            inventory.add_item(item_info.id, qty);
            ev_invopen.send(InventoryUpdate);
        }
    }
}

#[derive(Component)]
struct InventoryUi;

#[derive(Component)]
struct InventorySlot(u32);

#[derive(Component)]
struct InventoryPointer(usize);

fn create_inventory_ui(mut commands: Commands, font: Res<FontAssets>, elements: Res<UiAssets>) {
    let text_style = TextStyle {
        font: font.chunk.clone(),
        font_size: 30.0,
        color: Color::BLACK,
    };

    let inv_bg_style = Style {
        align_self: AlignSelf::Center,
        position_type: PositionType::Absolute,
        size: Size::new(Val::Px(600.), Val::Px(600.)),
        ..default()
    };

    let placeholder = format!("{: <40}AMT:{:>3}", "Wood", 999); 

    commands
        .spawn((
            NodeBundle {
                transform: Transform::from_xyz(0., 0., 80.),
                visibility: Visibility { is_visible: true},
                ..default()
            },
            InventoryUi,
        ))
        .with_children(|parent| {
            // the window which objects for the inventory ui will sit on
            parent
                .spawn(ImageBundle {
                    image: UiImage(elements.menubg.clone()),
                    style: inv_bg_style.clone(),
                    image_mode: ImageMode::KeepAspect,
                    ..default()
                })
                .with_children(|parent| {
                    // Empty slots for items
                    for i in 0..18 {
                        let offset: f32 = i as f32 * 25.0;
                        parent.spawn((
                            TextBundle::from_section(placeholder.clone(), text_style.clone()).with_style(Style {
                                align_self: AlignSelf::Center,
                                position_type: PositionType::Absolute,
                                position: UiRect {
                                    left: Val::Px(50.),
                                    top: Val::Px(49. + offset),
                                    ..default()
                                },
                                ..default()
                            }),
                            InventorySlot(i),
                        ));
                    }
                });
                // .with_children(|parent| {
                //     parent.spawn(ImageBundle {
                //         image: UiImage(elements.)
                //     }),
                //     InventoryPointer(0)
                // });
        });
}

pub struct InventoryUpdate;

fn ui_inventory_update(
    mut ev_invopen: EventReader<InventoryUpdate>,
    mut ui_slots_q: Query<&mut Text, With<InventorySlot>>,
    inv_q: Query<&Inventory, With<Player>>,
    item_db: Res<ItemDatabase>,
) {
    for _ in ev_invopen.iter() {
        let player_inv = match inv_q.get_single() {
            Ok(e) => e,
            Err(_) => panic!("Could not fetch the player's inventory!!!"),
        };

        let mut filled_in: usize = 0;
        for (mut text, InventoryItem{id, amt}) in ui_slots_q.iter_mut().zip(player_inv.items.iter()) {
            if let Some(info) = item_db.items.get(&ItemId(*id)) {
                text.sections[0].value = format!("{: <40}AMT:{:>3}", info.name, amt);
            } else {
                text.sections[0].value = format!("{: <20}AMT:{:>3}", "undefined", amt);
            }

            filled_in += 1;
        }
        for mut text in ui_slots_q.iter_mut().skip(filled_in) {
            text.sections[0].value = String::new();
        }
    }
}

// Toggles the game to inventory mode, game is shifted into Menu state so the game world pauses
// Will go between menu mode and running state
fn toggle_inventory(
    mut commands: Commands,
    mut inventory_ui_q: Query<&mut Visibility, With<InventoryUi>>,
    keeb: Res<Input<KeyCode>>,
    mut ev_invopen: EventWriter<InventoryUpdate>,
) {
    if !keeb.just_pressed(KeyCode::I) {
        return;
    }

    if let Ok(mut inventory_ui) = inventory_ui_q.get_single_mut() {
        inventory_ui.is_visible = !inventory_ui.is_visible;
        if inventory_ui.is_visible {
            commands.insert_resource(NextState(GameState::Menu));
            ev_invopen.send(InventoryUpdate);
        } else {
            commands.insert_resource(NextState(GameState::Running));
        }
    }
}

// Moves what position in the inventory ui the player is currently pointing at
// NOTE: need a way to get the item it is pointing at
fn move_inventory_cursor(mut inv_pointer_q: Query<&mut InventoryPointer>, player_q: Query<&Inventory, With<Player>>, keeb: Res<Input<KeyCode>>) {
    let player_inv = match player_q.get_single() {
        Ok(e) => e,
        Err(_) => panic!("Did not find a player inventory")
    };

    if let Ok(mut inv_pointer) = inv_pointer_q.get_single_mut() {
        if keeb.just_pressed(KeyCode::S) {
            if inv_pointer.0 < player_inv.max_size {
                inv_pointer.0 += 1;
            }
        } else if keeb.just_pressed(KeyCode::W) {
            if inv_pointer.0 > 0 {
                inv_pointer.0 -= 1;
            }
        }
    }
}

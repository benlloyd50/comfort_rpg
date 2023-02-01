use bevy::{prelude::*, ui::widget::ImageMode, utils::HashMap};
use bevy_ecs_tilemap::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    assets::{FontAssets, UiAssets},
    entity_tile_pos::EntityTilePos,
    item_util::{Item, ItemId, ItemQuantity, ItemDatabase},
    player::{Direction, Player, SystemOrder},
    world_gen::ItemStorage,
    AppState,
};

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(AppState::GameLoading, create_inventory_ui)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(AppState::Running)
                    .label(SystemOrder::Input)
                    .before(SystemOrder::Logic)
                    .with_system(take_item)
                    .with_system(toggle_inventory)
                    .into()
            )
            .add_event::<InventoryUpdate>()
            .add_system( ui_inventory_update.run_in_state(AppState::Running).run_on_event::<InventoryUpdate>());
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
            println!("Picked up {}", item_info.name);
        }
    }
}

#[derive(Component)]
struct InventoryUi;

#[derive(Component)]
struct InventorySlot(u32);

#[allow(dead_code)]
fn create_inventory_ui(mut commands: Commands, font: Res<FontAssets>, elements: Res<UiAssets>) {
    let text_style = TextStyle {
        font: font.chunk.clone(),
        font_size: 32.0,
        color: Color::BLACK,
    };

    let inv_bg_style = Style {
        align_self: AlignSelf::Center,
        position_type: PositionType::Absolute,
        size: Size::new(Val::Px(600.), Val::Px(600.)),
        ..default()
    };

    commands
        .spawn((NodeBundle {
            transform: Transform::from_xyz(0., 0., 80.),
            ..default()
        }, InventoryUi))
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
                    for i in 0..15 {
                        let offset: f32 = i as f32 * 26.0;
                        parent.spawn((
                            TextBundle::from_section(format!("{: <20} {:>3}", "======", "000"), text_style.clone())
                            .with_style(Style {
                                align_self: AlignSelf::Center,
                                position_type: PositionType::Absolute,
                                position: UiRect {
                                    left: Val::Px(40.),
                                    top: Val::Px(32. + offset),
                                    ..default()
                                },
                                ..default()
                            }),
                            InventorySlot(i),
                        ));
                    }
                });
        });
}

pub struct InventoryUpdate;

fn ui_inventory_update(
    mut ev_invopen: EventReader<InventoryUpdate>,
    mut ui_slots_q: Query<&mut Text, With<InventorySlot>>,
    inv_q: Query<&mut Inventory, With<Player>>,
    item_db: Res<ItemDatabase>,
) {
    for _ in ev_invopen.iter() {
        let player_inv = match inv_q.get_single() {
            Ok(e) => e,
            Err(_) => panic!("Could not fetch the player's inventory!!!"),
        };

        let mut filled_in: usize = 0;
        for (mut text, (item_id, qty)) in ui_slots_q.iter_mut().zip(player_inv.items.iter()) {
            if let Some(info) = item_db.items.get(&item_id) {
                text.sections[0].value = format!("{: <20}AMT:{:>3}", info.name, qty.0);
            } else {
                text.sections[0].value = format!("{: <20}AMT:{:>3}", "undefined", "XXX");
            }
            filled_in += 1;
        }
        for mut text in ui_slots_q.iter_mut().skip(filled_in) {
            text.sections[0].value = String::new();
        }
    }
}

fn toggle_inventory(mut inventory_ui_q: Query<&mut Visibility, With<InventoryUi>>, keeb: Res<Input<KeyCode>>
                    , mut ev_invopen: EventWriter<InventoryUpdate>) {
    if !keeb.just_pressed(KeyCode::I) {
        return;
    }

    if let Ok(mut inventory_ui) = inventory_ui_q.get_single_mut() {
        inventory_ui.is_visible = !inventory_ui.is_visible;
        if inventory_ui.is_visible {
            ev_invopen.send(InventoryUpdate);
        }
    }

}

use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    assets::SpriteAssets,
    effects::lerp,
    entity_tile_pos::EntityTilePos,
    interact::{HarvestInteraction, Interact},
    inventory::Inventory,
    world_gen::{within_bounds, Blocking, ObjectSize},
    AppState,
};

pub const PLAYER_Z: f32 = 50.0;
const PLAYER_TILE_SPEED: u32 = 1;
const PLAYER_MOVE_TIMER_MS: u64 = 175;

pub struct PlayerPlugin;

#[derive(Eq, PartialEq, SystemLabel)]
pub enum SystemOrder {
    Input,
    Logic,
    Graphic,
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        // Input => Logic => Graphic => cleanup
        app.add_enter_system(AppState::GameLoading, setup_character.after("map"))
            .add_event::<MoveEvent>()
            .add_system(
                move_player
                    .run_in_state(AppState::Running)
                    .run_if(movement_cooldown)
                    .label(SystemOrder::Logic)
                    .after(SystemOrder::Input),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(AppState::Running)
                    .label(SystemOrder::Input)
                    .before(SystemOrder::Logic)
                    .with_system(directional_input_handle)
                    .with_system(player_harvest_action)
                    .into(),
            )
            .add_system(
                update_sprite_position::<Player>
                    .run_in_state(AppState::Running)
                    .label(SystemOrder::Graphic)
                    .after(SystemOrder::Logic),
            )
            .add_system(
                move_target
                    .run_in_state(AppState::Running)
                    .label(SystemOrder::Graphic)
                    .after(SystemOrder::Logic),
            );
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
struct PlayerTarget;

#[derive(Component)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Component)]
struct HeldTimer(Timer);

fn setup_character(mut commands: Commands, sprites: Res<SpriteAssets>, _blocking_q: Query<&TilePos, With<Blocking>>) {
    // TODO: Find first nonblocking tilepos
    let starting_pos = EntityTilePos { x: 64, y: 64 };

    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: sprites.characters.clone(),
            transform: Transform::from_xyz(starting_pos.x as f32 * 8.0, starting_pos.y as f32 * 8.0, 50.0),
            ..default()
        },
        Player,
        Direction::Down,
        starting_pos,
        HeldTimer(Timer::new(Duration::from_millis(PLAYER_MOVE_TIMER_MS), TimerMode::Repeating)),
        Inventory::new(),
    ));

    println!("Created player succesfully");

    commands.spawn((
        SpriteBundle {
            texture: sprites.target.clone(),
            transform: Transform::from_xyz(starting_pos.x as f32 * 8.0, starting_pos.y as f32 * 8.0 - 8.0, 50.0),
            ..default()
        },
        PlayerTarget,
    ));
}

/// Moves the player target sprite in front of the player based on the last direction pressed,
/// purely visually as actions calculate their own positions
fn move_target(player_q: Query<(&EntityTilePos, &Direction)>, mut target_q: Query<&mut Transform, With<PlayerTarget>>) {
    if let Ok((player_tile_pos, dir)) = player_q.get_single() {
        if let Ok(mut target) = target_q.get_single_mut() {
            target.translation = match *dir {
                Direction::Up => Vec3::new(player_tile_pos.x as f32 * 8.0, player_tile_pos.y as f32 * 8.0 + 8.0, 50.0),
                Direction::Down => {
                    Vec3::new(player_tile_pos.x as f32 * 8.0, player_tile_pos.y as f32 * 8.0 - 8.0, 50.0)
                }
                Direction::Left => {
                    Vec3::new(player_tile_pos.x as f32 * 8.0 - 8.0, player_tile_pos.y as f32 * 8.0, 50.0)
                }
                Direction::Right => {
                    Vec3::new(player_tile_pos.x as f32 * 8.0 + 8.0, player_tile_pos.y as f32 * 8.0, 50.0)
                }
            };
        }
    };
}

struct MoveEvent(Entity, TilePos);

/// Moves player entity from input
fn move_player(mut player_q: Query<&mut EntityTilePos>, mut ev_move: EventReader<MoveEvent>) {
    for ev in ev_move.iter() {
        if let Ok(mut player_tile_pos) = player_q.get_mut(ev.0) {
            player_tile_pos.x = ev.1.x;
            player_tile_pos.y = ev.1.y;
        };
    }
}

// Returns true if the timer has finished in the frame
fn movement_cooldown(mut timer_q: Query<&mut HeldTimer, With<Player>>, time: Res<Time>) -> bool {
    let mut move_time = timer_q.single_mut();
    move_time.0.tick(time.delta());
    move_time.0.finished()
}

/// Updates the sprite position based on a discrete position in the entity
fn update_sprite_position<Type: Component>(mut entity_q: Query<(&mut Transform, &EntityTilePos), With<Type>>) {
    let (mut sprite_pos, entity_actual_pos) = entity_q.single_mut();
    let destination_pos = entity_actual_pos.center_in_world();

    let lerped_pos = lerp(sprite_pos.translation.truncate(), destination_pos, 0.15);
    sprite_pos.translation = Vec3::new(lerped_pos.x, lerped_pos.y, PLAYER_Z);
}

/// Checks to ensure dest tile is inbounds of the map
fn directional_input_handle(
    mut player_q: Query<(Entity, &EntityTilePos, &mut Direction), With<Player>>,
    obj_tiles_q: Query<(&TilePos, Option<&ObjectSize>, Option<&Blocking>)>,
    mut ev_moveplayer: EventWriter<MoveEvent>,
    keeb: Res<Input<KeyCode>>,
) {
    // find the dest_tile which is player_pos + direction pressed
    let (player_entity, player_tile_pos, mut direction) = player_q.single_mut();

    let mut dest_tile = Vec2::new(player_tile_pos.x as f32, player_tile_pos.y as f32);
    // else if here prevents dest_tile equalling zero delta
    if keeb.pressed(KeyCode::W) {
        dest_tile.y += PLAYER_TILE_SPEED as f32;
        *direction = Direction::Up;
    } else if keeb.pressed(KeyCode::S) {
        dest_tile.y -= PLAYER_TILE_SPEED as f32;
        *direction = Direction::Down;
    }
    if keeb.pressed(KeyCode::D) {
        dest_tile.x += PLAYER_TILE_SPEED as f32;
        *direction = Direction::Right;
    } else if keeb.pressed(KeyCode::A) {
        dest_tile.x -= PLAYER_TILE_SPEED as f32;
        *direction = Direction::Left;
    }

    // if the dest_tile or input was 0 then we don't do anything else
    // make sure the position is not out of bounds of the map
    if dest_tile == Vec2::new(player_tile_pos.x as f32, player_tile_pos.y as f32)
        || !within_bounds(Vec2::new(dest_tile.x as f32, dest_tile.y as f32))
    {
        return;
    }

    let dest_tile = TilePos {
        x: dest_tile.x as u32,
        y: dest_tile.y as u32,
    };

    // if the objects
    for (_, size, blocking) in obj_tiles_q.iter().filter(|x| dest_tile.eq(x.0)) {
        if size.is_some() || blocking.is_some() {
            return;
        }
    }
    ev_moveplayer.send(MoveEvent(player_entity, dest_tile));
}

fn player_harvest_action(
    player_q: Query<(Entity, &EntityTilePos, &Direction), With<Player>>,
    blocking_interact_q: Query<(Entity, &TilePos), (With<Interact>, With<Blocking>)>,
    obj_tiles_q: Query<(Entity, &ObjectSize, &TilePos)>,
    mut ev_interact: EventWriter<HarvestInteraction>,
    keeb: Res<Input<KeyCode>>,
) {
    if !keeb.just_pressed(KeyCode::Space) {
        return;
    }

    let (player_entity, pos, dir) = match player_q.get_single() {
        Ok(e) => e,
        Err(_) => {
            panic!("found more than one player in harvest fn")
        }
    };

    let dest_tile = match *dir {
        Direction::Up => TilePos { x: pos.x, y: pos.y + 1 },
        Direction::Down => TilePos { x: pos.x, y: pos.y - 1 },
        Direction::Left => TilePos { x: pos.x - 1, y: pos.y },
        Direction::Right => TilePos { x: pos.x + 1, y: pos.y },
    };

    // check if the tile is an interactable
    //   is the dest_tile part of a multi tile -> get owner entity
    //   is the owner entity in the interactable query -> get entity with components
    //   give entity to the interact system
    if let Some((dest_entity, size, _)) = obj_tiles_q.iter().find(|x| dest_tile.eq(x.2)) {
        println!("hit something");
        match *size {
            ObjectSize::Single => {
                ev_interact.send(HarvestInteraction {
                    harvester: player_entity,
                    harvested: dest_entity,
                    reciever_pos: dest_tile,
                });
            }
            ObjectSize::Multi(owner) => {
                if blocking_interact_q.get(owner).is_ok() {
                    ev_interact.send(HarvestInteraction {
                        harvester: player_entity,
                        harvested: owner,
                        reciever_pos: dest_tile,
                    });
                };
            }
        }
    }
}

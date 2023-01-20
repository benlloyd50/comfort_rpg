use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    assets::SpriteAssets,
    effects::lerp,
    entity_tile_pos::EntityTilePos,
    interact::{HealthBelowZeroEvent, Interact},
    world_gen::{within_bounds, Blocking, ObjectSize},
    AppState,
};

pub const PLAYER_Z: f32 = 50.0;
const PLAYER_TILE_SPEED: u32 = 1;
const PLAYER_INTERACT_TIMER_MS: u64 = 600;
const PLAYER_MOVE_TIMER_MS: u64 = 100;

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
            .add_event::<Interaction>()
            .add_system(
                move_player
                    .run_in_state(AppState::Running)
                    .run_if(movement_cooldown)
                    .label(SystemOrder::Logic)
                    .after(SystemOrder::Input),
            )
            .add_system(
                directional_input_handle
                    .run_in_state(AppState::Running)
                    .label(SystemOrder::Input)
                    .before(SystemOrder::Logic),
            )
            .add_system(
                player_interact_handler
                    .run_in_state(AppState::Running)
                    .run_if(interact_cooldown)
                    .label(SystemOrder::Logic)
                    .after(SystemOrder::Input),
            )
            .add_system(
                update_sprite_position::<Player>
                    .run_in_state(AppState::Running)
                    .label(SystemOrder::Graphic)
                    .after(SystemOrder::Logic),
            );
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct MoveEvent(Entity, TilePos);

struct Interaction {
    sender: Entity,
    reciever: Entity,
}

/// Timer used as an sleeper for held actions
#[derive(Component)]
struct InteractTimer(Timer);

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
        InteractTimer(Timer::new(Duration::from_millis(PLAYER_INTERACT_TIMER_MS), TimerMode::Repeating)),
        HeldTimer(Timer::new(Duration::from_millis(PLAYER_MOVE_TIMER_MS), TimerMode::Repeating)),
    ));

    println!("Created player succesfully");
}

/// Moves player entity from input
fn move_player(mut player_q: Query<&mut EntityTilePos>, mut ev_move: EventReader<MoveEvent>) {
    for ev in ev_move.iter() {
        match player_q.get_mut(ev.0) {
            Ok(mut player_tile_pos) => {
                player_tile_pos.x = ev.1.x;
                player_tile_pos.y = ev.1.y;
            }
            Err(_) => {}
        };
    }
}

// Returns true if the timer has finished in the frame
fn movement_cooldown(mut timer_q: Query<&mut HeldTimer, With<Player>>, time: Res<Time>) -> bool {
    let mut move_time = timer_q.single_mut();
    move_time.0.tick(time.delta());
    move_time.0.finished()
    // if keeb.any_just_pressed([KeyCode::D, KeyCode::A, KeyCode::S, KeyCode::W]) {
    //     move_time.0.tick(Duration::from_millis(PLAYER_MOVE_TIMER_MS));
    // }   // Too much power cannot slow down
}

fn interact_cooldown(mut timer_q: Query<&mut InteractTimer, With<Player>>, time: Res<Time>) -> bool {
    let mut interact_time = timer_q.single_mut();
    interact_time.0.tick(time.delta());
    interact_time.0.finished()
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
    blocking_q: Query<&TilePos, With<Blocking>>,
    blocking_interact_q: Query<(Entity, &TilePos), (With<Interact>, With<Blocking>)>,
    obj_tiles_q: Query<(Entity, &ObjectSize, &TilePos)>,
    tile_storage_q: Query<&TileStorage>,
    keeb: Res<Input<KeyCode>>,
    mut ev_moveplayer: EventWriter<MoveEvent>,
    mut ev_interact: EventWriter<Interaction>,
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

    // check if the tile is an interactable
    //   is the dest_tile part of a multi tile -> get owner entity
    //   is the owner entity in the interactable query -> get entity with components
    //   give entity to the interact system
    if let Some((dest_entity, size, _)) = obj_tiles_q.iter().find(|x| dest_tile.eq(x.2)) {
        match *size {
            ObjectSize::Single => {
                ev_interact.send(Interaction { sender: player_entity, reciever: dest_entity});
            },
            ObjectSize::Multi(owner) => {
                if let Ok(_) = blocking_interact_q.get(owner) {
                    ev_interact.send(Interaction { sender: player_entity, reciever: owner});
                };
            },
        }
    } else if let Some(_) = blocking_q.iter().find(|elem| dest_tile.eq(elem)) {
        return;
    }
    else {
        ev_moveplayer.send(MoveEvent(player_entity, dest_tile));
    }


    // if let Some((interactable_entity, _)) = blocking_interact_q.iter().find(|(_, elem)| dest_tile.eq(elem)) {
    //     ev_interact.send(Interaction { sender: player_entity, reciever: interactable_entity });
    // }
}

fn player_interact_handler(
    mut player_q: Query<Entity, (With<EntityTilePos>, With<Player>)>,
    mut interactables_q: Query<(&mut Interact, Entity, &TilePos)>,
    mut ev_interact: EventReader<Interaction>,
    mut ev_killed: EventWriter<HealthBelowZeroEvent>,
) {
    for ev in ev_interact.iter() {
        //TODO: remove this if not necessary
        let _ = match player_q.get_mut(ev.sender) {
            Ok(player_entity) => player_entity,
            Err(_) => return,
        };

        let mut interact = match interactables_q.get_mut(ev.reciever) {
            Ok(interact) => interact,
            Err(_) => return,
        };
        match &mut *interact.0 {
            // TODO: this should be moved into their own interact fns
            Interact::Harvest(health) => {
                if health.hp <= 0 {
                    return;
                }
                health.hp -= 2;
                println!("struck tree with fist hp: {}", health.hp);
                if health.hp <= 0 {
                    ev_killed.send(HealthBelowZeroEvent(interact.1, *interact.2));
                    println!("tree is dead");
                }
            }
            Interact::Pickup() => {}
            Interact::Consume() => {}
        }
    }
}

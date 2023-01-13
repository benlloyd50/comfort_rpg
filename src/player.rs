use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use iyes_loopless::prelude::*;

use crate::{assets::SpriteAssets, entity_tile_pos::EntityTilePos, world_gen::{Blocking, MAP_SIZE_X, MAP_SIZE_Y, world_size, within_bounds}, AppState};

pub const PLAYER_Z: f32 = 50.0;
const PLAYER_TILE_SPEED: u32 = 1;
const PLAYER_HELD_TIMER_MSEC: u64 = 100;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(AppState::GameLoading, setup_character.after("map"))
            .add_system(
                move_player
                    .run_in_state(AppState::Running)
                    .run_if_not(dest_tile_is_blocked)
                    .label("logic"),
            )
            .add_system(
                update_sprite_position::<Player>
                    .run_in_state(AppState::Running)
                    .label("graphic")
                    .after("logic"),
            );
    }
    //System Order Idea
    // player_input.label(Input::Listen).run_in_state(AppState::Running)
    // move_player.run_in_state(AppState::Running)
    //          .run_if(tile_is_unblocked).label(Input::Process).after(Input::Listen)
}

#[derive(Component)]
pub struct Player;

/// Timer used as an sleeper for held actions
#[derive(Component)]
struct HeldTimer {
    timer: Timer,
}

fn setup_character(mut commands: Commands, sprites: Res<SpriteAssets>, _blocking_q: Query<&TilePos, With<Blocking>>) {
    // Find first nonblocking tilepos
    let starting_pos = EntityTilePos { x: 64, y: 64 };
    // TODO test this out more and it doesn't work currently
    // loop {
    //     let blocked_tiles = blocking_q.iter_inner().filter(|elem| starting_pos.eq_tilepos(elem));
    //     let amt_blocked_tiles = blocked_tiles.count();
    //     println!("There were {} blocked tiles", amt_blocked_tiles);
    //     if amt_blocked_tiles <= 0 {
    //         break;
    //     }
    //
    //     for _ in blocking_q.iter_inner().filter(|elem| starting_pos.eq_tilepos(elem)) {
    //         println!("Moved player");
    //         starting_pos.x += 1;
    //         starting_pos.y += 1;
    //         break;
    //     }
    //
    // }

    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: sprites.characters.clone(),
            transform: Transform::from_xyz(starting_pos.x as f32 * 8.0, starting_pos.y as f32 * 8.0, 50.0),
            ..default()
        },
        Player,
        starting_pos,
        HeldTimer {
            timer: Timer::new(Duration::from_millis(PLAYER_HELD_TIMER_MSEC), TimerMode::Repeating),
        },
    ));

    println!("Created player succesfully");
}

/// Moves player entity from input
fn move_player(
    mut player_q: Query<(&mut EntityTilePos, &mut HeldTimer), With<Player>>,
    keeb: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (mut player_tile_pos, mut held_timer) = player_q.single_mut();
    held_timer.timer.tick(time.delta());

    // reset timer if tapping movement keys
    if keeb.any_just_pressed([KeyCode::W, KeyCode::S, KeyCode::A, KeyCode::D]) {
        held_timer
            .timer
            .set_duration(Duration::from_millis(PLAYER_HELD_TIMER_MSEC - 1));
    }

    // for when the button is `held`
    if !held_timer.timer.finished() {
        return;
    }

    if keeb.pressed(KeyCode::W) {
        player_tile_pos.y += PLAYER_TILE_SPEED
    } else if keeb.pressed(KeyCode::S) {
        player_tile_pos.y -= PLAYER_TILE_SPEED
    } else if keeb.pressed(KeyCode::D) {
        player_tile_pos.x += PLAYER_TILE_SPEED
    } else if keeb.pressed(KeyCode::A) {
        player_tile_pos.x -= PLAYER_TILE_SPEED
    }
}

/// Updates the sprite position based on a discrete position in the entity
fn update_sprite_position<Type: Component>(mut entity_q: Query<(&mut Transform, &EntityTilePos), With<Type>>) {
    let (mut sprite_pos, entity_actual_pos) = entity_q.single_mut();
    let destination_pos = entity_actual_pos.center_in_world();

    let lerped_pos = lerp(sprite_pos.translation.truncate(), destination_pos, 0.15);
    sprite_pos.translation = Vec3::new(lerped_pos.x, lerped_pos.y, PLAYER_Z);
}

/// Moves start vec towards finish vec by the scalar value (same in both directions)
fn lerp(start: Vec2, finish: Vec2, scalar: f32) -> Vec2 {
    start + (finish - start) * scalar
}

fn dest_tile_is_blocked(
    player_q: Query<&EntityTilePos, With<Player>>,
    blocking_q: Query<&TileStorage, With<Blocking>>,
    keeb: Res<Input<KeyCode>>,
) -> bool {
    // find the dest_tile which is player_pos + direction pressed
    let player_tile_pos = player_q.single();

    let mut dest_tile = Vec2::new(player_tile_pos.x as f32, player_tile_pos.y as f32);
    // else if here prevents dest_tile equalling zero delta and allowing a passthrough
    if keeb.pressed(KeyCode::W) {
        dest_tile.y += PLAYER_TILE_SPEED as f32
    } else if keeb.pressed(KeyCode::S) {
        dest_tile.y -= PLAYER_TILE_SPEED as f32
    } else if keeb.pressed(KeyCode::D) {
        dest_tile.x += PLAYER_TILE_SPEED as f32
    } else if keeb.pressed(KeyCode::A) {
        dest_tile.x -= PLAYER_TILE_SPEED as f32
    }

    if !within_bounds(dest_tile) {
        println!("Out of bounds check at {},{}", dest_tile.x, dest_tile.y);
        return true;
    }

    let dest_tile = TilePos{x: dest_tile.x as u32, y: dest_tile.y as u32};

    // compare the dest_tile to the blocker if there is one at that tile
    let blocking_tiles = match blocking_q.get_single() {
        Ok(e) => e,
        Err(_) => {
            println!("multiple storages :P");
            return true;
        } // if there are two tile storages we assume something is wrong and dont let the player move
    };

    match blocking_tiles.get(&dest_tile) {
        Some(_) => {
            println!("Blocking entity at {}, {}", dest_tile.x, dest_tile.y);
            true
        }
        None => {
            false 
        },
    }
}


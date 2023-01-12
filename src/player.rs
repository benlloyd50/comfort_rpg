use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use iyes_loopless::prelude::*;

use crate::{assets::SpriteAssets, entity_tile_pos::EntityTilePos, world_gen::Blocking, AppState};

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
                player_sprite_update
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

fn setup_character(mut commands: Commands, sprites: Res<SpriteAssets>, blocking_q: Query<&TilePos, With<Blocking>>) {
    // Find first nonblocking tilepos
    let mut starting_pos = EntityTilePos {x: 64, y: 64};
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

    // reset timer if tapping
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

fn player_sprite_update(mut player_q: Query<(&mut Transform, &EntityTilePos), With<Player>>) {
    let (mut player_pos, player_tile_pos) = player_q.single_mut();
    let actual_pos = player_tile_pos.center_in_world();

    let lerped_pos = lerp(player_pos.translation.truncate(), actual_pos, 0.25);
    player_pos.translation = Vec3::new(lerped_pos.x, lerped_pos.y, PLAYER_Z);
}

/// Moves start vec towards finish vec by the scalar value (same in both directions)
fn lerp(start: Vec2, finish: Vec2, scalar: f32) -> Vec2 {
    start + (finish - start) * scalar
}

fn dest_tile_is_blocked(
    player_q: Query<&EntityTilePos, With<Player>>,
    blocking_q: Query<(&Blocking, &TilePos), Without<Player>>,
    tile_storage_q: Query<&TileStorage>,
    keeb: Res<Input<KeyCode>>,
) -> bool {
    // find the dest_tile which is player_pos + direction pressed
    let player_tile_pos = player_q.single();

    let mut dest_tile = TilePos::new(player_tile_pos.x, player_tile_pos.y);
    // else if here prevents dest_tile equalling zero delta and allowing a passthrough
    if keeb.pressed(KeyCode::W) {
        dest_tile.y += PLAYER_TILE_SPEED
    } else if keeb.pressed(KeyCode::S) {
        dest_tile.y -= PLAYER_TILE_SPEED
    } else if keeb.pressed(KeyCode::D) {
        dest_tile.x += PLAYER_TILE_SPEED
    } else if keeb.pressed(KeyCode::A) {
        dest_tile.x -= PLAYER_TILE_SPEED
    }

    // println!("DestTile is {}, {}", dest_tile.x, dest_tile.y);

    // compare the dest_tile to the blocker if there is one at that tile
    let tile_storage = tile_storage_q.single();
    let tile_entity = tile_storage.get(&dest_tile).unwrap();
    match blocking_q.get(tile_entity) {
        Ok(_) => {
            println!("Blocking entity at {}, {}", dest_tile.x, dest_tile.y);
            true
        }
        _ => false,
    }
}

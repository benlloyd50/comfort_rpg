use bevy::prelude::*;
use iyes_loopless::prelude::*;

use crate::{AppState, assets::SpriteAssets};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(AppState::GameLoading, setup_character);

    }
    //System Order Idea
    // player_input.label(Input::Listen).run_in_state(AppState::Running)
    // move_player.run_in_state(AppState::Running)
    //          .run_if(tile_is_unblocked).label(Input::Process).after(Input::Listen)
}

#[derive(Component)]
struct Player;

fn setup_character(mut commands: Commands, sprites: Res<SpriteAssets>) {
    commands.spawn((SpriteSheetBundle {
        texture_atlas: sprites.characters.clone(),
        transform: Transform::from_xyz(64.0 * 8.0, 64.0 * 8.0, 50.0),
        ..default()
    },
    Player));

    println!("Created player succesfully");

}

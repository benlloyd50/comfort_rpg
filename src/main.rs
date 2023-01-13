mod entity_tile_pos;
mod constants;

mod assets;
use assets::AssetLoadPlugin;
mod world_gen;
use world_gen::WorldGenerationPlugin;
mod camera;
use camera::CameraPlugin;
mod player;
use player::PlayerPlugin;

use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_ecs_tilemap::TilemapPlugin;
use iyes_loopless::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    AssetLoading,
    GameLoading,
    Running,
}

fn main() {
    App::new()
        .add_loopless_state(AppState::AssetLoading) // Starting state which leads to the plugin doing its job first
        .add_plugin(DefaultPluginsWithImage)
        .add_plugin(AssetLoadPlugin)
        .add_plugin(TilemapPlugin)
        .add_plugin(WorldGenerationPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(PlayerPlugin)
        .add_system(run_game.run_in_state(AppState::GameLoading))
        .run();
}

// Gets the state out of Game Loading once everything is finished
fn run_game(mut commands: Commands) {
    commands.insert_resource(NextState(AppState::Running));
}

struct DefaultPluginsWithImage;
impl Plugin for DefaultPluginsWithImage {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        width: 640.0,
                        height: 480.0,
                        title: "Comfort RPG PROTOTYPE".to_string(),
                        present_mode: PresentMode::AutoVsync,
                        resizable: false,
                        ..Default::default()
                    },
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        );
    }
}

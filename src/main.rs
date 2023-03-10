mod comfort_config;
mod constants;
mod effects;
mod entity_tile_pos;

mod assets;
use assets::AssetLoadPlugin;
mod interact;
mod world_gen;
use interact::InteractPlugin;
use world_gen::WorldGenerationPlugin;
mod camera;
use camera::CameraPlugin;
mod player;
use player::PlayerPlugin;
mod item_util;
use item_util::ItemUtilPlugin;
mod inventory;
use inventory::InventoryPlugin;
mod crafting;
use crafting::CraftingPlugin;

use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_ecs_tilemap::TilemapPlugin;
use iyes_loopless::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    AssetLoading,
    GameLoading,
    Running,
    Menu,
}

fn main() {
    App::new()
        .add_loopless_state(GameState::AssetLoading) // Starting state which leads to the plugin doing its job first
        .add_plugin(DefaultPluginsWithImage)
        .add_plugin(AssetLoadPlugin)
        .add_plugin(TilemapPlugin)
        .add_plugin(WorldGenerationPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(InteractPlugin)
        .add_plugin(ItemUtilPlugin)
        .add_plugin(InventoryPlugin)
        .add_plugin(CraftingPlugin)
        .add_system(run_game.run_in_state(GameState::GameLoading))
        .add_system(bevy::window::close_on_esc)
        .run();
}

// Gets the state out of Game Loading once everything is finished
fn run_game(mut commands: Commands) {
    commands.insert_resource(NextState(GameState::Running));
}

struct DefaultPluginsWithImage;
impl Plugin for DefaultPluginsWithImage {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        width: 1080.0,
                        height: 720.0,
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

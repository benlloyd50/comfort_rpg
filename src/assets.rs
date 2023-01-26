use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::AppState;
pub struct AssetLoadPlugin;

impl Plugin for AssetLoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(AppState::AssetLoading)
                .continue_to_state(AppState::GameLoading)
                .with_collection::<SpriteAssets>(),
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct SpriteAssets {
    #[asset(path = "terrain.png")]
    pub terrain: Handle<Image>,

    #[asset(path = "world_objs.png")]
    pub world_objs: Handle<Image>,

    // TODO: consider joining with world_objs
    #[asset(path = "items.png")]
    pub items: Handle<Image>,

    #[asset(texture_atlas(tile_size_x = 8., tile_size_y = 8., columns = 8, rows = 1))]
    #[asset(path = "characters.png")]
    pub characters: Handle<TextureAtlas>,

    #[asset(path = "plants.png")]
    pub plants: Handle<Image>,

    #[asset(texture_atlas(tile_size_x = 8., tile_size_y = 16., columns = 1, rows = 1))]
    #[asset(path = "tree1.png")]
    pub tree: Handle<TextureAtlas>,

    #[asset(path = "target_outline.png")]
    pub target: Handle<Image>,
}

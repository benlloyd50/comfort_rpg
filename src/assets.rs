use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::AppState;
pub struct AssetLoadPlugin;

impl Plugin for AssetLoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(AppState::AssetLoading)
                .continue_to_state(AppState::GameLoading)
                .with_dynamic_collections::<StandardDynamicAssetCollection>(vec!["dynamic_asset.assets"])
                .with_collection::<SpriteAssets>()
                .with_collection::<FontAssets>()
                .with_collection::<UiAssets>(),
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct FontAssets {
    #[asset(key = "chunkfive")]
    pub chunk: Handle<Font>,
}

#[derive(AssetCollection, Resource)]
pub struct UiAssets {
    #[asset(key = "menubg")]
    pub menubg: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct SpriteAssets {
    #[asset(key = "terrain")]
    pub terrain: Handle<Image>,

    #[asset(key = "world_objs")]
    pub world_objs: Handle<Image>,

    #[asset(key = "items")]
    pub items: Handle<Image>,

    #[asset(key = "characters")]
    pub characters: Handle<TextureAtlas>,

    #[asset(key = "plants")]
    pub plants: Handle<Image>,

    #[asset(key = "tree")]
    pub tree: Handle<TextureAtlas>,

    #[asset(key = "target")]
    pub target: Handle<Image>,
}

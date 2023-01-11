// Player has an entitytilepos component
// Shouldn't be grouped into most queries as it is different from tiles
// this allow for entities on a tile to be handled differently, ie attack, mine, interact, etc
// move_player updates tile_pos and another system will move the character
// this opens up potential for seperate space and character movement, ie animations
// this also means game logic will use TilePos or EntityTilePos for any logic
// and the Transform component on the player, atleast, will be purely graphical
//
// This should ultimately end up in its own class as some point
// 1/11/23 TODO I also currently have concerns that this is justified over TilePos for no good reason currently
// This should be addressed by the end of the prototype

/// EntityTilePos
/// An entity position on the grid
use crate::world_gen::{TILE_PIXELS_X, TILE_PIXELS_Y};

use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;

#[derive(Component)]
pub struct EntityTilePos {
    pub x: u32,
    pub y: u32,
}

impl EntityTilePos {
    pub fn center_in_world(&self) -> Vec2 {
        Vec2::new(TILE_PIXELS_X * self.x as f32, TILE_PIXELS_Y * self.y as f32)
    }

    #[allow(dead_code)]
    /// Checks if the EntityTilePos is equal to a TilePos based on x, y values
    pub fn eq_tilepos(&self, other: &TilePos) -> bool {
        self.x == other.x && self.y == other.y
    }
}

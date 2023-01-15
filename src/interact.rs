use crate::world_gen::Blocking;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub struct InteractPlugin;

impl Plugin for InteractPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<HealthBelowZeroEvent>().add_system(destroy_dead);
    }
}

/// Type of the interaction with relavent information
#[derive(Component)]
pub enum Interact {
    Harvest(Health),
    #[allow(dead_code)]
    Pickup(),
    #[allow(dead_code)]
    Consume(),
}

pub struct HealthBelowZeroEvent(pub Entity, pub TilePos);

pub struct Health {
    pub max_hp: u32,
    pub hp: i32,
}

impl Health {
    pub fn new(hp: u32) -> Health {
        Health {
            max_hp: hp,
            hp: hp as i32,
        }
    }
}

fn destroy_dead(
    mut commands: Commands,
    mut ev_killed: EventReader<HealthBelowZeroEvent>,
    mut tile_storage_q: Query<&mut TileStorage, With<Blocking>>,
) {
    for ev in ev_killed.iter() {
        for mut tile_storage in tile_storage_q.iter_mut() {
            tile_storage.remove(&ev.1);
        }
        commands.entity(ev.0).despawn_recursive();
    }
}

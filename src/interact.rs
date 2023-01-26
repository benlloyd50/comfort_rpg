use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    item_util::SpawnItemEvent,
    player::SystemOrder,
    world_gen::{Blocking, ObjectSize},
    AppState,
};

pub struct InteractPlugin;

impl Plugin for InteractPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<HealthBelowZeroEvent>()
            .add_event::<HarvestInteraction>()
            .add_system(cleanup_world_objs)
            .add_system(
                harvest_interact_handler
                    .run_in_state(AppState::Running)
                    .label(SystemOrder::Logic)
                    .after(SystemOrder::Input),
            );
    }
}

/// Type of the interaction with relavent information
#[derive(Component)]
#[allow(dead_code)]
pub enum Interact {
    Harvest,
    Pickup,
    Consume,
}

pub struct HealthBelowZeroEvent(pub Entity, pub TilePos);

#[derive(Component)]
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

// Removes dead world objs that are sent via the HealthBelowZeroEvent
fn cleanup_world_objs(
    mut commands: Commands,
    mut ev_killed: EventReader<HealthBelowZeroEvent>,
    mut tile_storage_q: Query<&mut TileStorage, With<Blocking>>,
    world_objs_q: Query<(Entity, &ObjectSize, &TilePos)>,
) {
    for ev in ev_killed.iter() {
        for (obj, obj_size, obj_pos) in world_objs_q.iter() {
            match obj_size {
                ObjectSize::Single => {
                    for mut tile_storage in tile_storage_q.iter_mut() {
                        tile_storage.remove(&ev.1);
                    }
                    commands.entity(ev.0).despawn_recursive();
                }
                ObjectSize::Multi(owner) => {
                    if *owner == ev.0 {
                        // will remove the owner and the tiles associated with the owner
                        for mut tile_storage in tile_storage_q.iter_mut() {
                            tile_storage.remove(obj_pos);
                        }
                        commands.entity(obj).despawn_recursive();
                    }
                }
            }
        }
    }
}

pub struct HarvestInteraction {
    pub harvester: Entity,       
    pub harvested: Entity,     
    pub reciever_pos: TilePos,
}

fn harvest_interact_handler(
    mut interactables_q: Query<(Entity, &Interact, &mut Health, &TilePos)>,
    mut ev_harvest: EventReader<HarvestInteraction>,
    mut ev_destroyed: EventWriter<HealthBelowZeroEvent>,
    mut ev_spawnitem: EventWriter<SpawnItemEvent>,
) {
    for ev in ev_harvest.iter() {
        if let Ok((interactable, _, mut health, pos)) = interactables_q.get_mut(ev.harvested) {
            if health.hp <= 0 {
                return;
            }
            health.hp -= 2;
            println!("struck obj with fist hp: {}", health.hp);
            if health.hp <= 0 {
                ev_destroyed.send(HealthBelowZeroEvent(interactable, *pos));
                ev_spawnitem.send(SpawnItemEvent::from(ev.reciever_pos.x, ev.reciever_pos.y, 1));
                println!("obj is dead");
            }
        }
    }
}

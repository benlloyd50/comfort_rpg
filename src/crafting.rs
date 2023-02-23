use bevy::{prelude::*, utils::HashMap};
use iyes_loopless::prelude::*;
use serde::Deserialize;
use std::{error::Error, fs};

use crate::{
    inventory::Inventory,
    item_util::{ItemId, ItemQuantity},
    player::Player,
    GameState,
};

pub struct CraftingPlugin;
impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::GameLoading, init_recipe_database)
            .add_system(
                handle_crafting_event
                    .run_in_state(GameState::Running)
                    .run_on_event::<CraftItemEvent>(),
            )
            .add_system(testcraft.run_in_state(GameState::Running))
            .add_event::<CraftItemEvent>();
    }
}

struct CraftItemEvent {
    who: Entity,      // the entity crafting the item so we know what inventory to use
    recipe: RecipeId, // what is being crafted
}

fn handle_crafting_event(
    mut inventory_q: Query<&mut Inventory, With<Player>>,
    mut ev_crafting: EventReader<CraftItemEvent>,
    recipe_db: Res<RecipeDatabase>,
) {
    for ev in ev_crafting.iter() {
        if let Ok(mut inventory) = inventory_q.get_mut(ev.who) {
            if let Some(recipe) = recipe_db.recipes.get(&ev.recipe) {
                for ingredient in recipe.ingredients.iter() {
                    if !inventory.contains_item(ingredient.item_id, &ingredient.item_quantity) {
                        println!("cannot craft, not enough {:?}", ingredient.item_id);
                        return;
                    }
                }
                for ingredient in recipe.ingredients.iter() {
                    inventory.remove_item(ingredient.item_id, &ingredient.item_quantity);
                }
                println!("crafted {:?}", recipe.id);
                inventory.add_item(recipe.output_id, &recipe.output_amt);
            }
        }
    }
}

fn testcraft(
    keeb: Res<Input<KeyCode>>,
    player_q: Query<Entity, With<Player>>,
    mut ev_crafting: EventWriter<CraftItemEvent>,
) {
    if !keeb.just_pressed(KeyCode::Key7) {
        return;
    }
    if let Ok(player) = player_q.get_single() {
        ev_crafting.send(CraftItemEvent {
            who: player,
            recipe: RecipeId(1),
        });
    }
}

// Static information about the recipe that is set
#[derive(Deserialize, Debug, Component, Clone)]
pub struct Recipe {
    id: RecipeId,
    ingredients: Vec<Ingredient>,
    output_id: ItemId,
    output_amt: ItemQuantity,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Ingredient {
    item_id: ItemId,
    item_quantity: ItemQuantity,
}

#[derive(Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct RecipeId(u32);

// Maps all items to a unique u32, loaded on startup and should not be mutated at runtime
#[derive(Resource)]
pub struct RecipeDatabase {
    pub recipes: HashMap<RecipeId, Recipe>,
}

fn init_recipe_database(mut commands: Commands) {
    let recipes = match load_from_json("comfort_recipes") {
        Ok(recipes) => recipes,
        Err(err) => panic!("Could not load recipes from json, {err}\n"),
    };

    let mut recipe_db = HashMap::new();
    for recipe in recipes {
        recipe_db.insert(recipe.id, recipe);
    }

    commands.insert_resource(RecipeDatabase { recipes: recipe_db });
}

// Attempts to load item definitions from a json file
fn load_from_json(name: &str) -> Result<Vec<Recipe>, Box<dyn Error>> {
    let contents = fs::read_to_string(format!("assets/items/{name}.json"))?;
    let recipes: Vec<Recipe> = serde_json::from_str(&contents)?;
    Ok(recipes)
}

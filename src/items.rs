use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use ambient_api::{components::core::primitives::cube, concepts::make_transformable, prelude::*};

use crate::components::{crafting::*, player};

lazy_static::lazy_static! {
    pub static ref BLUE_ITEM: EntityId = Entity::new()
        .with(color(), vec4(0.0, 0.0, 1.0, 1.0))
        .spawn();

    pub static ref GREEN_ITEM: EntityId = Entity::new()
        .with(color(), vec4(0.0, 1.0, 0.0, 1.0))
        .spawn();

    pub static ref YELLOW_ITEM: EntityId = Entity::new()
        .with(color(), vec4(1.0, 1.0, 0.0, 1.0))
        .spawn();
}

/// Wasm-side crafting recipe data.
pub struct CraftingRecipe {
    pub recipe_entity: EntityId,
    pub primary_ingredient: EntityId,
    pub secondary_ingredient: EntityId,
    pub primary_yield: EntityId,
    pub secondary_yield: EntityId,
}

/// The set of all available crafting recipes.
pub struct RecipeStore {
    recipes: HashMap<(EntityId, EntityId), CraftingRecipe>,
}

impl RecipeStore {
    pub fn new() -> Self {
        Self {
            recipes: Default::default(),
        }
    }

    pub fn match_ingredients(
        &self,
        left_ingredient: EntityId,
        right_ingredient: EntityId,
    ) -> Option<(&CraftingRecipe, bool)> {
        if let Some(recipe) = self.recipes.get(&(left_ingredient, right_ingredient)) {
            Some((recipe, false))
        } else if let Some(recipe) = self.recipes.get(&(right_ingredient, left_ingredient)) {
            Some((recipe, true))
        } else {
            None
        }
    }

    pub fn apply_craft(
        &self,
        left_held: EntityId,
        right_held: EntityId,
    ) -> Option<(EntityId, EntityId)> {
        let (recipe, right_is_primary) = self.match_ingredients(left_held, right_held)?;
        if !right_is_primary {
            Some((recipe.primary_yield, recipe.secondary_yield))
        } else {
            Some((recipe.secondary_yield, recipe.primary_yield))
        }
    }
}

/// Sets up server-side item- and crafting-related systems.
#[cfg(feature = "server")]
pub fn init_server_items() {
    let store = RecipeStore::new();
    let store = Arc::new(Mutex::new(store));

    spawn_query((
        recipe(),
        primary_ingredient(),
        secondary_ingredient(),
        primary_yield(),
        secondary_yield(),
    ))
    .bind({
        let store = store.clone();
        move |recipes| {
            let mut store = store.lock().unwrap();
            for (
                e,
                (_recipe, primary_ingredient, secondary_ingredient, primary_yield, secondary_yield),
            ) in recipes
            {
                let recipe = CraftingRecipe {
                    recipe_entity: e,
                    primary_ingredient,
                    secondary_ingredient,
                    primary_yield,
                    secondary_yield,
                };

                let recipe_key = (primary_ingredient, secondary_ingredient);

                if store.recipes.contains_key(&recipe_key)
                    || store
                        .recipes
                        .contains_key(&(secondary_ingredient, primary_ingredient))
                {
                    eprintln!("Duplicate crafting recipe");
                    continue;
                }

                store.recipes.insert(recipe_key, recipe);
            }
        }
    });

    crate::messages::PlayerCraftInput::subscribe({
        let store = store.clone();
        move |source, _| {
            let Some(player_entity) = source.client_entity_id() else { return; };
            let Some(player_head) = entity::get_component(player_entity, player::head_ref()) else { return; };
            let Some(left_hand) = entity::get_component(player_head, player::left_hand_ref()) else { return; };
            let Some(right_hand) = entity::get_component(player_head, player::right_hand_ref()) else { return; };

            let left_held =
                entity::get_component(left_hand, player::held_item_ref()).unwrap_or_default();

            let right_held =
                entity::get_component(right_hand, player::held_item_ref()).unwrap_or_default();

            let store = store.lock().unwrap();
            if let Some((new_left_held, new_right_held)) = store.apply_craft(left_held, right_held)
            {
                entity::add_component(left_hand, player::held_item_ref(), new_left_held);
                entity::add_component(right_hand, player::held_item_ref(), new_right_held);
            }
        }
    });

    // temp crafting recipe
    Entity::new()
        .with_default(recipe())
        .with(primary_ingredient(), *BLUE_ITEM)
        .with(secondary_ingredient(), *YELLOW_ITEM)
        .with(primary_yield(), *GREEN_ITEM)
        .with(secondary_yield(), EntityId::null())
        .spawn();
}

/// Inits client-side item- and crafting-related systems.
#[cfg(feature = "client")]
pub fn init_client_items() {
    change_query((player::held_item_ref(), user_id()))
        .track_change(player::held_item_ref())
        .bind(move |changes| {
            for (hand, (item, uid)) in changes {
                for child in entity::get_component(hand, children()).unwrap_or_default() {
                    entity::despawn_recursive(child);
                }

                if item.is_null() {
                    continue;
                }

                let item_color = entity::get_component(item, color());
                let new_color = item_color.unwrap_or(vec4(1.0, 0.0, 1.0, 1.0));

                let item_instance = Entity::new()
                    .with_default(local_to_parent())
                    .with_default(cube())
                    .with(color(), new_color)
                    .with(user_id(), uid)
                    .spawn();

                entity::add_child(hand, item_instance);
            }
        });
}

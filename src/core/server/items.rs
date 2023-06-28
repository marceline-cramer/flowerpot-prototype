use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use ambient_api::prelude::*;

use crate::components::{
    crafting::*,
    crops::{self, medium_occupant_ref},
    items::*,
    map,
};
use crate::player::PlayerEntities;

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
            let Some(mut player) = PlayerEntities::from_source(&source) else { return; };
            let store = store.lock().unwrap();
            let crafted = store.apply_craft(player.left_held, player.right_held);
            if let Some((new_left_held, new_right_held)) = crafted {
                player.set_left_held(new_left_held);
                player.set_right_held(new_right_held);
            }
        }
    });

    crate::messages::PlayerSwapItemsInput::subscribe(move |source, _| {
        let Some(mut player) = PlayerEntities::from_source(&source) else { return; };
        let left_held = player.left_held;
        let right_held = player.right_held;
        player.set_left_held(right_held);
        player.set_right_held(left_held);
    });

    crate::messages::PlayerPickUpItemInput::subscribe(move |source, data| {
        let Some(mut player) = PlayerEntities::from_source(&source) else { return };

        match entity::get_component(data.target, map::position()) {
            Some(_) => {}   // TODO range checking for pickups
            None => return, // invalid item grab
        }

        let Some(class) = entity::get_component(data.target, class_ref()) else {
            eprintln!("Item {:?} has no class", data.target);
            return;
        };

        if player.right_held.is_null() {
            player.set_right_held(class);
        } else if player.left_held.is_null() {
            player.set_left_held(class);
        } else {
            return;
        }

        // TODO proper item instance management
        entity::remove_component(data.target, map::position());
    });

    crate::messages::PlayerDropItemInput::subscribe(move |source, data| {
        let Some(mut player) = PlayerEntities::from_source(&source) else { return };

        let Some(position) = entity::get_component(player.entity, map::position()) else {
            eprintln!("Player {:?} has no position", player.entity);
            return;
        };

        let class;
        if data.hand {
            class = player.right_held;
            player.set_right_held(EntityId::null());
        } else {
            class = player.left_held;
            player.set_left_held(EntityId::null());
        }

        if class.is_null() {
            return;
        }

        Entity::new()
            .with(map::position(), position)
            .with(class_ref(), class)
            .spawn();
    });

    crate::messages::PlayerUseItemInput::subscribe(move |source, data| {
        let Some(mut player) = PlayerEntities::from_source(&source) else { return };

        let held = if data.hand {
            player.right_held
        } else {
            player.left_held
        };

        if let Some(crop) = entity::get_component(held, plantable_crop_class_ref()) {
            if !entity::has_component(data.target_ref, map::tile())
                || entity::has_component(data.target_ref, medium_occupant_ref())
            {
                return;
            }

            entity::add_component(
                data.target_ref,
                medium_occupant_ref(),
                crate::crop::new_medium(crop, data.target_ref),
            );

            if data.hand {
                player.set_right_held(EntityId::null());
            } else {
                player.set_left_held(EntityId::null());
            }
        } else if let Some(crop) = entity::get_component(data.target_ref, crops::class_ref()) {
            let Some(item) = entity::get_component(crop, crops::harvest_item_class_ref()) else { return };

            if !entity::has_component(data.target_ref, crops::is_medium_crop()) {
                return;
            }

            let hand = if data.hand {
                player.right_held
            } else {
                player.left_held
            };

            if !hand.is_null() {
                return;
            }

            let Some(tile) = entity::get_component(data.target_ref, map::on_tile()) else { return };

            entity::despawn(data.target_ref);
            entity::remove_component(tile, medium_occupant_ref());

            if data.hand {
                player.set_right_held(item);
            } else {
                player.set_left_held(item);
            }
        }
    });
}

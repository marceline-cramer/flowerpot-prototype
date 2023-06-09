use ambient_api::{
    components::core::{
        ecs::children,
        prefab::prefab_from_url,
        primitives::cube,
        rendering::color,
        transform::{local_to_parent, local_to_world},
    },
    prelude::*,
};

use crate::components::{items, map, player};

pub fn init_items() {
    crate::shared::partitioning::init_qbvh(
        items::class_ref(),
        items::search_radius(),
        items::search_result(),
    );

    change_query(player::held_item_ref())
        .track_change(player::held_item_ref())
        .bind(move |changes| {
            for (hand, class) in changes {
                spawn_item_model(hand, class);
            }
        });

    spawn_query((map::position(), items::class_ref())).bind(move |items| {
        for (e, (_map_pos, class)) in items {
            entity::add_component(e, local_to_world(), Default::default());
            spawn_item_model(e, class);
        }
    });

    despawn_query((map::position(), items::class_ref())).bind(move |items| {
        for (e, _) in items {
            // TODO this is hacky because item instances are placed on the map
            // at the time of writing player's hands reference classes directly.
            entity::despawn_recursive(e);
        }
    });
}

/// Helper function to spawn models of items.
fn spawn_item_model(parent: EntityId, class: EntityId) {
    for child in entity::get_component(parent, children()).unwrap_or_default() {
        entity::despawn_recursive(child);
    }

    if class.is_null() {
        return;
    }

    let mut item_instance = Entity::new().with_default(local_to_parent());

    if let Some(new_color) = entity::get_component(class, color()) {
        item_instance.set(color(), new_color);
    }

    if let Some(prefab) = entity::get_component(class, items::prefab_path()) {
        item_instance.set(prefab_from_url(), asset::url(prefab).unwrap());
    } else {
        item_instance.set(cube(), ());
    }

    entity::add_child(parent, item_instance.spawn());
}

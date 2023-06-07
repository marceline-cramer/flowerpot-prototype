use ambient_api::{components::core::primitives::cube, prelude::*};

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

    let item_color = entity::get_component(class, color());
    let new_color = item_color.unwrap_or(vec4(1.0, 0.0, 1.0, 1.0));

    let item_instance = Entity::new()
        .with_default(local_to_parent())
        .with_default(cube())
        .with(color(), new_color)
        .spawn();

    entity::add_child(parent, item_instance);
}

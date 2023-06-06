use ambient_api::{components::core::primitives::cube, prelude::*};

use crate::components::player;

pub fn init_items() {
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

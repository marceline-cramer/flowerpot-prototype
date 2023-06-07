use ambient_api::prelude::*;

use crate::components::map::{elevation, position};

pub fn init_map() {
    // update entities' translation with map coordinates
    change_query(position())
        .track_change(position())
        .bind(move |changes| {
            for (e, xy) in changes {
                update_transform(e, xy);
            }
        });

    spawn_query(position()).bind(move |changes| {
        for (e, xy) in changes {
            update_transform(e, xy);
        }
    });
}

/// Helper function to update a map-positioned transform.
fn update_transform(target: EntityId, xy: Vec2) {
    let elevation = entity::get_component(target, elevation()).unwrap_or(0.0);
    entity::add_component(target, translation(), xy.extend(elevation));
}

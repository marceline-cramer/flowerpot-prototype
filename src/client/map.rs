use ambient_api::prelude::*;

use crate::components::map::{elevation, position};

pub fn init_map() {
    // update entities' translation with map coordinates
    change_query(position())
        .track_change(position())
        .bind(move |changes| {
            for (e, xy) in changes {
                let elevation = entity::get_component(e, elevation()).unwrap_or(0.0);
                entity::add_component(e, translation(), xy.extend(elevation));
            }
        });
}

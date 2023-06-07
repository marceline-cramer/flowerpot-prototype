use ambient_api::prelude::*;

use crate::components::{map_elevation, map_position};

pub fn init_map() {
    // update entities' translation with map coordinates
    change_query(map_position())
        .track_change(map_position())
        .bind(move |changes| {
            for (e, xy) in changes {
                let elevation = entity::get_component(e, map_elevation()).unwrap_or(0.0);
                entity::add_component(e, translation(), xy.extend(elevation));
            }
        });
}

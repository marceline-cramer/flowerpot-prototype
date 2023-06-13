use std::{collections::HashMap, sync::Mutex};

use ambient_api::{
    components::core::primitives::quad, concepts::make_transformable, glam::IVec2, prelude::*,
};

use crate::components::{cover_crop_occupant, map::*};

lazy_static::lazy_static! {
    pub static ref MAP: Mutex<HashMap<IVec2, EntityId>> = Mutex::new(HashMap::new());
}

pub fn init_map() {
    // set new entities' translation with map_position
    spawn_query(position()).bind(move |changes| {
        for (e, xy) in changes {
            update_transform(e, xy);
        }
    });

    // update entities' translation with map_position
    change_query(position())
        .track_change(position())
        .bind(move |changes| {
            for (e, xy) in changes {
                update_transform(e, xy);
            }
        });

    // render map tiles
    spawn_query((tile(), position())).bind(move |tiles| {
        for (tile, (_, xy)) in tiles {
            entity::add_components(
                tile,
                make_transformable()
                    .with(translation(), xy.extend(0.0))
                    .with_default(quad()),
            );
        }
    });

    // add tiles to the map
    spawn_query((tile(), position())).bind(move |tiles| {
        let mut map = MAP.lock().unwrap();
        for (tile, (_, xy)) in tiles {
            let xy = (xy + 0.5).floor().as_ivec2();
            map.insert(xy, tile);
        }
    });

    // remove tiles from the map
    despawn_query((tile(), position())).bind(move |tiles| {
        let mut map = MAP.lock().unwrap();
        for (_tile, (_, xy)) in tiles {
            let xy = (xy + 0.5).floor().as_ivec2();
            map.remove(&xy);
        }
    });

    // set soil tiles material
    spawn_query((tile(), soil()))
        // .excludes(cover_crop_occupant())
        .bind(move |tiles| {
            for (e, (_, _)) in tiles {
                entity::set_component(
                    e,
                    pbr_material_from_url(),
                    asset::url("assets/materials/materials/pipeline.json/0/mat.json").unwrap(),
                );
            }
        });

    // update materials of tiles with cover crops
    change_query((tile(), cover_crop_occupant()))
        .track_change(cover_crop_occupant())
        .bind(move |tiles| {
            for (e, (_, cover_crop)) in tiles {
                if let Some(new_mat) = entity::get_component(cover_crop, pbr_material_from_url()) {
                    entity::add_component(e, pbr_material_from_url(), new_mat);
                }
            }
        });
}

/// Helper function to update a map-positioned transform.
fn update_transform(target: EntityId, xy: Vec2) {
    let elevation = entity::get_component(target, elevation()).unwrap_or(0.0);
    entity::add_component(target, translation(), xy.extend(elevation));
}

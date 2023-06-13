use ambient_api::{components::core::primitives::quad, concepts::make_transformable, prelude::*};

use crate::components::{cover_crop_occupant, map::*};

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

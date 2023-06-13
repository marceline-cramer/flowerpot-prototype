use ambient_api::{prelude::*, rand};

use crate::{
    components::{crops::*, *},
    messages,
};

/// Sets up crop-related systems.
pub fn init_crops() {
    // init cover crop searching
    crate::shared::partitioning::init_qbvh(
        cover_crop_occupant(),
        search_cover_crop_radius(),
        search_cover_crop_result(),
    );

    messages::GrowTick::subscribe({
        let growable_query = query((map::tile(), cover_crop_occupant())).build();
        let mut rng = rand::thread_rng();
        move |_, _| {
            for (tile, (_, cover_crop)) in growable_query.evaluate() {
                crate::map::for_random_neighbors(&mut rng, tile, |neighbor| {
                    if entity::has_component(neighbor, cover_crop_occupant()) {
                        None
                    } else {
                        entity::add_component(neighbor, cover_crop_occupant(), cover_crop);
                        Some(())
                    }
                });
            }
        }
    });

    messages::GrowTick::subscribe({
        let growable_query = query((map::on_tile(), is_medium_crop(), class_ref())).build();
        let mut rng = rand::thread_rng();
        move |_, _| {
            for (crop, (tile, _, class)) in growable_query.evaluate() {
                if let Some(seed_class) = entity::get_component(class, seed_ref()) {
                    crate::map::for_random_neighbors(&mut rng, tile, |neighbor| {
                        if entity::has_component(neighbor, medium_occupant_ref()) {
                            None
                        } else {
                            let child = new_medium(seed_class, neighbor);
                            entity::add_component(neighbor, medium_occupant_ref(), child);
                            Some(())
                        }
                    });
                }

                if let Some(next_class) = entity::get_component(class, next_growth_phase_ref()) {
                    let next_instance = new_medium(next_class, tile);
                    entity::set_component(tile, medium_occupant_ref(), next_instance);
                    entity::despawn(crop);
                }
            }
        }
    });

    run_async(async move {
        loop {
            sleep(1.0).await;
            messages::GrowTick::new().send_local_broadcast(true);
        }
    });
}

/// Helper function to instantiate a medium crop.
pub fn new_medium(class: EntityId, tile: EntityId) -> EntityId {
    Entity::new()
        .with(class_ref(), class)
        .with(is_medium_crop(), ())
        .with(map::on_tile(), tile)
        .with(
            map::position(),
            entity::get_component(tile, map::position()).unwrap(),
        )
        .spawn()
}

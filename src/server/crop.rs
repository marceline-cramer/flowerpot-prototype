use ambient_api::{prelude::*, rand};

use crate::{components::*, messages};

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

    run_async(async move {
        loop {
            sleep(10.0).await;
            messages::GrowTick::new().send_local_broadcast(true);
        }
    });
}

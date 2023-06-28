use ambient_api::{
    components::core::{
        ecs::children,
        prefab::prefab_from_url,
        transform::{local_to_parent, local_to_world},
    },
    prelude::*,
};

use crate::components::{crops::*, map};

pub fn init_crops() {
    spawn_query((is_medium_crop(), class_ref(), map::on_tile())).bind(move |crops| {
        for (crop, (_, class, _tile)) in crops {
            entity::add_component(crop, local_to_world(), Default::default());

            for child in entity::get_component(crop, children()).unwrap_or_default() {
                entity::despawn_recursive(child);
            }

            let Some(prefab) = entity::get_component(class, prefab_url()) else { continue };

            let crop_model = Entity::new()
                .with_default(local_to_parent())
                .with(prefab_from_url(), asset::url(prefab).unwrap())
                .spawn();

            entity::add_child(crop, crop_model);
        }
    });

    despawn_query((is_medium_crop(), map::on_tile(), children())).bind(move |crops| {
        for (e, (_, _, children)) in crops {
            for child in children {
                entity::despawn_recursive(child);
            }
        }
    });
}

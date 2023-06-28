use ambient_api::{components::core::transform::rotation, prelude::*};

use crate::components::*;

/// Sets up queries relating to fauna.
pub fn init_fauna() {
    // decrease fauna fullness by hunger rate
    query((fullness(), hunger_rate()))
        .requires(fauna())
        .each_frame(|entities| {
            for (e, (old_fullness, hunger_rate)) in entities {
                let fullness_delta = hunger_rate * delta_time();
                let new_fullness = old_fullness - fullness_delta;
                entity::set_component(e, fullness(), new_fullness);
            }
        });

    // kill fauna with non-positive fullness
    change_query(fullness())
        .track_change(fullness())
        .requires(fauna())
        .bind(|changed| {
            for (e, fullness) in changed {
                if fullness <= 0.0 {
                    entity::despawn(e);
                }
            }
        });

    // passive metabolism refills fauna stamina
    query((stamina(), passive_metabolism()))
        .requires(fauna())
        .each_frame(|entities| {
            for (e, (old_stamina, metabolism)) in entities {
                let new_stamina = old_stamina + metabolism * delta_time();
                entity::set_component(e, stamina(), new_stamina);
            }
        });

    // move fauna
    query((
        fauna(),
        map::position(),
        map::on_tile(),
        stamina(),
        fullness(),
        movement_cost(),
        movement_distance(),
        search_cover_crop_result(),
    ))
    .excludes(movement_step())
    .each_frame(|entities| {
        for (
            e,
            (
                _fauna,
                map_pos,
                tile,
                old_stamina,
                old_fullness,
                movement_cost,
                movement_distance,
                search_result,
            ),
        ) in entities
        {
            if old_stamina < movement_cost {
                continue;
            }

            entity::remove_component(e, search_cover_crop_result());

            let new_stamina = old_stamina - movement_cost;
            entity::set_component(e, stamina(), new_stamina);

            if let Some(cover_crop) = entity::get_component(tile, cover_crop_occupant()) {
                if let Some(sustenance) = entity::get_component(cover_crop, sustenance()) {
                    let new_fullness = old_fullness + sustenance;
                    entity::set_component(e, fullness(), new_fullness);
                    entity::remove_component(tile, cover_crop_occupant());
                    continue;
                }
            }

            if search_result.is_null() {
                continue;
            }

            let target_pos = match entity::get_component(search_result, map::position()) {
                Some(target_pos) => target_pos,
                None => continue,
            };

            let target_delta = target_pos - map_pos;
            let movement_delta = target_delta.clamp_length_max(movement_distance);
            let movement_theta = -movement_delta.angle_between(-Vec2::X);

            if !movement_theta.is_finite() {
                continue;
            }

            let components = Entity::new()
                .with(rotation(), Quat::from_rotation_z(movement_theta))
                .with(movement_step(), 0.0)
                .with(movement_duration(), 0.25)
                .with(movement_start(), map_pos)
                .with(movement_target(), map_pos + movement_delta)
                .with(movement_height(), 0.5);

            entity::add_components(e, components);
        }
    });
}

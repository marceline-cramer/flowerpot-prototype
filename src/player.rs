use std::f32::consts::{FRAC_PI_2, TAU};

use ambient_api::{
    components::core::{camera::aspect_ratio_from_window, primitives::cube},
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    prelude::*,
};

use crate::{components::*, messages};

/// Sets up player-related systems.
pub fn init_players() {
    spawn_query((player(), user_id())).bind(move |players| {
        for (e, (_player, uid)) in players {
            let head = Entity::new()
                .with_merge(make_perspective_infinite_reverse_camera())
                .with(aspect_ratio_from_window(), EntityId::resources())
                .with_default(main_scene())
                .with(user_id(), uid)
                .with(translation(), Vec3::Z * 1.5)
                .with(parent(), e)
                .with_default(local_to_parent())
                .with(rotation(), Quat::from_rotation_x(FRAC_PI_2))
                .spawn();

            entity::add_components(
                e,
                Entity::new()
                    .with_merge(make_transformable())
                    .with_default(cube())
                    .with(map_position(), Vec2::new(16.0, 16.0))
                    .with(children(), vec![head])
                    .with(player_head_ref(), head)
                    .with(player_pitch(), 0.0)
                    .with(player_yaw(), 0.0),
            );
        }
    });

    messages::PlayerMovementInput::subscribe(move |source, msg| {
        let Some(player_id) = source.client_entity_id() else { return; };

        let direction = msg.direction.normalize_or_zero();
        let yaw = msg.yaw % TAU;
        let pitch = msg.pitch.clamp(-FRAC_PI_2, FRAC_PI_2);

        entity::add_components(
            player_id,
            Entity::new()
                .with(player_movement_direction(), direction)
                .with(player_yaw(), yaw)
                .with(player_pitch(), pitch),
        );
    });

    change_query((player(), player_yaw(), player_pitch()))
        .track_change((player_yaw(), player_pitch()))
        .bind(move |players| {
            for (e, (_player, yaw, pitch)) in players {
                entity::set_component(e, rotation(), Quat::from_rotation_z(yaw));
                if let Some(head) = entity::get_component(e, player_head_ref()) {
                    entity::set_component(
                        head,
                        rotation(),
                        Quat::from_rotation_x(FRAC_PI_2 + pitch),
                    );
                }
            }
        });

    query((player(), player_movement_direction(), player_yaw())).each_frame(move |players| {
        for (e, (_, direction, yaw)) in players {
            let speed = 0.1;
            let direction = Mat2::from_angle(yaw) * direction;
            entity::mutate_component(e, map_position(), |pos| *pos += direction * speed);
        }
    });
}

use std::f32::consts::{FRAC_PI_2, TAU};

use ambient_api::prelude::*;

use crate::{
    components::{map_position, player::*},
    messages,
};

pub fn init_players() {
    spawn_query(player()).bind(move |players| {
        for (e, _) in players {
            entity::add_components(
                e,
                Entity::new()
                    .with(map_position(), vec2(16.0, 16.0))
                    .with(yaw(), 0.0)
                    .with(pitch(), 0.0),
            );
        }
    });

    messages::PlayerMovementInput::subscribe(move |source, msg| {
        let Some(id) = source.client_entity_id() else { return; };

        let direction = msg.direction.normalize_or_zero();
        let new_yaw = msg.yaw % TAU;
        let new_pitch = msg.pitch.clamp(-FRAC_PI_2, FRAC_PI_2);

        entity::add_components(
            id,
            Entity::new()
                .with(movement_direction(), direction)
                .with(yaw(), new_yaw)
                .with(pitch(), new_pitch),
        );
    });

    query((player(), movement_direction(), yaw())).each_frame(move |players| {
        for (e, (_, direction, yaw)) in players {
            let speed = 0.1;
            let direction = Mat2::from_angle(yaw) * direction;
            entity::mutate_component(e, map_position(), |pos| *pos += direction * speed);
        }
    });
}

use std::f32::consts::{FRAC_PI_2, TAU};

use ambient_api::prelude::*;

use crate::{
    components::{map_position, player::*},
    messages,
};

pub fn init_players() {
    spawn_query((player(), user_id())).bind(move |players| {
        for (player_entity, (_, user)) in players {
            let make_hand = |held| {
                Entity::new()
                    .with(user_id(), user.clone())
                    .with(held_item_ref(), held)
                    .spawn()
            };

            let left_hand = make_hand(*crate::items::BLUE_ITEM);
            let right_hand = make_hand(*crate::items::YELLOW_ITEM);

            entity::add_components(
                player_entity,
                Entity::new()
                    .with_default(loaded())
                    .with(left_hand_ref(), left_hand)
                    .with(right_hand_ref(), right_hand)
                    .with(map_position(), vec2(16.0, 16.0))
                    .with(yaw(), 0.0)
                    .with(pitch(), 0.0),
            );
        }
    });

    despawn_query((player(), left_hand_ref(), right_hand_ref())).bind(move |players| {
        for (_player_entity, (_, left_hand, right_hand)) in players {
            entity::despawn(left_hand);
            entity::despawn(right_hand);
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

    query((player(), loaded(), movement_direction(), yaw())).each_frame(move |players| {
        for (e, (_, _, direction, yaw)) in players {
            let speed = 0.1;
            let direction = Mat2::from_angle(yaw) * direction;
            entity::mutate_component(e, map_position(), |pos| *pos += direction * speed);
        }
    });
}

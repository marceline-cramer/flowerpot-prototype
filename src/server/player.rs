use std::f32::consts::{FRAC_PI_2, TAU};

use ambient_api::{message::Source, prelude::*};

use crate::{
    components::{map, player::*},
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
                    .with(map::position(), vec2(16.0, 16.0))
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
            entity::mutate_component(e, map::position(), |pos| *pos += direction * speed);
        }
    });
}

/// Helper function to retrieve the entities that compose a player.
pub struct PlayerEntities {
    pub entity: EntityId,
    pub left_hand: EntityId,
    pub right_hand: EntityId,
    pub left_held: EntityId,
    pub right_held: EntityId,
}

impl PlayerEntities {
    pub fn from_source(source: &Source) -> Option<Self> {
        let entity = source.clone().client_entity_id()?;
        Self::from_entity(entity)
    }

    pub fn from_entity(entity: EntityId) -> Option<PlayerEntities> {
        let left_hand = entity::get_component(entity, left_hand_ref())?;
        let right_hand = entity::get_component(entity, right_hand_ref())?;
        let left_held = entity::get_component(left_hand, held_item_ref()).unwrap_or_default();
        let right_held = entity::get_component(right_hand, held_item_ref()).unwrap_or_default();

        Some(Self {
            entity,
            left_hand,
            right_hand,
            left_held,
            right_held,
        })
    }

    pub fn set_left_held(&mut self, item: EntityId) {
        entity::add_component(self.left_hand, held_item_ref(), item);
        self.left_held = item;
    }

    pub fn set_right_held(&mut self, item: EntityId) {
        entity::add_component(self.right_hand, held_item_ref(), item);
        self.right_held = item;
    }
}

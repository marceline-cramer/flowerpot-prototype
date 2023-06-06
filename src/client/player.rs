use std::f32::consts::FRAC_PI_2;

use ambient_api::{
    components::core::{camera::aspect_ratio_from_window, primitives::cube},
    concepts::{make_perspective_infinite_reverse_camera, make_sphere, make_transformable},
    prelude::*,
};

use crate::components::player::*;

pub fn init_players() {
    on_player_spawn(|player_entity, user, is_local_player| {
        if !is_local_player {
            // TODO player models for other players
            entity::add_components(
                player_entity,
                Entity::new()
                    .with_merge(make_transformable())
                    .with_merge(make_sphere())
                    .with(color(), vec4(1.0, 0.0, 0.0, 1.0)),
            );

            return;
        }

        let head = Entity::new()
            .with_merge(make_perspective_infinite_reverse_camera())
            .with(aspect_ratio_from_window(), EntityId::resources())
            .with_default(main_scene())
            .with(user_id(), user.clone())
            .with(translation(), Vec3::Z * 1.5)
            .with_default(local_to_parent())
            .with(rotation(), Quat::from_rotation_x(FRAC_PI_2))
            .spawn();

        entity::add_child(player_entity, head);

        let init_hand = |hand_ref, offset| {
            let e = entity::get_component(player_entity, hand_ref)
                .expect("Loaded player entity does not have hand reference component");

            entity::add_components(
                e,
                Entity::new()
                    .with_default(main_scene())
                    .with_default(local_to_parent())
                    .with_default(local_to_world())
                    .with(translation(), offset)
                    .with(rotation(), Quat::IDENTITY)
                    .with(scale(), Vec3::splat(0.1)),
            );

            entity::add_child(head, e);
        };

        init_hand(left_hand_ref(), Vec3::new(-0.5, -0.4, 1.0));
        init_hand(right_hand_ref(), Vec3::new(0.5, -0.4, 1.0));

        entity::add_components(
            player_entity,
            Entity::new()
                .with_merge(make_transformable())
                .with_default(cube())
                .with(head_ref(), head),
        );
    });

    change_query((player(), yaw(), pitch()))
        .track_change((yaw(), pitch()))
        .bind(move |players| {
            for (e, (_player, yaw, pitch)) in players {
                entity::add_component(e, rotation(), Quat::from_rotation_z(yaw));
                if let Some(head) = entity::get_component(e, head_ref()) {
                    entity::add_component(
                        head,
                        rotation(),
                        Quat::from_rotation_x(FRAC_PI_2 + pitch),
                    );
                }
            }
        });
}

/// Helper function to run a closure when player entities finish loading.
///
/// The closure takes the entity ID of the new player, the user ID, and
/// whether the player is the local player.
pub fn on_player_spawn(cb: impl Fn(EntityId, String, bool) + 'static) {
    spawn_query((player(), loaded(), user_id())).bind(move |players| {
        let local_uid = entity::get_component(entity::resources(), local_user_id()).unwrap();
        for (player_entity, (_, _, user)) in players {
            let is_local_player = user == local_uid;
            cb(player_entity, user, is_local_player);
        }
    });
}

lazy_static::lazy_static! {
    pub static ref LOCAL_PLAYER_ENTITY: EntityId = {
        let (e_tx, e_rx) = std::sync::mpsc::sync_channel(0);

        on_player_spawn(move |e, _user, is_local| {
            if is_local {
                let _ = e_tx.send(e);
            }
        });

        e_rx.recv().unwrap()
    };
}

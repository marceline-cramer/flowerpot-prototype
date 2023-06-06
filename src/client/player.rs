use std::f32::consts::FRAC_PI_2;

use ambient_api::{
    components::core::{
        camera::aspect_ratio_from_window,
        primitives::{cube, sphere_radius},
    },
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
            .with(parent(), player_entity)
            .with_default(local_to_parent())
            .with(rotation(), Quat::from_rotation_x(FRAC_PI_2))
            .spawn();

        let make_hand = |offset| {
            Entity::new()
                .with_default(main_scene())
                .with(user_id(), user.clone())
                .with(parent(), head)
                .with_default(local_to_parent())
                .with(translation(), offset)
                .with(rotation(), Quat::IDENTITY)
                .with(scale(), Vec3::splat(0.1))
                .with(held_item_ref(), EntityId::null())
                .with_merge(make_sphere())
                .with(sphere_radius(), 0.01)
                .spawn()
        };

        let left_hand = make_hand(Vec3::new(-0.5, -0.4, 1.0));
        let right_hand = make_hand(Vec3::new(0.5, -0.4, 1.0));

        entity::add_component(head, children(), vec![left_hand, right_hand]);
        entity::add_component(head, left_hand_ref(), left_hand);
        entity::add_component(head, right_hand_ref(), right_hand);

        entity::add_components(
            player_entity,
            Entity::new()
                .with_merge(make_transformable())
                .with_default(cube())
                .with(children(), vec![head])
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

/// Helper function to run a closure when player entities spawns.
///
/// The closure takes the entity ID of the new player, the user ID, and
/// whether the player is the local player.
pub fn on_player_spawn(cb: impl Fn(EntityId, String, bool) + 'static) {
    spawn_query((player(), user_id())).bind(move |players| {
        let local_uid = entity::get_component(entity::resources(), local_user_id()).unwrap();
        for (player_entity, (_, user)) in players {
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

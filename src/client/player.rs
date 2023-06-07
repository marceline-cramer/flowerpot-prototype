use std::f32::consts::{FRAC_PI_2, TAU};

use ambient_api::{
    components::core::{
        camera::aspect_ratio_from_window, prefab::prefab_from_url, primitives::cube,
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    input::get_previous,
    messages::Frame,
    prelude::*,
};

use crate::{
    components::player::*,
    messages::{PlayerCraftInput, PlayerMovementInput, PlayerSwapItemsInput},
};

/// Initializes player-related systems. Returns the local player entity ID.
pub async fn init_players() -> EntityId {
    on_player_spawn(|player_entity, user, is_local_player| {
        let init_hand = |parent, hand_ref, offset| {
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

            entity::add_child(parent, e);
        };

        if is_local_player {
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

            init_hand(head, left_hand_ref(), Vec3::new(-0.5, -0.4, 1.0));
            init_hand(head, right_hand_ref(), Vec3::new(0.5, -0.4, 1.0));

            entity::add_components(
                player_entity,
                Entity::new()
                    .with_merge(make_transformable())
                    .with_default(local_player())
                    .with_default(cube())
                    .with(head_ref(), head),
            );

            entity::add_component(entity::resources(), local_player_ref(), player_entity);
        } else {
            // hand offsets eyeballed to line up with temp player model hands
            init_hand(
                player_entity,
                left_hand_ref(),
                Vec3::new(-0.648, 0.0, 0.945),
            );

            init_hand(
                player_entity,
                right_hand_ref(),
                Vec3::new(0.648, 0.0, 0.945),
            );

            entity::add_components(
                player_entity,
                Entity::new()
                    .with_merge(make_transformable())
                    .with(
                        prefab_from_url(),
                        asset::url("assets/player/player.glb").unwrap(),
                    )
                    .with(color(), vec4(1.0, 0.0, 0.0, 1.0)),
            );
        }
    });

    change_query((player(), yaw(), pitch()))
        .track_change((yaw(), pitch()))
        .excludes(local_player())
        .bind(move |players| {
            for (e, (_player, yaw, pitch)) in players {
                update_player_yaw_pitch(e, yaw, pitch);
            }
        });

    change_query((player(), local_player(), local_yaw(), local_pitch()))
        .track_change((local_yaw(), local_pitch()))
        .bind(move |players| {
            for (e, (_, _, yaw, pitch)) in players {
                update_player_yaw_pitch(e, yaw, pitch);
            }
        });

    let local_player_entity = entity::wait_for_component(entity::resources(), local_player_ref())
        .await
        .expect("local_player_ref resource was deleted");

    Frame::subscribe({
        let mut cursor_lock = input::CursorLockGuard::new(true);
        let mut pitch = 0.0;
        let mut yaw = 0.0;
        move |_| {
            let input = input::get();
            if !cursor_lock.auto_unlock_on_escape(&input) {
                return;
            }

            let mut direction = Vec2::ZERO;
            let speed = 1.0; // always 1.0 because PlayerMovementInput is normalized
            if input.keys.contains(&KeyCode::W) {
                direction.y -= speed;
            }
            if input.keys.contains(&KeyCode::S) {
                direction.y += speed;
            }
            if input.keys.contains(&KeyCode::A) {
                direction.x -= speed;
            }
            if input.keys.contains(&KeyCode::D) {
                direction.x += speed;
            }

            let direction = direction.normalize();

            let pitch_factor = 0.01;
            let yaw_factor = 0.01;
            yaw = (yaw + input.mouse_delta.x * yaw_factor) % TAU;
            pitch = (pitch + input.mouse_delta.y * pitch_factor).clamp(-FRAC_PI_2, FRAC_PI_2);

            PlayerMovementInput::new(direction, pitch, yaw).send_server_reliable();

            entity::add_component(local_player_entity, local_yaw(), yaw);
            entity::add_component(local_player_entity, local_pitch(), pitch);

            let last_input = get_previous();
            let input_delta = input.delta(&last_input);

            if input_delta.keys.contains(&KeyCode::Q) {
                PlayerCraftInput::new().send_server_reliable();
            }

            if input_delta.keys.contains(&KeyCode::F) {
                PlayerSwapItemsInput::new().send_server_reliable();
            }
        }
    });

    local_player_entity
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

/// Helper function to update the yaw of a player and optionally its head's pitch.
pub fn update_player_yaw_pitch(e: EntityId, yaw: f32, pitch: f32) {
    entity::add_component(e, rotation(), Quat::from_rotation_z(yaw));
    if let Some(head) = entity::get_component(e, head_ref()) {
        entity::add_component(head, rotation(), Quat::from_rotation_x(FRAC_PI_2 + pitch));
    }
}

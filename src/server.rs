use std::{
    collections::HashMap,
    f32::consts::{FRAC_PI_2, TAU},
    sync::Arc,
};

use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        prefab::prefab_from_url,
        primitives::{cube, quad},
        transform::translation,
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    glam::IVec2,
    prelude::*,
    rand,
};

use components::*;

mod partitioning;

const SOIL_COLOR: Vec4 = Vec4::new(0.13, 0.07, 0.05, 1.0);
const GRASS_COLOR: Vec4 = Vec4::new(0.39, 0.67, 0.2, 1.0);

pub enum OrdinalDirection {
    West,
    North,
    East,
    South,
}

impl OrdinalDirection {
    pub fn closest_to_vec2(v: Vec2) -> Self {
        use OrdinalDirection::*;
        if v.x.abs() > v.y.abs() {
            if v.x > 0.0 {
                East
            } else {
                West
            }
        } else {
            if v.y > 0.0 {
                South
            } else {
                North
            }
        }
    }

    pub fn as_neighbor_component(&self) -> Component<EntityId> {
        use OrdinalDirection::*;
        match self {
            West => west_neighbor(),
            North => north_neighbor(),
            East => east_neighbor(),
            South => south_neighbor(),
        }
    }

    pub fn get_tile_neighbor(&self, tile: EntityId) -> Option<EntityId> {
        let neighbor = self.as_neighbor_component();
        entity::get_component(tile, neighbor)
    }
}

fn for_random_neighbors<T>(
    rng: &mut impl Rng,
    tile: EntityId,
    mut cb: impl FnMut(EntityId) -> Option<T>,
) -> Option<T> {
    use OrdinalDirection::*;
    let mut directions = [West, North, East, South];
    directions.shuffle(rng);

    for dir in directions {
        if let Some(neighbor) = dir.get_tile_neighbor(tile) {
            if let Some(t) = cb(neighbor) {
                return Some(t);
            }
        }
    }

    None
}

#[main]
pub fn main() {
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

    // create a grass cover crop prototype
    let grass = Entity::new()
        .with_default(cover_crop())
        .with(sustenance(), 2.0)
        .with(color(), GRASS_COLOR)
        .spawn();

    // spawn some initial tiles and store their IDs
    let mut rng = rand::thread_rng();
    let mut map = HashMap::new();
    let map_width = 32;
    let map_height = 32;
    for x in 0..map_width {
        for y in 0..map_height {
            let xy = IVec2::new(x, y);

            let tile = Entity::new()
                .with_merge(make_transformable())
                .with(translation(), Vec3::new(x as f32, y as f32, 0.0))
                .with_default(quad())
                .with_default(tile())
                .with_default(soil())
                .with(cover_crop_occupant(), grass)
                .with(map_position(), xy.as_vec2())
                .spawn();

            map.insert(xy, tile);
        }
    }

    let map = Arc::new(map);

    // connect each tile's neighbor
    for (xy, e) in map.iter() {
        let xy = *xy;
        let e = *e;

        if let Some(neighbor) = map.get(&(xy - IVec2::X)) {
            entity::add_component(e, west_neighbor(), *neighbor);
        }

        if let Some(neighbor) = map.get(&(xy - IVec2::Y)) {
            entity::add_component(e, north_neighbor(), *neighbor);
        }

        if let Some(neighbor) = map.get(&(xy + IVec2::X)) {
            entity::add_component(e, east_neighbor(), *neighbor);
        }

        if let Some(neighbor) = map.get(&(xy + IVec2::Y)) {
            entity::add_component(e, south_neighbor(), *neighbor);
        }
    }

    // init cover crop searching
    partitioning::init_qbvh(
        cover_crop_occupant(),
        search_cover_crop_radius(),
        search_cover_crop_result(),
    );

    // color soil tiles correctly
    spawn_query((tile(), soil()))
        .excludes(cover_crop_occupant())
        .bind(move |tiles| {
            for (e, (_, _)) in tiles {
                entity::add_component(e, color(), SOIL_COLOR);
            }
        });

    // color tiles with cover crops correctly
    change_query((tile(), cover_crop_occupant()))
        .track_change(cover_crop_occupant())
        .bind(move |tiles| {
            for (e, (_, cover_crop)) in tiles {
                if let Some(new_color) = entity::get_component(cover_crop, color()) {
                    entity::add_component(e, color(), new_color);
                }
            }
        });

    // decrease fullness by hunger rate
    query((fullness(), hunger_rate())).each_frame(|entities| {
        for (e, (old_fullness, hunger_rate)) in entities {
            let fullness_delta = hunger_rate * frametime();
            let new_fullness = old_fullness - fullness_delta;
            entity::set_component(e, fullness(), new_fullness);
        }
    });

    // kill entities with non-positive fullness
    change_query(fullness())
        .track_change(fullness())
        .bind(|changed| {
            for (e, fullness) in changed {
                if fullness <= 0.0 {
                    entity::despawn(e);
                }
            }
        });

    // passive metabolism refills entity stamina
    query((stamina(), passive_metabolism())).each_frame(|entities| {
        for (e, (old_stamina, metabolism)) in entities {
            let new_stamina = old_stamina + metabolism * frametime();
            entity::set_component(e, stamina(), new_stamina);
        }
    });

    // update entities' translation and OnTile with map coordinates
    change_query(map_position())
        .track_change(map_position())
        .bind({
            let map = map.clone();
            move |changes| {
                for (e, xy) in changes {
                    let elevation = entity::get_component(e, map_elevation()).unwrap_or(0.0);
                    entity::add_component(e, translation(), xy.extend(elevation));

                    let xy = (xy + 0.5).floor().as_ivec2();
                    match map.get(&xy) {
                        None => entity::remove_component(e, on_tile()),
                        Some(tile) => entity::add_component(e, on_tile(), *tile),
                    }
                }
            }
        });

    // anchor entities on tiles but without map positions to their tiles
    change_query(on_tile())
        .track_change(on_tile())
        .excludes(map_position())
        .bind(|changes| {
            for (e, tile) in changes {
                let new_translation = entity::get_component(tile, translation()).unwrap();
                entity::add_component(e, translation(), new_translation);
            }
        });

    // step moving entities
    query((
        movement_step(),
        movement_duration(),
        movement_start(),
        movement_target(),
    ))
    .each_frame(|entities| {
        for (e, (step, duration, start, target)) in entities {
            let new_step = step + frametime();
            if new_step > duration {
                entity::remove_component(e, movement_step());
                entity::add_component(e, map_position(), target);
            } else {
                let delta = new_step / duration;
                let new_pos = start * (1.0 - delta) + target * delta;
                entity::set_component(e, movement_step(), new_step);
                entity::add_component(e, map_position(), new_pos);
            }
        }
    });

    // set elevation of entities with movement height
    change_query((movement_step(), movement_duration(), movement_height()))
        .track_change(movement_step())
        .bind(|changes| {
            for (e, (step, duration, height)) in changes {
                let delta = step / duration;
                let height_factor = delta * (1.0 - delta) * 4.0;
                let elevation = height * height_factor;
                entity::add_component(e, map_elevation(), elevation);
            }
        });

    // spawn some bunnies
    for tile in map
        .values()
        .collect::<Vec<_>>()
        .partial_shuffle(&mut rng, 5)
        .0
        .to_vec()
    {
        Entity::new()
            .with_merge(make_transformable())
            .with(
                prefab_from_url(),
                asset::url("assets/fauna/rabbit.glb").unwrap(),
            )
            .with_default(fauna())
            .with_default(bunny())
            .with(stamina(), 0.0)
            .with(passive_metabolism(), 1.0)
            .with(movement_cost(), rng.gen_range(0.4..0.6))
            .with(movement_distance(), 0.5)
            .with(search_cover_crop_radius(), 10.0)
            .with(
                map_position(),
                entity::get_component(*tile, map_position()).unwrap(),
            )
            .with(fullness(), 1.0)
            .with(hunger_rate(), 0.1)
            .with(sustenance(), 10.0)
            .spawn();
    }

    // move fauna
    query((
        fauna(),
        map_position(),
        on_tile(),
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

            let target_pos = match entity::get_component(search_result, map_position()) {
                Some(target_pos) => target_pos,
                None => continue,
            };

            let target_delta = target_pos - map_pos;
            let movement_delta = target_delta.clamp_length_max(movement_distance);
            let movement_theta = -movement_delta.angle_between(Vec2::Y);

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

    messages::GrowTick::subscribe({
        let growable_query = query((tile(), cover_crop_occupant())).build();
        let mut rng = rand::thread_rng();
        move |_, _| {
            for (tile, (_, cover_crop)) in growable_query.evaluate() {
                for_random_neighbors(&mut rng, tile, |neighbor| {
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

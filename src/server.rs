use std::{collections::HashMap, sync::Arc};

use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        prefab::prefab_from_url,
        primitives::{cube, quad},
        transform::{lookat_target, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    glam::{ivec2, IVec2},
    prelude::*,
    rand,
};

use components::*;

mod partitioning;

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

// TODO make this a concept (?)
fn spawn_grass(tile: EntityId) -> EntityId {
    let grass = Entity::new()
        .with_merge(make_transformable())
        .with_default(small_crop())
        .with_default(grass())
        .with(sustenance(), 0.1)
        .with_default(cube())
        .with(scale(), Vec3::splat(0.25))
        .with(color(), Vec4::new(0.0, 1.0, 0.0, 1.0))
        .with(
            map_position(),
            entity::get_component(tile, map_position()).unwrap(),
        )
        .spawn();

    entity::add_component(tile, small_crop_occupant(), grass);

    grass
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
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), Vec3::new(-5.0, -5.0, 12.0))
        .with(lookat_target(), vec3(16., 16., 0.))
        .spawn();

    // spawn a ground plane
    let map_width = 32;
    let map_height = 32;
    let map_dims = ivec2(map_width, map_height).as_vec2();
    Entity::new()
        .with_merge(make_transformable())
        .with(translation(), (map_dims / 2.0).extend(0.0))
        .with(scale(), (map_dims + 1.0).extend(1.0))
        .with_default(quad())
        .with(color(), Vec4::new(0.2, 1.0, 0.0, 1.0))
        .spawn();

    // spawn some initial tiles and store their IDs
    let mut rng = rand::thread_rng();
    let mut map = HashMap::new();
    for x in 0..map_width {
        for y in 0..map_height {
            let xy = IVec2::new(x, y);

            let tile = Entity::new()
                .with_merge(make_transformable())
                .with(translation(), Vec3::new(x as f32, y as f32, 0.0))
                .with_default(tile())
                .with_default(soil())
                .with(map_position(), xy.as_vec2())
                .spawn();

            spawn_grass(tile);
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

    // init small crop searching
    partitioning::init_qbvh(
        small_crop(),
        search_small_crop_radius(),
        search_small_crop_result(),
    );

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

    // when bunnies move they consume small crops and restore hunger
    change_query((bunny(), on_tile(), fullness()))
        .track_change(on_tile())
        .bind(|changes| {
            for (e, (_bunny, tile, old_fullness)) in changes {
                if let Some(small_crop) = entity::get_component(tile, small_crop_occupant()) {
                    if let Some(sustenance) = entity::get_component(small_crop, sustenance()) {
                        let new_fullness = old_fullness + sustenance;
                        entity::set_component(e, fullness(), new_fullness);
                        entity::despawn(small_crop);
                        entity::remove_component(tile, small_crop_occupant());
                    }
                }
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
            .with(prefab_from_url(), asset::url("assets/fauna/rabbit.glb").unwrap())
            .with_default(fauna())
            .with_default(bunny())
            .with(stamina(), 0.0)
            .with(passive_metabolism(), 1.0)
            .with(movement_cost(), rng.gen_range(0.4..0.6))
            .with(movement_distance(), 0.5)
            .with(search_small_crop_radius(), 10.0)
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
        stamina(),
        movement_cost(),
        movement_distance(),
        search_small_crop_result(),
    ))
    .excludes(movement_step())
    .each_frame(|entities| {
        for (e, (_fauna, map_pos, old_stamina, movement_cost, movement_distance, search_result)) in
            entities
        {
            if old_stamina < movement_cost {
                continue;
            }

            entity::remove_component(e, search_small_crop_result());

            if search_result.is_null() {
                continue;
            }

            let target_pos = match entity::get_component(search_result, map_position()) {
                Some(target_pos) => target_pos,
                None => continue,
            };

            let target_delta = target_pos - map_pos;
            let movement_delta = target_delta.clamp_length(0.0, movement_distance);
            let movement_theta = -movement_delta.angle_between(Vec2::Y);
            let new_stamina = old_stamina - movement_cost;

            let components = Entity::new()
                .with(rotation(), Quat::from_rotation_z(movement_theta))
                .with(movement_step(), 0.0)
                .with(movement_duration(), 0.25)
                .with(movement_start(), map_pos)
                .with(movement_target(), map_pos + movement_delta)
                .with(movement_height(), 0.5)
                .with(stamina(), new_stamina);

            entity::add_components(e, components);
        }
    });

    let grass_grow = query((grass(), on_tile())).build();
    let mut rng = rand::thread_rng();
    messages::GrowTick::subscribe(move |_, _| {
        for (_e, (_grass, tile)) in grass_grow.evaluate() {
            for_random_neighbors(&mut rng, tile, |neighbor| {
                if entity::has_component(neighbor, small_crop_occupant()) {
                    None
                } else {
                    spawn_grass(neighbor);
                    Some(())
                }
            });
        }
    });

    run_async(async move {
        loop {
            sleep(10.0).await;
            messages::GrowTick::new().send_local_broadcast(true);
        }
    });
}

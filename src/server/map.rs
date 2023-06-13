use std::{collections::HashMap, sync::Arc};

use ambient_api::{
    components::core::prefab::prefab_from_url, concepts::make_transformable, glam::IVec2,
    prelude::*, rand,
};

use crate::components::*;

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
            West => map::west_neighbor_ref(),
            North => map::north_neighbor_ref(),
            East => map::east_neighbor_ref(),
            South => map::south_neighbor_ref(),
        }
    }

    pub fn get_tile_neighbor(&self, tile: EntityId) -> Option<EntityId> {
        let neighbor = self.as_neighbor_component();
        entity::get_component(tile, neighbor)
    }
}

pub(crate) fn for_random_neighbors<T>(
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

/// Sets up map-related queries and spawns the map.
pub fn init_map() {
    // create a grass cover crop prototype
    let grass = Entity::new()
        .with_default(cover_crop())
        .with(sustenance(), 2.0)
        .with(
            pbr_material_from_url(),
            asset::url("assets/materials/materials/pipeline.json/1/mat.json").unwrap(),
        )
        .spawn();

    // spawn some initial tiles and store their IDs
    let mut map = HashMap::new();
    let map_width = 32;
    let map_height = 32;
    for x in 0..map_width {
        for y in 0..map_height {
            let xy = IVec2::new(x, y);

            let tile = Entity::new()
                .with_default(map::tile())
                .with_default(map::soil())
                .with(cover_crop_occupant(), grass)
                .with(map::position(), xy.as_vec2())
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
            entity::add_component(e, map::west_neighbor_ref(), *neighbor);
        }

        if let Some(neighbor) = map.get(&(xy - IVec2::Y)) {
            entity::add_component(e, map::north_neighbor_ref(), *neighbor);
        }

        if let Some(neighbor) = map.get(&(xy + IVec2::X)) {
            entity::add_component(e, map::east_neighbor_ref(), *neighbor);
        }

        if let Some(neighbor) = map.get(&(xy + IVec2::Y)) {
            entity::add_component(e, map::south_neighbor_ref(), *neighbor);
        }
    }

    // set elevation of entities with movement height
    change_query((movement_step(), movement_duration(), movement_height()))
        .track_change(movement_step())
        .bind(|changes| {
            for (e, (step, duration, height)) in changes {
                let delta = step / duration;
                let height_factor = delta * (1.0 - delta) * 4.0;
                let elevation = height * height_factor;
                entity::add_component(e, map::elevation(), elevation);
            }
        });

    // update entities' on_tile based on map_position
    change_query(map::position())
        .track_change(map::position())
        .bind({
            let map = map.clone();
            move |changes| {
                for (e, xy) in changes {
                    let xy = (xy + 0.5).floor().as_ivec2();
                    match map.get(&xy) {
                        None => entity::remove_component(e, map::on_tile()),
                        Some(tile) => entity::add_component(e, map::on_tile(), *tile),
                    }
                }
            }
        });

    // spawn some items on the ground
    {
        use crate::data::*;

        Entity::new()
            .with(map::position(), vec2(10.0, 15.0))
            .with(items::class_ref(), *BLUE_ITEM)
            .spawn();

        Entity::new()
            .with(map::position(), vec2(17.0, 13.0))
            .with(items::class_ref(), *YELLOW_ITEM)
            .spawn();

        Entity::new()
            .with(map::position(), vec2(10.0, 13.0))
            .with(items::class_ref(), *items::MAIZE_SEEDS)
            .spawn();

        let maize_tile = *map.get(&IVec2::new(0, 0)).unwrap();

        entity::add_component(
            maize_tile,
            crate::components::crops::medium_occupant_ref(),
            crate::crop::new_medium(*crops::MAIZE_STAGE_1, maize_tile),
        );
    }

    // TODO make bunnies fun to code and important to gameplay
    return;

    // spawn some bunnies
    let mut rng = rand::thread_rng();
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
                map::position(),
                entity::get_component(*tile, map::position()).unwrap(),
            )
            .with(fullness(), 1.0)
            .with(hunger_rate(), 0.1)
            .with(sustenance(), 10.0)
            .spawn();
    }
}

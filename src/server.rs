use std::collections::HashMap;

use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        primitives::{cube, quad},
        transform::{lookat_target, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    glam::IVec2,
    prelude::*,
    rand,
};

use components::*;

enum OrdinalDirection {
    West,
    North,
    East,
    South,
}

// TODO make this a concept (?)
fn spawn_grass(tile: EntityId) -> EntityId {
    let mut rng = thread_rng();

    let grass = Entity::new()
        .with_merge(make_transformable())
        .with_default(small_crop())
        .with_default(grass())
        .with(sustenance(), 0.1)
        .with(stamina(), 0.0)
        .with(passive_metabolism(), rng.gen_range(0.09..0.11))
        .with(movement_cost(), 1.0)
        .with_default(cube())
        .with(scale(), Vec3::splat(0.25))
        .with(color(), Vec4::new(0.0, 1.0, 0.0, 1.0))
        .with(on_tile(), tile)
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
        let neighbor = match dir {
            West => west_neighbor(),
            North => north_neighbor(),
            East => east_neighbor(),
            South => south_neighbor(),
        };

        if let Some(neighbor) = entity::get_component(tile, neighbor) {
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

    // decrease fullness by hunger rate
    query((fullness(), hunger_rate())).each_frame(|entities| {
        for (e, (old_fullness, hunger_rate)) in entities {
            let fullness_delta = hunger_rate * frametime();
            let new_fullness = old_fullness - fullness_delta;
            entity::set_component(e, fullness(), new_fullness);
        }
    });

    // kill entities with non-positive fullness
    change_query((fullness(),))
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

    // remove despawned fauna on tiles from their tiles
    despawn_query((fauna(), on_tile())).bind(|despawned| {
        for (_e, (_fauna, tile)) in despawned {
            entity::remove_component(tile, fauna_occupant());
        }
    });

    // when fertility changes, update the tile's color
    change_query((soil(), color(), fertility()))
        .track_change(fertility())
        .bind(|changes| {
            for (e, (_soil, _color, new_fertility)) in changes.iter() {
                let new_color = Vec4::new(0.2, *new_fertility, 0., 1.);
                entity::set_component(*e, color(), new_color);
            }
        });

    // reposition changed tile occupants to their tile
    change_query((translation(), on_tile()))
        .track_change(on_tile())
        .bind(|changes| {
            for (e, (_translation, tile)) in changes {
                if let Some(pos) = entity::get_component(tile, translation()) {
                    entity::add_component(e, translation(), pos);
                }
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

    // spawn some initial tiles and store their IDs
    let mut rng = rand::thread_rng();
    let mut map = HashMap::new();
    for x in 0..32 {
        for y in 0..32 {
            let xy = IVec2::new(x, y);

            let tile = Entity::new()
                .with_merge(make_transformable())
                .with(translation(), Vec3::new(x as f32, y as f32, 0.0))
                .with_default(quad())
                .with_default(tile())
                .with_default(soil())
                .with(fertility(), rng.gen_range(0.0..1.0))
                .with(color(), Vec4::new(1.0, 0.0, 1.0, 1.0))
                .spawn();

            spawn_grass(tile);
            map.insert(xy, tile);
        }
    }

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

    // spawn some bunnies
    for tile in map
        .values()
        .collect::<Vec<_>>()
        .partial_shuffle(&mut rng, 100)
        .0
        .to_vec()
    {
        let fauna = Entity::new()
            .with_merge(make_transformable())
            .with(scale(), Vec3::splat(0.5))
            .with_default(cube())
            .with(color(), Vec4::new(0.2, 0.3, 0.7, 1.0))
            .with_default(fauna())
            .with_default(bunny())
            .with(stamina(), 0.0)
            .with(passive_metabolism(), 1.0)
            .with(movement_cost(), rng.gen_range(0.4..0.6))
            .with(on_tile(), *tile)
            .with(fullness(), 1.0)
            .with(hunger_rate(), 0.1)
            .spawn();

        // TODO automatically set occupants with query events
        entity::add_component(*tile, fauna_occupant(), fauna);
    }

    // move fauna
    query((fauna(), on_tile(), stamina(), movement_cost())).each_frame(|entities| {
        let mut rng = rand::thread_rng();
        for (e, (_fauna, tile, old_stamina, movement_cost)) in entities {
            if old_stamina < movement_cost {
                continue;
            }

            let moved = for_random_neighbors(&mut rng, tile, |neighbor| {
                if entity::has_component(neighbor, fauna_occupant()) {
                    None
                } else {
                    entity::add_component(neighbor, fauna_occupant(), e);
                    entity::remove_component(tile, fauna_occupant());
                    entity::set_component(e, on_tile(), neighbor);
                    Some(())
                }
            })
            .is_some();

            if moved {
                let new_stamina = old_stamina - movement_cost;
                entity::set_component(e, stamina(), new_stamina);
            }
        }
    });

    // reproduce grass
    query((grass(), on_tile(), stamina(), movement_cost())).each_frame(|entities| {
        let mut rng = rand::thread_rng();
        for (e, (_grass, tile, old_stamina, movement_cost)) in entities {
            if old_stamina < movement_cost {
                continue;
            }

            let moved = for_random_neighbors(&mut rng, tile, |neighbor| {
                if entity::has_component(neighbor, small_crop_occupant()) {
                    None
                } else {
                    spawn_grass(neighbor);
                    Some(())
                }
            })
            .is_some();

            if moved {
                let new_stamina = old_stamina - movement_cost;
                entity::set_component(e, stamina(), new_stamina);
            }
        }
    });

    println!("Hello, Ambient!");
}

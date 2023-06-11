use ambient_api::{concepts::make_transformable, prelude::*};

use components::{map::position, *};

mod crop;
mod data;
mod fauna;
mod items;
mod map;
mod player;

#[path = "../shared/mod.rs"]
mod shared;

#[main]
pub fn main() {
    make_transformable()
        .with_default(sun())
        .with(rotation(), Quat::from_rotation_y(-45_f32.to_radians()))
        .with(light_diffuse(), Vec3::ONE)
        .with_default(main_scene())
        .spawn();

    make_transformable().with_default(sky()).spawn();

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
                entity::add_component(e, position(), target);
            } else {
                let delta = new_step / duration;
                let new_pos = start * (1.0 - delta) + target * delta;
                entity::set_component(e, movement_step(), new_step);
                entity::add_component(e, position(), new_pos);
            }
        }
    });

    crop::init_crops();
    data::init_data();
    fauna::init_fauna();
    items::init_server_items();
    map::init_map();
    player::init_players();
}

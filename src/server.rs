use ambient_api::prelude::*;

use components::*;

mod crop;
mod fauna;
mod map;
mod partitioning;
mod player;

#[main]
pub fn main() {
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

    crop::init_crops();
    fauna::init_fauna();
    map::init_map();
    player::init_players();
}

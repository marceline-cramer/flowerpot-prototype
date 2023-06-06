use std::f32::consts::{FRAC_PI_2, TAU};

use ambient_api::prelude::*;

mod items;
mod player;

#[main]
pub fn main() {
    items::init_items();
    player::init_players();

    let mut cursor_lock = input::CursorLockGuard::new(true);
    let mut pitch = 0.0;
    let mut yaw = 0.0;
    let mut craft_last = false;
    ambient_api::messages::Frame::subscribe(move |_| {
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

        messages::PlayerMovementInput::new(direction, pitch, yaw).send_server_reliable();

        let craft_pressed = input.keys.contains(&KeyCode::Q);
        if !craft_last && craft_pressed {
            messages::PlayerCraftInput::new().send_server_reliable();
        }

        craft_last = craft_pressed;
    });
}

mod items;
mod map;
mod player;

#[path = "../shared/mod.rs"]
mod shared;

#[ambient_api::main]
pub async fn main() {
    items::init_items();
    map::init_map();
    player::init_players().await;
}

mod crops;
mod items;
mod map;
mod player;

#[path = "../shared/mod.rs"]
mod shared;

#[ambient_api::main]
pub async fn main() {
    crops::init_crops();
    items::init_items();
    map::init_map();
    player::init_players().await;
}

use ambient_api::prelude::*;

macro_rules! expand_props {
    ($e:expr, $component:ident: $value:expr $(, $component_tail:ident: $value_tail:expr)* $(,)?) => {
        expand_props!(Entity::with($e, $component(), $value.into()) $(, $component_tail: $value_tail)*)
    };
    ($e:expr) => ($e);
}

macro_rules! spawn_entity {
    ($($component:ident: $value:expr),* $(,)?) => {
        expand_props!(Entity::new(), $($component: $value),*).spawn()
    }
}

macro_rules! def_entity {
    ($item_name:ident $(, $component:ident: $value:expr)* $(,)?) => {
        ::lazy_static::lazy_static! {
            pub static ref $item_name: EntityId = spawn_entity!($($component: $value),*);
        }
    }
}

def_entity!(
    BLUE_ITEM,
    name: "Blue Item",
    color: vec4(0.0, 0.0, 1.0, 1.0),
);

def_entity!(
    GREEN_ITEM,
    name: "Green Item",
    color: vec4(0.0, 1.0, 0.0, 1.0),
);

def_entity!(
    YELLOW_ITEM,
    name: "Yellow Item",
    color: vec4(1.0, 1.0, 0.0, 1.0),
);

pub mod items {
    use super::*;

    pub use crate::components::items::*;

    def_entity!(
        MAIZE_SEEDS,
        name: "Maize Seeds",
        prefab_path: "assets/crops/maize_seeds.glb",
    );
}

pub mod crops {
    use super::*;

    use crate::components::crops::*;

    def_entity!(
        MAIZE,
        prefab_url: "assets/crops/maize4.glb",
    );

    def_entity!(
        MAIZE_STAGE_3,
        prefab_url: "assets/crops/maize3.glb",
        next_growth_phase_ref: *MAIZE,
    );

    def_entity!(
        MAIZE_STAGE_2,
        prefab_url: "assets/crops/maize2.glb",
        next_growth_phase_ref: *MAIZE_STAGE_3,
    );

    def_entity!(
        MAIZE_STAGE_1,
        prefab_url: "assets/crops/maize1.glb",
        next_growth_phase_ref: *MAIZE_STAGE_2,
    );
}

pub fn init_data() {
    use crate::components::crafting::*;

    // TODO better cyclic data definitions
    entity::add_component(
        *crops::MAIZE,
        crate::components::crops::seed_ref(),
        *crops::MAIZE_STAGE_1,
    );

    spawn_entity!(
        recipe: (),
        primary_ingredient: *BLUE_ITEM,
        secondary_ingredient: *YELLOW_ITEM,
        primary_yield: *GREEN_ITEM,
        secondary_yield: EntityId::null(),
    );
}

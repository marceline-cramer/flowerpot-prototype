use std::sync::atomic::AtomicBool;

use ambient_api::{
    components::core::{app::name, rendering::color},
    prelude::*,
};
use once_cell::sync::OnceCell;

/// A single-instance, lazily-spawned entity for use with the Prototype pattern.
pub struct PrototypeEntity {
    entity: OnceCell<EntityId>,
    add_cb: Box<dyn Fn(EntityId) + Send + Sync + 'static>,
    added: AtomicBool,
}

impl PrototypeEntity {
    pub fn new(cb: impl Fn(EntityId) + Send + Sync + 'static) -> Self {
        Self {
            entity: OnceCell::new(),
            add_cb: Box::new(cb),
            added: AtomicBool::new(false),
        }
    }

    pub fn get(&self) -> EntityId {
        let e = *self.entity.get_or_init(|| Entity::new().spawn());

        if !self.added.swap(true, std::sync::atomic::Ordering::SeqCst) {
            (*self.add_cb)(e);
        }

        e
    }
}

macro_rules! expand_props {
    ($e:expr, $component:ident: $value:expr $(, $component_tail:ident: $value_tail:expr)* $(,)?) => {
        expand_props!(Entity::with($e, $component(), $value.into()) $(, $component_tail: $value_tail)*)
    };
    ($e:expr) => ($e);
}

macro_rules! def_entity {
    ($($component:ident: $value:expr),* $(,)?) => {
        expand_props!(Entity::new(), $($component: $value),*)
    }
}

macro_rules! def_prototype {
    ($item_name:ident $(, $component:ident: $value:expr)* $(,)?) => {
        ::lazy_static::lazy_static! {
            pub static ref $item_name: PrototypeEntity = PrototypeEntity::new(move |e| {
                entity::add_components(e, def_entity!($($component: $value),*));
            });
        }
    }
}

def_prototype!(
    BLUE_ITEM,
    name: "Blue Item",
    color: vec4(0.0, 0.0, 1.0, 1.0),
);

def_prototype!(
    GREEN_ITEM,
    name: "Green Item",
    color: vec4(0.0, 1.0, 0.0, 1.0),
);

def_prototype!(
    YELLOW_ITEM,
    name: "Yellow Item",
    color: vec4(1.0, 1.0, 0.0, 1.0),
);

pub mod items {
    use super::*;

    pub use crate::components::items::*;

    def_prototype!(
        MAIZE,
        name: "Maize",
        prefab_path: "assets/items/maize.glb",
    );

    def_prototype!(
        MAIZE_SEEDS,
        name: "Maize Seeds",
        prefab_path: "assets/items/maize_seeds.glb",
        plantable_crop_class_ref: super::crops::MAIZE_STAGE_1.get(),
    );

    def_prototype!(
        TOMATO_SEEDS,
        name: "Tomato Seeds",
        prefab_path: "assets/items/tomato_seeds.glb",
        plantable_crop_class_ref: super::crops::TOMATOES_STAGE_1.get(),
    );
}

pub mod crops {
    use super::*;

    use crate::components::crops::*;

    def_prototype!(
        MAIZE,
        prefab_url: "assets/crops/maize4.glb",
        seed_ref: MAIZE_STAGE_1.get(),
        harvest_item_class_ref: items::MAIZE.get(),
    );

    def_prototype!(
        MAIZE_STAGE_3,
        prefab_url: "assets/crops/maize3.glb",
        next_growth_phase_ref: MAIZE.get(),
        harvest_item_class_ref: items::MAIZE_SEEDS.get(),
    );

    def_prototype!(
        MAIZE_STAGE_2,
        prefab_url: "assets/crops/maize2.glb",
        next_growth_phase_ref: MAIZE_STAGE_3.get(),
        harvest_item_class_ref: items::MAIZE_SEEDS.get(),
    );

    def_prototype!(
        MAIZE_STAGE_1,
        prefab_url: "assets/crops/maize1.glb",
        next_growth_phase_ref: MAIZE_STAGE_2.get(),
        harvest_item_class_ref: items::MAIZE_SEEDS.get(),
    );

    def_prototype!(
        IRIS,
        prefab_url: "assets/crops/iris.glb",
        seed_ref: IRIS.get(),
    );

    def_prototype!(
        TOMATOES,
        prefab_url: "assets/crops/tomatoes5.glb",
        seed_ref: TOMATOES_STAGE_1.get(),
        harvest_item_class_ref: items::TOMATO_SEEDS.get(),
    );

    def_prototype!(
        TOMATOES_STAGE_4,
        prefab_url: "assets/crops/tomatoes4.glb",
        next_growth_phase_ref: TOMATOES.get(),
        harvest_item_class_ref: items::TOMATO_SEEDS.get(),
    );

    def_prototype!(
        TOMATOES_STAGE_3,
        prefab_url: "assets/crops/tomatoes3.glb",
        next_growth_phase_ref: TOMATOES_STAGE_4.get(),
        harvest_item_class_ref: items::TOMATO_SEEDS.get(),
    );

    def_prototype!(
        TOMATOES_STAGE_2,
        prefab_url: "assets/crops/tomatoes2.glb",
        next_growth_phase_ref: TOMATOES_STAGE_3.get(),
        harvest_item_class_ref: items::TOMATO_SEEDS.get(),
    );

    def_prototype!(
        TOMATOES_STAGE_1,
        prefab_url: "assets/crops/tomatoes1.glb",
        next_growth_phase_ref: TOMATOES_STAGE_2.get(),
        harvest_item_class_ref: items::TOMATO_SEEDS.get(),
    );
}

pub fn init_data() {
    use crate::components::crafting::*;
    use items::*;

    def_entity!(
        recipe: (),
        primary_ingredient: BLUE_ITEM.get(),
        secondary_ingredient: YELLOW_ITEM.get(),
        primary_yield: GREEN_ITEM.get(),
        secondary_yield: EntityId::null(),
    )
    .spawn();

    def_entity!(
        recipe: (),
        primary_ingredient: MAIZE.get(),
        secondary_ingredient: EntityId::null(),
        primary_yield: MAIZE_SEEDS.get(),
        secondary_yield: MAIZE_SEEDS.get(),
    )
    .spawn();
}

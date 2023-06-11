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

pub fn init_data() {
    use crate::components::crafting::*;

    spawn_entity!(
        recipe: (),
        primary_ingredient: *BLUE_ITEM,
        secondary_ingredient: *YELLOW_ITEM,
        primary_yield: *GREEN_ITEM,
        secondary_yield: EntityId::null(),
    );
}

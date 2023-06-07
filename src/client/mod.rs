mod items;
mod map;
mod player;
use ambient_api::{
    client::{material, mesh, sampler, texture},
    components::core::{
        camera::aspect_ratio_from_window,
        procedurals::{procedural_material, procedural_mesh},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    prelude::*,
};

use palette::IntoColor;
use ambient_api::components::core::primitives::cube;

#[path = "../grid/mod.rs"]
mod grid;

#[path = "../tooling/mod.rs"]
mod tooling;

#[path = "../tree/mod.rs"]
mod tree;

const RESOLUTION_X: u32 = 32;
const RESOLUTION_Y: u32 = 32;
const TEXTURE_RESOLUTION_X: u32 = 4 * RESOLUTION_X;
const TEXTURE_RESOLUTION_Y: u32 = 4 * RESOLUTION_Y;
const SIZE_X: f32 = RESOLUTION_X as f32 / RESOLUTION_Y as f32;
const SIZE_Y: f32 = 1.0;

const TAU: f32 = std::f32::consts::TAU;
const WAVE_AMPLITUDE: f32 = 0.25;
const WAVE_FREQUENCY: f32 = 0.5 * TAU;


fn register_augmentors() {
    
    spawn_query((
        components::tree_seed(),
        components::tree_foliage_density(),
        components::tree_foliage_radius(),
        components::tree_foliage_segments(),
        components::tree_branch_length(),
        components::tree_branch_angle(),
        components::tree_trunk_height(),
        components::tree_trunk_radius(),
        components::tree_trunk_segments(),
    ))
    .bind(move |trees| {
        for (
            id,
            (
                seed,
                foliage_density,
                foliage_radius,
                foliage_segments,
                branch_length,
                branch_angle,
                trunk_height,
                trunk_radius,
                trunk_segments,
            ),
        ) in trees
        {
            let tree = tree::create_tree(tree::TreeMesh {
                seed,
                trunk_radius,
                trunk_height,
                trunk_segments,
                branch_length,
                branch_angle,
                branch_segments: 8,
                foliage_radius,
                foliage_density,
                foliage_segments,
            });
            let mesh = mesh::create(&mesh::Descriptor {
                vertices: &tree.vertices,
                indices: &tree.indices,
            });

            entity::add_components(
                id,
                Entity::new().with(procedural_mesh(), mesh), //.with_default(cast_shadows()),
            );
        }
    });
}

fn make_vegetation(vegetation_type: &str) {
    let (seed, num_vegetation) = match vegetation_type {
        "trees" => (123456, 30),
        "trees2" => (123460, 30),
        "rocks" => (123457, 60),
        _ => panic!("Invalid vegetation type"),
    };

    for i in 0..num_vegetation {
        let (
            trunk_radius,
            trunk_height,
            trunk_segments,
            branch_length,
            branch_angle,
            foliage_density,
            foliage_radius,
            foliage_segments,
        ) = match vegetation_type {
            "trees" => (
                tooling::gen_rn(seed + i, 10.0, 15.0),
                tooling::gen_rn(seed + i, 15.0, 20.0),
                tooling::gen_rn(seed + i, 12.0, 20.0) as u32,
                tooling::gen_rn(seed + i, 0.1, 0.3),
                tooling::gen_rn(seed + i, 10.0, 12.0),
                5,
                2.0,
                5,
            ),
            "trees2" => (
                tooling::gen_rn(seed + i, 2.5, 4.0),
                tooling::gen_rn(seed + i, 1.0, 3.0),
                tooling::gen_rn(seed + i, 6.0, 12.0) as u32,
                tooling::gen_rn(seed + i, 0.3, 0.4),
                tooling::gen_rn(seed + i, 60.0, 90.0),
                1,
                1.0,
                1,
            ),
            "rocks" => (
                tooling::gen_rn(seed + i, 4.5, 5.0),
                tooling::gen_rn(seed + i, 3.5, 5.0),
                3,
                tooling::gen_rn(seed + i, 0.3, 0.4),
                tooling::gen_rn(seed + i, 60.0, 90.0),
                1,
                1.0,
                1,
            ),
            _ => panic!("Invalid vegetation type"),
        };

        let x = tooling::gen_rn(seed + i, 0.0, 15.0) * 2.0;
        let y = tooling::gen_rn(seed + seed + i, 0.0, 15.0) * 2.0;
        let position = vec3(x, y, 0.0);//tooling::get_height(x, y) * 2.0 - 0.1);

        let _id = Entity::new()
            .with_merge(concepts::make_tree())
            .with_merge(make_transformable())
            //.with_default(cube())
            .with(
                scale(),
                Vec3::ONE
                    * tooling::gen_rn(
                        i,
                        0.05,
                        0.1
                    ),
            )
            .with(translation(), position)
            .with(components::tree_seed(), seed + i)
            .with(components::tree_trunk_radius(), trunk_radius)
            .with(components::tree_trunk_height(), trunk_height)
            .with(components::tree_trunk_segments(), trunk_segments)
            .with(components::tree_branch_length(), branch_length)
            .with(components::tree_branch_angle(), branch_angle)
            .with(components::tree_foliage_density(), foliage_density)
            .with(components::tree_foliage_radius(), foliage_radius)
            .with(components::tree_foliage_segments(), foliage_segments)
            .with(
                pbr_material_from_url(),
                if vegetation_type == "trees2" {
                    asset::url("assets/pipeline.json/1/mat.json").unwrap()
                } else
                if vegetation_type == "rocks" {
                    asset::url("assets/pipeline.json/2/mat.json").unwrap()
                } else {
                    asset::url("assets/pipeline.json/0/mat.json").unwrap()
                },
            )
            .spawn();        
    }
}

#[ambient_api::main]
pub async fn main() {
    items::init_items();
    map::init_map();

    register_augmentors();
    make_vegetation("trees");
    make_vegetation("trees2");
    make_vegetation("rocks");
    player::init_players().await;

}

use std::f32::consts::{FRAC_PI_2, TAU};

use ambient_api::prelude::*;

use ambient_api::{
    client::{material, mesh, sampler, texture},
    components::core::{
        camera::aspect_ratio_from_window,
        primitives::quad,
        procedurals::{procedural_material, procedural_mesh},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_transformable},
    mesh::Vertex,
};

use components::rotating_sun;
use palette::IntoColor;
use rand_chacha::ChaCha8Rng;

const RESOLUTION_X: u32 = 32;
const RESOLUTION_Y: u32 = 8;
const TEXTURE_RESOLUTION_X: u32 = 4 * RESOLUTION_X;
const TEXTURE_RESOLUTION_Y: u32 = 4 * RESOLUTION_Y;
const SIZE_X: f32 = RESOLUTION_X as f32 / RESOLUTION_Y as f32;
const SIZE_Y: f32 = 1.0;

mod tree;
mod tooling;
mod grid;

#[element_component]
fn App(_hooks: &mut Hooks, sun_id: EntityId) -> Element {
    FocusRoot::el([FlowColumn::el([FlowRow::el([Button::new(
        "Toggle sun rotation",
        move |_| {
            entity::mutate_component(sun_id, rotating_sun(), |rotating_sun| {
                *rotating_sun = !*rotating_sun;
            });
        },
    )
    .el()])])
    .with_padding_even(10.0)])
}
fn make_texture<PixelFn>(mut pixel_fn: PixelFn) -> ProceduralTextureHandle
where
    PixelFn: FnMut(f32, f32) -> [u8; 4],
{
    let mut pixels = vec![0_u8; (4 * TEXTURE_RESOLUTION_X * TEXTURE_RESOLUTION_Y) as usize];
    for y in 0..TEXTURE_RESOLUTION_Y {
        for x in 0..TEXTURE_RESOLUTION_X {
            let dst = (4 * (x + y * TEXTURE_RESOLUTION_X)) as usize;
            let dst = &mut pixels[dst..(dst + 4)];
            let px = (x as f32 + 0.5) / (TEXTURE_RESOLUTION_X as f32);
            let py = (y as f32 + 0.5) / (TEXTURE_RESOLUTION_Y as f32);
            dst.copy_from_slice(&pixel_fn(px, py));
        }
    }
    texture::create_2d(&texture::Descriptor2D {
        width: TEXTURE_RESOLUTION_X,
        height: TEXTURE_RESOLUTION_Y,
        format: texture::Format::Rgba8Unorm,
        data: &pixels,
    })
}

fn default_nearest_sampler() -> ProceduralSamplerHandle {
    sampler::create(&sampler::Descriptor {
        address_mode_u: sampler::AddressMode::ClampToEdge,
        address_mode_v: sampler::AddressMode::ClampToEdge,
        address_mode_w: sampler::AddressMode::ClampToEdge,
        mag_filter: sampler::FilterMode::Nearest,
        min_filter: sampler::FilterMode::Nearest,
        mipmap_filter: sampler::FilterMode::Nearest,
    })
}

fn register_tree_entity_augmentors() {

    let base_color_map = make_texture(|x, _| {
        let hsl = palette::Hsl::new(360.0 * x, 1.0, 0.5).into_format::<f32>();
        let rgb: palette::LinSrgb = hsl.into_color();
        let r = 50;//(255.0 * rgb.red/2.0) as u8;
        let g = (255.0 * rgb.green) as u8;
        let b = 50;//(255.0 * rgb.blue/2.0) as u8;
        let a = 255;
        [r, g, b, a]
    });

    let base_color_map2 = make_texture(|x, y| {
            let mx = x * 10.0;
            let my = y * 10.0;
            let mut h = tooling::get_height(mx, my);
            h = h * 255.0 / 4.0;
            let r = h as u8 + 100;
            let g = h as u8 + 50;
            let b = h as u8;
            let a = 255 as u8;
            [r, g, b, a]
    });


    let metallic_roughness_map2 = make_texture(|x, y| {
        let mx = x * 10.0;
        let my = y * 10.0;
        let mut h = tooling::get_height(mx, my);
        h = h * 255.0 / 10.0;
        let r = h as u8;
        let g = h as u8;
        let b = h as u8;
        let a = 255 as u8;
        [r, g, b, a]
    });


    let normal_map = make_texture(|_, _| [128, 128, 255, 0]);
    let _normal_map2 = make_texture(|_, _| [255, 128, 255, 0]);
    let metallic_roughness_map = make_texture(|_, _| [255, 255, 0, 0]);
    let sampler = sampler::create(&sampler::Descriptor {
        address_mode_u: sampler::AddressMode::ClampToEdge,
        address_mode_v: sampler::AddressMode::ClampToEdge,
        address_mode_w: sampler::AddressMode::ClampToEdge,
        mag_filter: sampler::FilterMode::Nearest,
        min_filter: sampler::FilterMode::Nearest,
        mipmap_filter: sampler::FilterMode::Nearest,
    });

    let material2 = material::create(&material::Descriptor {
        base_color_map: base_color_map2,
        normal_map : base_color_map2,
        metallic_roughness_map : metallic_roughness_map2,
        sampler,
        transparent: false,
    });
    let _material = material::create(&material::Descriptor {
        base_color_map: base_color_map,
        normal_map,
        metallic_roughness_map,
        sampler,
        transparent: false,
    });

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
                Entity::new()
                    .with(procedural_mesh(), mesh)
                    // green
                    //.with(procedural_material(), material)
                    .with_default(cast_shadows()),
            );
        }
    });

}
fn make_vegetation(vegetation_type: &str) {
    let (seed, num_vegetation) = match vegetation_type {
        "trees" => (123456, 50),
        "bush" => (123457, 100),
        "mushrooms" => (123458, 50),
        _ => panic!("Invalid vegetation type"),
    };

    for i in 0..num_vegetation {
        let (trunk_radius, trunk_height, trunk_segments, branch_length, branch_angle, foliage_density, foliage_radius, foliage_segments) =
            match vegetation_type {
                "trees" => (
                    tooling::gen_rn(seed + i, 2.0, 3.0),
                    tooling::gen_rn(seed + i, 15.0, 20.0),
                    tooling::gen_rn(seed + i, 6.0, 12.0) as u32,
                    tooling::gen_rn(seed + i, 0.1, 0.3),
                    tooling::gen_rn(seed + i, 10.0, 12.0),
                    5,
                    2.0,
                    5,
                ),
                "bush" => (
                    tooling::gen_rn(seed + i, 0.2, 0.3),
                    0.01,
                    1,
                    1.0,
                    1.0,
                    0,
                    0.0,
                    0,
                ),
                "mushrooms" => (
                    1.0,
                    2.0,
                    2,
                    0.01,
                    0.01,
                    1,
                    0.9,
                    5,
                ),
                _ => panic!("Invalid vegetation type"),
            };

        let x = tooling::gen_rn(seed + i, 0.0, 5.0) * 2.0;
        let y = tooling::gen_rn(seed + seed + i, 0.0, 5.0) * 2.0;
        let position = vec3(x, y, 0.0);//tooling::get_height(x, y)*2.0)+0.2;

        let _id = Entity::new()
            .with_merge(concepts::make_tree())
            .with_merge(make_transformable())
            .with(scale(), Vec3::ONE * tooling::gen_rn(i, if vegetation_type == "trees" { 0.06 } else { 0.1 }, if vegetation_type == "trees" { 0.16 } else { 0.2 }))
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
                if vegetation_type == "mushrooms" { asset::url("assets/pipeline.json/1/mat.json").unwrap() } else { asset::url("assets/pipeline.json/0/mat.json").unwrap() },
            )
            .spawn();
    }
}

#[main]
pub fn main() {
    register_tree_entity_augmentors();
    
    make_vegetation("trees");
    make_vegetation("bush");
    make_vegetation("mushrooms");    
    
    let mut cursor_lock = input::CursorLockGuard::new(true);
    let mut pitch = 0.0;
    let mut yaw = 0.0;
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
    });
}

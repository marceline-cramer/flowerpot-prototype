use ambient_api::{
    client::{mesh},
    mesh::Vertex,
    prelude::*,
};

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[path = "../tooling/mod.rs"]
mod tooling;

const RESOLUTION_X: u32 = 32;
const RESOLUTION_Y: u32 = 32;
const TEXTURE_RESOLUTION_X: u32 = 4 * RESOLUTION_X;
const TEXTURE_RESOLUTION_Y: u32 = 4 * RESOLUTION_Y;
const SIZE_X: f32 = RESOLUTION_X as f32 / RESOLUTION_Y as f32;
const SIZE_Y: f32 = 1.0;



const TAU: f32 = std::f32::consts::TAU;
const WAVE_AMPLITUDE: f32 = 0.25;
const WAVE_FREQUENCY: f32 = 0.5 * TAU;

#[derive(Clone)]
pub struct TreeMesh {
    pub seed: i32,
    pub trunk_radius: f32,
    pub trunk_height: f32,
    pub trunk_segments: u32,
    pub branch_length: f32,
    pub branch_angle: f32,
    pub branch_segments: u32,
    pub foliage_radius: f32,
    pub foliage_density: u32,
    pub foliage_segments: u32,
}

pub fn create_tree(tree: TreeMesh) -> tooling::MeshDescriptor {
    // Create the trunk
    let (mut vertices1, top_vertices1, mut normals1, mut uvs1, _trunk_direction) =
        build_trunk(&tree);

    let sectors = 12;
    let trunk_segments = tree.trunk_segments;
    let mut indices = Vec::new();
    let trunk_vertices_count = (trunk_segments + 1) * (sectors + 1);

    let _vertices: Vec<Vertex> = Vec::with_capacity(vertices1.len());

    // Connect trunk segments
    for i in 0..(trunk_segments) {
        for j in 0..sectors {
            let k1 = i * (sectors + 1) + j;
            let k2 = (i + 1) * (sectors + 1) + j;

            indices.push(k1);
            indices.push(k1 + 1);
            indices.push(k2);

            indices.push(k1 + 1);
            indices.push(k2 + 1);
            indices.push(k2);
        }
    }

    // Generate branches
    let branch_count = tree.branch_segments;
    let _branch_radius_variance = 0.02;
    let branch_position_variance = vec3(0.8, 0.8, 0.7);

    let mut rng = ChaCha8Rng::seed_from_u64(tree.seed as u64);

    for i in 0..branch_count {
        let branch_radius = 0.3;
        //* (1.0 - rng.gen_range(0.0..1.0) * branch_radius_variance);
        let mut branch_position = top_vertices1[rng.gen_range(0..top_vertices1.len())]
            + vec3(
                rng.gen_range(0.0..1.0) * branch_position_variance.x,
                rng.gen_range(0.0..1.0) * branch_position_variance.y,
                rng.gen_range(0.0..1.0) * branch_position_variance.z - 1.0,
            );

        let segments = tree.branch_segments;
        let sector_step = 2. * std::f32::consts::PI / segments as f32;

        let mut branch_vertices = Vec::new();
        let mut branch_normals = Vec::new();
        let mut branch_uvs = Vec::new();

        let mut direction = vec3(0.0, 0.0, 1.0);
        let direction_variance = 0.05;

        // Get a random vertex from the top vertices of the trunk
        let random_vertex_index = rng.gen_range(0..top_vertices1.len());
        let random_vertex = normals1[random_vertex_index];

        // Calculate the initial direction of the branch from the chosen vertex
        direction = (random_vertex - branch_position).normalize()
            + vec3(
                tooling::gen_rn(tree.seed + i as i32 + 4, -1.0, 1.0),
                tooling::gen_rn(tree.seed + i as i32 + 5, -1.0, 1.0),
                0.0,
            );

        for i in 0..=segments {
            let random_direction = vec3(
                tooling::gen_rn(tree.seed + i as i32 + 1, 0.0, 1.0) - 0.5,
                tooling::gen_rn(tree.seed + i as i32 + 2, 0.0, 1.0) - 0.5,
                tooling::gen_rn(tree.seed + i as i32 + 3, 0.0, 1.0) - 0.5,
            )
            .normalize()
                * direction_variance;
            direction = (direction + random_direction).normalize();

            let theta = (i as f32 / segments as f32) * tree.branch_angle;
            let height = branch_position.z + (tree.branch_length * theta.cos()) * direction.z;
            let segment_radius = branch_radius * theta.sin();

            for j in 0..=sectors {
                let phi = j as f32 * sector_step;
                let x = branch_position.x + segment_radius * phi.cos();
                let y = branch_position.y + segment_radius * phi.sin();
                let z = height;

                branch_vertices.push(vec3(x, y, z));
                branch_normals.push(
                    vec3(
                        x - branch_position.x,
                        y - branch_position.y,
                        z - branch_position.z,
                    )
                    .normalize(),
                );
                branch_uvs.push(vec2(j as f32 / sectors as f32, i as f32 / segments as f32));
            }
            branch_position = branch_position
                + vec3(
                    rng.gen_range(-1.0..1.0) * branch_position_variance.x,
                    rng.gen_range(-1.0..1.0) * branch_position_variance.y,
                    rng.gen_range(0.0..1.0) * branch_position_variance.z,
                );
        }

        let branch_indices = generate_branch_indices(segments as usize, sectors as usize);

        vertices1.extend(branch_vertices.clone());
        normals1.extend(branch_normals);
        uvs1.extend(branch_uvs);

        let offset = vertices1.len() - branch_vertices.len();
        indices.extend(branch_indices.iter().map(|i| *i + offset as u32));
    }

    // Generate foliage
    let foliage_count = tree.foliage_density + tree.foliage_segments;
    let foliage_radius_variance = 0.05;
    let mut foliage_position_variance = vec3(3.0, 3.0, 3.0);
    if tree.foliage_radius < 1.0
    {
//        foliage_count = 2;
//        foliage_radius_variance = 0.1;
        foliage_position_variance = vec3(0.1, 0.1, 0.1);
    }
    for i in 0..foliage_count {
        let foliage_radius = tree.foliage_radius
            * (1.0 - tooling::gen_rn(tree.seed + i as i32, 0.0, 1.0) * foliage_radius_variance);
        let mut position_shift_z = 2.0;
        if foliage_radius < 1.0 {
            position_shift_z = 0.0;
        }
        let foliage_position = top_vertices1
            [tooling::gen_rn(tree.seed, 0.0, top_vertices1.len() as f32) as usize]
            + vec3(
                tooling::gen_rn(tree.seed + i as i32, -1.0, 1.0) * foliage_position_variance.x,
                tooling::gen_rn(tree.seed + i as i32 + 1, -1.0, 1.0) * foliage_position_variance.y,
                tooling::gen_rn(tree.seed + i as i32 + 2, 0.0, 1.0) * foliage_position_variance.z + position_shift_z,
            );

        let segments = tree.foliage_segments;
        let _density = tree.foliage_density;
        let sector_step = 2. * std::f32::consts::PI / segments as f32;

        let mut sphere_vertices = Vec::new();
    let mut sphere_normals = Vec::new();
    let mut sphere_uvs = Vec::new();

    for i in 0..=segments {
        let theta = (i as f32 / segments as f32) * std::f32::consts::PI;
        let height = foliage_position.z + (foliage_radius * theta.cos());
        let segment_radius = foliage_radius * theta.sin();

        for j in 0..=segments {
            let phi = j as f32 * sector_step;
            let x = foliage_position.x + segment_radius * phi.cos();
            let y = foliage_position.y + segment_radius * phi.sin();
            let z = height;

            sphere_vertices.push(vec3(x, y, z));

            // Calculate the foliage normal based on the vector from the foliage position to the vertex
            let normal = vec3(x, y, z) - foliage_position;
            sphere_normals.push(normal.normalize());


            // Calculate the v-coordinate for foliage vertices
            let v = if i < segments {
                // Map foliage to the upper portion of the texture
                0.5 + (i as f32 / segments as f32) * 0.5
            } else {
                // Map trunk to the lower portion of the texture
                i as f32 / segments as f32
            };

            // Update the UV coordinate calculation
            sphere_uvs.push(vec2(j as f32 / segments as f32, v));
        }
    }

        let sphere_indices = generate_sphere_indices(segments as usize, segments as usize);

        vertices1.extend(sphere_vertices.clone());
        normals1.extend(sphere_normals);
        uvs1.extend(sphere_uvs);

        let offset = vertices1.len() - sphere_vertices.len();
        indices.extend(sphere_indices.iter().map(|i| *i + offset as u32));
    }

    // Function to generate indices for a sphere based on segments and density
    fn generate_sphere_indices(segments: usize, density: usize) -> Vec<u32> {
        let mut indices = Vec::with_capacity(segments * density * 6);

        for i in 0..segments {
            for j in 0..density {
                let index1 = i * (density + 1) + j;
                let index2 = index1 + 1;
                let index3 = (i + 1) * (density + 1) + j;
                let index4 = index3 + 1;

                indices.push(index1 as u32);
                indices.push(index2 as u32);
                indices.push(index3 as u32);

                indices.push(index2 as u32);
                indices.push(index4 as u32);
                indices.push(index3 as u32);
            }
        }
        indices
    }

    // Function to generate indices for a branch based on segments and sectors
    fn generate_branch_indices(segments: usize, sectors: usize) -> Vec<u32> {
        let mut indices = Vec::with_capacity(segments * sectors * 6);

        for i in 0..segments {
            for j in 0..sectors {
                let index1 = i * (sectors + 1) + j;
                let index2 = index1 + 1;
                let index3 = (i + 1) * (sectors + 1) + j;
                let index4 = index3 + 1;

                indices.push(index1 as u32);
                indices.push(index2 as u32);
                indices.push(index3 as u32);

                indices.push(index2 as u32);
                indices.push(index4 as u32);
                indices.push(index3 as u32);
            }
        }
        indices
    }

    let mut vertices: Vec<Vertex> = Vec::with_capacity(vertices1.len());

    for i in 0..vertices1.len() {
        let px = vertices1[i].x;
        let py = vertices1[i].y;
        let pz = vertices1[i].z;
        let _u = uvs1[i].x;
        let _v = uvs1[i].y;
        let nx = normals1[i].x;
        let ny = normals1[i].y;
        let nz = normals1[i].z;

        let v = mesh::Vertex {
            position: vec3(px, py, pz) + vec3(-0.5 * SIZE_X, -0.5 * SIZE_Y, 0.0),
            normal: vec3(nx, ny, nz),
            tangent: vec3(1.0, 0.0, 0.0),
            texcoord0: if i < trunk_vertices_count as usize {
                // Trunk UVs (bottom half of the texture)
                let u = uvs1[i].x;
                let v = uvs1[i].y * 0.5;
                vec2(u, v)
            } else {
                // Foliage UVs (upper half of the texture)
                let u = uvs1[i].x;
                let v = uvs1[i].y * 0.5 + 0.5;
                vec2(u, v)
            },
        };

        vertices.push(v);
        // let v = mesh::Vertex {
        //     position: vec3(px, py, pz) + vec3(-0.5 * SIZE_X, -0.5 * SIZE_Y, 0.0),
        //     normal: vec3(nx, ny, nz),
        //     tangent: vec3(1.0, 0.0, 0.0),
        //     texcoord0: vec2(u, v),
        // };
        // vertices.push(v);
    }

    tooling::MeshDescriptor { vertices, indices }
}

fn build_trunk(tree: &TreeMesh) -> (Vec<Vec3>, Vec<Vec3>, Vec<Vec3>, Vec<Vec2>, Vec3) {
    let sectors = 12;
    let sector_step = 2. * std::f32::consts::PI / sectors as f32;

    let mut vertices = Vec::new();
    let mut top_vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();

    let mut trunk_direction = vec3(0.0, 0.0, 1.0);
    let direction_variance = 0.08;

    let radius_variance = 0.02;

    for i in 0..=tree.trunk_segments {
        let variance = tooling::gen_rn(tree.seed + i as i32, 0.0, 1.0) * radius_variance;
        let z = tree.trunk_height * (i as f32 / tree.trunk_segments as f32);
        let s = tree.trunk_radius;
        let radius = s * (1.0 - i as f32 / tree.trunk_segments as f32) * (1.0 - variance);

        let top_segment_radius = tree.trunk_radius * 0.1;
        let radius = if i == tree.trunk_segments && radius < top_segment_radius {
            top_segment_radius
        } else {
            radius
        };

        let random_direction = vec3(
            tooling::gen_rn(tree.seed + i as i32 + 1, 0.0, 1.0) - 0.5,
            tooling::gen_rn(tree.seed + i as i32 + 2, 0.0, 1.0) - 0.5,
            tooling::gen_rn(tree.seed + i as i32 + 3, 0.0, 1.0) - 0.5,
        )
        .normalize()
            * direction_variance;
        trunk_direction = (trunk_direction + random_direction).normalize();

        let top_position = trunk_direction * z;

        let gravity_factor = (1.0 - (i as f32 / tree.trunk_segments as f32)).powf(2.0);
        let gravity_offset = trunk_direction * gravity_factor * 2.0 * i as f32;

        for j in 0..=sectors {
            let sector_angle = j as f32 * sector_step;
            let x = radius * sector_angle.cos();
            let y = radius * sector_angle.sin();

            vertices.push(top_position + vec3(x, y, 0.0) - gravity_offset);
            normals.push(vec3(x, y, 0.0).normalize());
            uvs.push(vec2(
                j as f32 / sectors as f32,
                i as f32 / tree.trunk_segments as f32,
            ));
        }

        if i == tree.trunk_segments {
            let top_vertex_start = vertices.len() - sectors - 1;
            let top_vertex_end = vertices.len();
            top_vertices.extend(vertices[top_vertex_start..top_vertex_end].iter().cloned());

            // Add faces to connect the last ring of vertices
            for j in 0..sectors {
                let v1 = top_vertex_start + j;
                let v2 = top_vertex_start + j + 1;
                let v3 = top_vertex_end - 1;
                let v4 = top_vertex_start;

                // First triangle
                vertices.push(vertices[v1] - gravity_offset);
                vertices.push(vertices[v2] - gravity_offset);
                vertices.push(vertices[v3] - gravity_offset);

                normals.push(normals[v1]);
                normals.push(normals[v2]);
                normals.push(normals[v3]);

                uvs.push(uvs[v1]);
                uvs.push(uvs[v2]);
                uvs.push(uvs[v3]);

                // Second triangle
                vertices.push(vertices[v1] - gravity_offset);
                vertices.push(vertices[v3] - gravity_offset);
                vertices.push(vertices[v4] - gravity_offset);

                normals.push(normals[v1]);
                normals.push(normals[v3]);
                normals.push(normals[v4]);

                uvs.push(uvs[v1]);
                uvs.push(uvs[v3]);
                uvs.push(uvs[v4]);
            }
        }
    }

    (vertices, top_vertices, normals, uvs, trunk_direction)
}
use ambient_api::{
    client::{mesh},
    mesh::Vertex,
    prelude::*,
};

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

#[derive(Debug, Clone)]
pub struct GridMesh {
    pub top_left: glam::Vec2,
    pub size: glam::Vec2,
    pub n_vertices_width: usize,
    pub n_vertices_height: usize,
    pub uv_min: glam::Vec2,
    pub uv_max: glam::Vec2,
    pub normal: glam::Vec3,
}

impl Default for GridMesh {
    fn default() -> GridMesh {
        GridMesh {
            top_left: glam::Vec2::ZERO,
            size: glam::Vec2::ONE,
            n_vertices_width: 10,
            n_vertices_height: 10,
            uv_min: glam::Vec2::ZERO,
            uv_max: glam::Vec2::ONE,
            normal: glam::Vec3::Z,
        }
    }
}

pub fn create_tile(grid: GridMesh) -> tooling::MeshDescriptor {
    // Create the tile
    let (vertices1, uvs1, normals1, indices) = build_tile(&grid);

    let mut vertices: Vec<Vertex> = Vec::with_capacity(vertices1.len());

    for i in 0..vertices1.len() {
        let px = vertices1[i].x;
        let py = vertices1[i].y;
        let pz = vertices1[i].z;
        let u = uvs1[i].x;
        let v = uvs1[i].y;
        let nx = normals1[i].x;
        let ny = normals1[i].y;
        let nz = normals1[i].z;

        let v = mesh::Vertex {
            position: vec3(px, py, pz) + vec3(-0.5 * SIZE_X, -0.5 * SIZE_Y, 0.0),
            normal: vec3(nx, ny, nz),
            tangent: vec3(1.0, 0.0, 0.0),
            texcoord0: vec2(u, v),
        };
        vertices.push(v);
    }

    tooling::MeshDescriptor { vertices, indices }
}

pub fn build_tile(grid: &GridMesh) -> (Vec<Vec3>, Vec<Vec2>, Vec<Vec3>, Vec<u32>) {
    let mut positions = Vec::new();
    let mut texcoords = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    for y in 0..grid.n_vertices_height {
        for x in 0..grid.n_vertices_width {
            let p = glam::Vec2::new(
                2.0 * x as f32 / (grid.n_vertices_width as f32 - 1.0),
                2.0 * y as f32 / (grid.n_vertices_height as f32 - 1.0),
            );
            positions.push(vec3(
                grid.top_left.x + grid.size.x * p.x,
                grid.top_left.y + grid.size.y * p.y,
                tooling::get_height((grid.top_left.x + grid.size.x * p.x)*2.0, (grid.top_left.y + grid.size.y * p.y)*2.0),
            ));
            texcoords.push(vec2(
                grid.uv_min.x + (grid.uv_max.x - grid.uv_min.x) * p.x,
                grid.uv_min.y + (grid.uv_max.y - grid.uv_min.y) * p.y,
            ));
            let normal = grid.normal;
            normals.push(vec3(normal.x, normal.y, normal.z));
            if y < grid.n_vertices_height - 1 && x < grid.n_vertices_width - 1 {
                let vert_index = x + y * grid.n_vertices_width;
                indices.push((vert_index) as u32);
                indices.push((vert_index + 1) as u32);
                indices.push((vert_index + grid.n_vertices_width) as u32);

                indices.push((vert_index + 1) as u32);
                indices.push((vert_index + grid.n_vertices_width + 1) as u32);
                indices.push((vert_index + grid.n_vertices_width) as u32);
            }
        }
    }

    (positions, texcoords, normals, indices)
}
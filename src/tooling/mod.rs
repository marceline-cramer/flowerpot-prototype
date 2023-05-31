use rand::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use ambient_api::mesh::Vertex;

pub struct MeshDescriptor {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub fn gen_rn(seed: i32, min: f32, max: f32) -> f32 {
    let mut rng = ChaCha8Rng::seed_from_u64(seed as u64);
    rng.gen_range(min..max)
}

pub struct SimplexNoise {
    grad3: [[i32; 3]; 12],
    perm: Vec<i32>,
    sqrt3: f32,
}

impl SimplexNoise {
    pub fn new() -> SimplexNoise {
        let grad3: [[i32; 3]; 12] = [
            [1, 1, 0],
            [-1, 1, 0],
            [1, -1, 0],
            [-1, -1, 0],
            [1, 0, 1],
            [-1, 0, 1],
            [1, 0, -1],
            [-1, 0, -1],
            [0, 1, 1],
            [0, -1, 1],
            [0, 1, -1],
            [0, -1, -1],
        ];

        let p: Vec<i32> = vec![
            151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30,
            69, 142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0, 26, 197, 62, 94,
            252, 219, 203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171,
            168, 68, 175, 74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60,
            211, 133, 230, 220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161, 1,
            216, 80, 73, 209, 76, 132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188, 159, 86,
            164, 100, 109, 198, 173, 186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118,
            126, 255, 82, 85, 212, 207, 206, 59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170,
            213, 119, 248, 152, 2, 44, 154, 163, 70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39,
            253, 19, 98, 108, 110, 79, 113, 224, 232, 178, 185, 112, 104, 218, 246, 97, 228, 251, 34,
            242, 193, 238, 210, 144, 12, 191, 179, 162, 241, 81, 51, 145, 235, 249, 14, 239, 107, 49,
            192, 214, 31, 181, 199, 106, 157, 184, 84, 204, 176, 115, 121, 50, 45, 127, 4, 150, 254,
            138, 236, 205, 93, 222, 114, 67, 29, 24, 72, 243, 141, 128, 195, 78, 66, 215, 61, 156, 180,
        ];

        let mut perm = vec![0; 512];
        for i in 0..512 {
            perm[i] = p[i & 255];
        }

        SimplexNoise {
            grad3,
            perm,
            sqrt3: f32::sqrt(3.0),
        }
    }

    fn floor(&self, n: f32) -> i32 {
        if n > 0.0 {
            n as i32
        } else {
            (n as i32) - 1
        }
    }

    fn dot(&self, g: [i32; 3], x: f32, y: f32) -> f32 {
        (g[0] as f32) * x + (g[1] as f32) * y
    }

    pub fn noise(&self, x: f32, y: f32) -> f32 {
        let n0: f32;
        let n1: f32;
        let n2: f32;
        let f2: f32 = 0.5 * (self.sqrt3 - 1.0);
        let s: f32 = (x + y) * f2;
        let i: i32 = self.floor(x + s);
        let j: i32 = self.floor(y + s);
        let g2: f32 = (3.0 - self.sqrt3) / 6.0;
        let t: f32 = (i + j) as f32 * g2;
        let x0: f32 = x - (i as f32) + t;
        let y0: f32 = y - (j as f32) + t;
        let (i1, j1): (i32, i32);
        if x0 > y0 {
            i1 = 1;
            j1 = 0;
        } else {
            i1 = 0;
            j1 = 1;
        }
        let x1: f32 = x0 - (i1 as f32) + g2;
        let y1: f32 = y0 - (j1 as f32) + g2;
        let x2: f32 = x0 - 1.0 + 2.0 * g2;
        let y2: f32 = y0 - 1.0 + 2.0 * g2;
        let ii: i32 = i & 255;
        let jj: i32 = j & 255;
        let gi0: i32 = self.perm[(ii + self.perm[jj as usize]) as usize] % 12;
        let gi1: i32 = self.perm[(ii + i1 + self.perm[(jj + j1) as usize]) as usize] % 12;
        let gi2: i32 = self.perm[(ii + 1 + self.perm[(jj + 1) as usize]) as usize] % 12;
        let t0: f32 = 0.5 - x0 * x0 - y0 * y0;
        if t0 < 0.0 {
            n0 = 0.0;
        } else {
            let t02: f32 = t0 * t0;
            n0 = t02 * t02 * self.dot(self.grad3[gi0 as usize], x0, y0);
        }
        let t1: f32 = 0.5 - x1 * x1 - y1 * y1;
        if t1 < 0.0 {
            n1 = 0.0;
        } else {
            let t12: f32 = t1 * t1;
            n1 = t12 * t12 * self.dot(self.grad3[gi1 as usize], x1, y1);
        }
        let t2: f32 = 0.5 - x2 * x2 - y2 * y2;
        if t2 < 0.0 {
            n2 = 0.0;
        } else {
            let t22: f32 = t2 * t2;
            n2 = t22 * t22 * self.dot(self.grad3[gi2 as usize], x2, y2);
        }
        70.0 * (n0 + n1 + n2)
    }

    fn harmonic_noise_2d(
        &self,
        x: f32,
        y: f32,
        harmonics: i32,
        freq_x: f32,
        freq_y: f32,
        smoothness: f32,
    ) -> f32 {
        let mut h: f32 = 1.0;
        let mut sum: f32 = 0.0;
        for _ in 0..harmonics {
            sum += self.noise(x * h * freq_x, y * h * freq_y) / smoothness;
            h *= 2.0;
        }
        sum
    }
}

pub fn get_height(x:f32, y:f32) -> f32 {
    let x = x as f32;
    let y = y as f32;
    // perlin noise without crate
    let noise = x.sin() * y.cos();
    let mut height = noise * 0.5 + 0.5;

    let simplex = SimplexNoise::new();
    let mut level: f32 = 8.0;
    height += simplex.noise(x as f32 / level, y as f32 / level) / 2.0 + 0.5;
    level *= 3.0;
    height += (simplex.noise(x as f32 / level, y as f32 / level) / 2.0 + 0.5) * 0.7;
    level *= 2.0;
    height += (simplex.noise(x as f32 / level, y as f32 / level) / 2.0 + 0.5) * 1.0;
    // level *= 2.0;
    // height -= (f32::cos((x / 2.0 + 50.0) as f32 / 40.0) * 2.0)
    //     + (f32::sin((y / 2.0 + 110.0) as f32 / 40.0) * 2.0)
    //     + 6.0;
    // height += (simplex.noise(x as f32 / level, y as f32 / level) / 2.0 + 0.5) * 1.8;
    // height /= 1.0 + 0.5 + 0.25 + 0.125;
    height = (f32::cos(x)+f32::sin(y))/5.0;
    height
}

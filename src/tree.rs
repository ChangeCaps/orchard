use ike::{d3::Indices, prelude::*};
use rand::Rng;

#[derive(Clone, Debug)]
pub enum TreeStage {
    Sapling(f32),
    Grown,
}

impl Default for TreeStage {
    #[inline]
    fn default() -> Self {
        Self::Sapling(0.0)
    }
}

#[derive(Default)]
pub struct Tree {
    pub mesh: Mesh,
    pub stage: TreeStage,
    pub trunk_radius: f32,
    pub radius_decay: f32,
    pub branch_length: f32,
    pub trunk_color: Color,
    pub leaf_color: Color,
}

impl Tree {
    #[inline]
    pub fn update(&mut self, ctx: &mut UpdateCtx) {
        match self.stage {
            TreeStage::Sapling(ref mut growth) => {
                *growth += ctx.delta_time / 45.0;

                if *growth >= 1.0 {
                    self.stage = TreeStage::Grown;
                    self.generate_mesh_grown();
                }
            }
            _ => {}
        }
    }

    #[inline]
    pub fn generate_mesh_sapling(&mut self) {
        self.mesh.vertices.clear();
        self.mesh.indices.clear();

        let base = Ring::new(self.trunk_color, self.trunk_radius / 2.0, 5);
        let indices = base.insert(&mut self.mesh);

        let mut branches: Vec<BranchBase> = vec![BranchBase {
            indices,
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
        }];
        let mut trunk_radius = self.trunk_radius / 1.5;

        let mut rng = rand::thread_rng();

        for i in 0..4 {
            for base in std::mem::replace(&mut branches, Vec::new()) {
                let branch = Branch {
                    position: base.position,
                    rotation: base.rotation,
                    res: 5,
                    color: self.trunk_color,
                    radius: trunk_radius,
                    length: self.branch_length / 1.5,
                };

                let base = branch.generate(&mut self.mesh, &base.indices);

                let num_branches: u32;

                if i == 0 {
                    num_branches = rng.gen_range(3..4);
                } else if i < 3 {
                    num_branches = rng.gen_range(2..3);
                } else {
                    num_branches = rng.gen_range(1..2);
                }

                for j in 0..num_branches {
                    let mut base = base.clone();

                    if i == 0 {
                        let angle = (j as f32 / num_branches as f32) * std::f32::consts::TAU;
                        base.rotation.y = angle;
                        base.rotation.x -= 0.1;
                    } else {
                        base.rotation.y += rng.gen_range(-1.0..1.0);
                        base.rotation.x -= rng.gen_range(0.1..0.4);
                    }

                    branches.push(base);
                }
            }

            trunk_radius *= self.radius_decay;
        }

        for base in branches {
            sphere(&mut self.mesh, base.position, 4.0, self.leaf_color);
        }

        self.mesh.calculate_normals();
    }

    #[inline]
    pub fn generate_mesh_grown(&mut self) {
        self.mesh.vertices.clear();
        self.mesh.indices.clear();

        let base = Ring::new(self.trunk_color, self.trunk_radius, 5);
        let indices = base.insert(&mut self.mesh);

        let mut branches: Vec<BranchBase> = vec![BranchBase {
            indices,
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
        }];
        let mut trunk_radius = self.trunk_radius;

        let mut rng = rand::thread_rng();

        for i in 0..5 {
            for base in std::mem::replace(&mut branches, Vec::new()) {
                let branch = Branch {
                    position: base.position,
                    rotation: base.rotation,
                    res: 5,
                    color: self.trunk_color,
                    radius: trunk_radius,
                    length: self.branch_length,
                };

                let base = branch.generate(&mut self.mesh, &base.indices);

                let num_branches: u32;

                if i == 0 {
                    num_branches = rng.gen_range(3..4);
                } else if i < 3 {
                    num_branches = rng.gen_range(2..3);
                } else {
                    num_branches = rng.gen_range(1..2);
                }

                for j in 0..num_branches {
                    let mut base = base.clone();

                    if i == 0 {
                        let angle = (j as f32 / num_branches as f32) * std::f32::consts::TAU;
                        base.rotation.y = angle;
                        base.rotation.x -= 0.1;
                    } else {
                        base.rotation.y += rng.gen_range(-1.0..1.0);
                        base.rotation.x -= rng.gen_range(0.1..0.4);
                    }

                    branches.push(base);
                }
            }

            trunk_radius *= self.radius_decay;
        }

        for base in branches {
            sphere(&mut self.mesh, base.position, 8.0, self.leaf_color);
        }

        sphere(
            &mut self.mesh,
            Vec3::new(0.0, 40.0, 0.0),
            16.0,
            self.leaf_color,
        );

        self.mesh.calculate_normals();
    }
}

pub struct Ring {
    pub vertices: Vec<Vertex>,
}

impl Ring {
    #[inline]
    pub fn new(color: Color, radius: f32, res: usize) -> Self {
        let mut vertices = Vec::with_capacity(res);

        for i in 0..res {
            let a = i as f32 / res as f32 * std::f32::consts::TAU;

            vertices.push(Vertex {
                position: Vec3::new(a.cos(), 0.0, a.sin()) * radius,
                normal: Vec3::ZERO,
                uv: Vec2::ZERO,
                color,
            })
        }

        Self { vertices }
    }

    #[inline]
    pub fn rotate(&mut self, rot: Quat) {
        for vertex in &mut self.vertices {
            vertex.position = rot * vertex.position;
        }
    }

    #[inline]
    pub fn translate(&mut self, translation: Vec3) {
        for vertex in &mut self.vertices {
            vertex.position += translation;
        }
    }

    #[inline]
    pub fn insert(self, mesh: &mut Mesh) -> Vec<usize> {
        self.vertices
            .into_iter()
            .map(|vertex| {
                let index = mesh.vertices.len();

                mesh.vertices.push(vertex);

                index
            })
            .collect()
    }
}

#[derive(Clone)]
pub struct BranchBase {
    pub indices: Vec<usize>,
    pub position: Vec3,
    pub rotation: Vec3,
}

pub struct Branch {
    pub position: Vec3,
    // euler angles
    pub rotation: Vec3,
    pub res: usize,
    pub color: Color,
    pub radius: f32,
    pub length: f32,
}

impl Branch {
    pub fn generate(&self, mesh: &mut Mesh, base: &[usize]) -> BranchBase {
        let mut ring = Ring::new(self.color, self.radius, self.res);
        let rot = euler_rot(self.rotation);
        ring.rotate(rot);

        let direction = rot * Vec3::Y;
        let position = self.position + direction * self.length;

        ring.translate(position);

        let indices = ring.insert(mesh);

        bridge_loops(mesh, base, &indices);

        BranchBase {
            indices,
            position,
            rotation: self.rotation,
        }
    }
}

#[inline]
fn sphere(mesh: &mut Mesh, center: Vec3, radius: f32, color: Color) {
    let v = mesh.vertices.len() as u32;

    for i in 0..6 {
        let h = i as f32 / 2.5 - 1.0;
        let t = h.abs().acos().sin();

        for j in 0..6 {
            let a = j as f32 / 6.0 * std::f32::consts::TAU;

            let i0 = i * 6 + j;
            let i1 = i * 6 + ((j + 1) % 6);
            let j0 = i0 + 6;
            let j1 = i1 + 6;

            mesh.vertices.push(Vertex {
                position: center + Vec3::new(a.cos() * t, h, a.sin() * t) * radius,
                normal: Vec3::ZERO,
                uv: Vec2::ZERO,
                color,
            });

            if i < 5 {
                mesh.indices.push(v + i0);
                mesh.indices.push(v + j0);
                mesh.indices.push(v + i1);

                mesh.indices.push(v + i1);
                mesh.indices.push(v + j0);
                mesh.indices.push(v + j1);
            }
        }
    }
}

#[inline]
fn euler_rot(euler: Vec3) -> Quat {
    let m = Mat2::from_angle(euler.y);

    let e = m * Vec2::new(euler.x, euler.z);

    Quat::from_euler(EulerRot::XYZ, e.x, 0.0, e.y)
}

#[inline]
fn bridge_loops(mesh: &mut Mesh, from: &[usize], to: &[usize]) {
    assert_eq!(from.len(), to.len());

    for i in 0..from.len() {
        // get next index
        let j = (i + 1) % from.len();

        // add two triangles
        mesh.indices.push(from[i] as u32);
        mesh.indices.push(to[i] as u32);
        mesh.indices.push(from[j] as u32);

        mesh.indices.push(from[j] as u32);
        mesh.indices.push(to[i] as u32);
        mesh.indices.push(to[j] as u32);
    }
}

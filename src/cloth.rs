use std::collections::{BTreeSet, HashMap};

use ike::prelude::*;

use crate::render::{Mesh, Vertex};

pub struct Node {
    pub position: Vec3,
    pub prev_position: Vec3,
    pub locked: bool,
}

impl Node {
    #[inline]
    pub fn new(position: Vec3, locked: bool) -> Self {
        Self {
            position,
            prev_position: position,
            locked,
        }
    }
}

#[derive(Default)]
pub struct Cloth {
    pub mesh: Mesh,
    pub nodes: Vec<Node>,
    pub connections: HashMap<(usize, usize), f32>,
    pub flicker: BTreeSet<usize>,
}

impl Cloth {
    #[inline]
    pub fn generate(width: usize, height: usize) -> Self {
        let mut cloth = Self::default();

        for y in 0..height {
            for x in 0..width {
                let position = Vec3::new(-(x as f32), y as f32 - height as f32 / 2.0, 0.0);
                let node = Node::new(position, x == 0);

                if x == 1 {
                    cloth.flicker.insert(cloth.nodes.len());
                }

                cloth.mesh.vertices.push(Vertex {
                    position,
                    normal: -Vec3::Z,
                    uv: Vec2::ZERO,
                    color: Color::RED,
                });

                cloth.nodes.push(node);

                if x < width - 1 && y < height - 1 {
                    cloth.mesh.indices.push((y * width + x) as u32);
                    cloth.mesh.indices.push((y * width + x + 1) as u32);
                    cloth.mesh.indices.push(((y + 1) * width + x) as u32);

                    cloth.mesh.indices.push((y * width + x + 1) as u32);
                    cloth.mesh.indices.push(((y + 1) * width + x + 1) as u32);
                    cloth.mesh.indices.push(((y + 1) * width + x) as u32);
                }

                if x < width - 1 {
                    cloth
                        .connections
                        .insert((y * width + x, y * width + x + 1), 1.0);
                }

                if y < height - 1 {
                    cloth
                        .connections
                        .insert((y * width + x, (y + 1) * width + x), 1.0);
                }
            }
        }

        cloth.mesh.mark_changed();

        cloth
    }

    #[inline]
    pub fn update(&mut self, delta_time: f32, wind: Vec3, flicker: Vec2) {
        const GRAVITY: f32 = 9.81 / 8.0;

        for (i, node) in self.nodes.iter_mut().enumerate() {
            if !node.locked {
                let prev = node.position;
                node.position += node.position - node.prev_position;
                node.position += -Vec3::Y * GRAVITY * delta_time * delta_time;
                node.position += wind * delta_time;

                if self.flicker.contains(&i) {
                    node.position.x = flicker.x;
                    node.position.z = flicker.y;
                }

                node.prev_position = prev;
            }
        }

        for _ in 0..32 {
            for ((a, b), length) in &self.connections {
                let center = (self.nodes[*a].position + self.nodes[*b].position) / 2.0;
                let dir = (self.nodes[*a].position - self.nodes[*b].position).normalize();

                if !self.nodes[*a].locked {
                    self.nodes[*a].position = center + dir * *length / 2.0;
                }

                if !self.nodes[*b].locked {
                    self.nodes[*b].position = center - dir * *length / 2.0;
                }
            }
        }

        for (i, node) in self.nodes.iter().enumerate() {
            self.mesh.vertices[i].position = node.position * 2.0;
        }

        self.mesh.calculate_normals();

        self.mesh.mark_changed();
    }
}

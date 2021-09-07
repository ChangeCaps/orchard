use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use ike::{
    d2::{render::Render2dCtx, sprite::Sprite, transform2d::Transform2d},
    prelude::*,
};
use rand::{Rng, SeedableRng};

use crate::{assets::Assets, cloth::Cloth, config::Config, iso::from_iso, item::{ItemType, Items}, render::Ctx, tree::{Tree, TreeStage}};

#[derive(Debug)]
pub enum FarmPlant {
    Wheat { growth: f32 },
}

impl FarmPlant {
    #[inline]
    pub fn texture<'a>(&self, assets: &'a mut Assets, cfg: &Config, p: u64) -> &'a mut Texture {
        let mut rng = rand::rngs::StdRng::seed_from_u64(p);

        match self {
            FarmPlant::Wheat { growth, .. } => {
                match *growth + rng.gen_range(0.0..cfg.plants.wheat.growth_variance) {
                    x if x < 1.0 => &mut assets.wheat_0,
                    x if x < 2.0 => &mut assets.wheat_1,
                    x if x < 3.0 => &mut assets.wheat_2,
                    _ => &mut assets.wheat_3,
                }
            }
        } 
    }

    #[inline]
    pub fn harvestable(&self) -> bool {
        match self {
            FarmPlant::Wheat { growth } => *growth > 3.0,
        }
    }

    #[inline]
    pub fn update(&mut self, ctx: &mut UpdateCtx, cfg: &Config) {
        match self {
            FarmPlant::Wheat { growth } => {
                *growth += ctx.delta_time * (1.0 / cfg.plants.wheat.growth_time)
            }
        }
    }
}

pub enum Structure {
    Pole { cloth: Cloth, frames: u8, time: f32 },
    Tree(Tree),
}

impl Structure {
    #[inline]
    pub fn pole() -> Self {
        let mut rng = rand::thread_rng();

        Structure::Pole {
            cloth: Cloth::generate(15, 4),
            frames: 0,
            time: rng.gen_range(0.0..std::f32::consts::PI),
        }
    }

    #[inline]
    pub fn tree() -> Self {
        let mut tree = Tree::default();
        tree.trunk_color = Color8::srgb(180, 105, 43).into();
        tree.leaf_color = Color8::srgb(76, 193, 59).into();
        tree.trunk_radius = 4.0;
        tree.radius_decay = 0.8;
        tree.branch_length = 8.0;

        tree.generate_mesh_sapling();

        Structure::Tree(tree)
    }

    #[inline]
    pub fn update(&mut self, ctx: &mut UpdateCtx, cfg: &Config) {
        #[allow(unreachable_patterns)]
        match self {
            Structure::Pole {
                cloth,
                frames,
                time,
            } => {
                if cfg.graphics.instance_cloth {
                    return;
                }

                *frames += 1;

                *time += ctx.delta_time;

                if *frames >= 2 {
                    *frames = 0;

                    cloth.update(
                        ctx.delta_time * 2.0,
                        Vec3::new(-1.25, 0.0, -1.25),
                        Vec2::new((*time * 10.0).cos() - 1.0, (*time * 10.0).sin() - 1.0),
                    );
                }
            }
            Structure::Tree(tree) => tree.update(ctx),
            _ => {}
        }
    }
    
    #[inline]
    pub fn destroy(self, position: Vec2, _ctx: &mut UpdateCtx, items: &mut Items, _cfg: &Config) {
        #[allow(unreachable_patterns)]
        match self {
            Self::Pole { .. } => {
                items.spawn(ItemType::Pole, position, 1);
            },
            Self::Tree(tree) => {
                if let TreeStage::Grown = tree.stage {
                    items.spawn(ItemType::Wood, position + Vec2::new(-4.0, -2.0), 1);
                    items.spawn(ItemType::Sapling, position + Vec2::new(4.0, 2.0), 1);
                } else {
                    items.spawn(ItemType::Sapling, position, 1);
                }
            }
            _ => {},
        }
    }

    #[inline]
    pub fn texture<'a>(&self, assets: &'a mut Assets) -> Option<&'a mut Texture> {
        match self {
            Self::Pole { .. } => Some(&mut assets.pole),
            _ => None,
        }
    }

    #[inline]
    pub fn mesh_render(
        &self,
        ctx: &mut Ctx,
        position: Vec3,
        transform: &Transform3d,
        instanced_cloth: &Cloth,
        cfg: &Config,
    ) {
        match self {
            Self::Pole { cloth, .. } => {
                let transform = transform
                    * Transform3d::from_translation(position + Vec3::new(-5.0, 30.5, 0.0));

                if cfg.graphics.instance_cloth {
                    ctx.render_mesh(&instanced_cloth.mesh, transform.matrix());
                } else {
                    ctx.render_mesh(&cloth.mesh, transform.matrix());
                }
            }
            Self::Tree(tree) => {
                let transform =
                    transform * Transform3d::from_translation(position + Vec3::new(0.0, 0.0, 0.0));

                ctx.render_mesh(&tree.mesh, transform.matrix());
            }
        }
    }
}

pub enum Tile {
    Grass { structure: Option<Structure>, destruction: f32, },
    Farmed { time: f32, plant: Option<FarmPlant> },
}

impl Tile {
    #[inline]
    pub fn grass_plain() -> Self {
        Self::Grass { structure: None, destruction: 0.0 }
    }

    #[inline]
    pub fn grass() -> Self {
        let mut rng = rand::thread_rng();

        let structure = match rng.gen_range(0..100) {
            0 => Some(Structure::pole()),
            1..=5 => Some(Structure::tree()),
            _ => None,
        };

        Self::Grass { structure, destruction: 0.0 }
    }

    #[inline]
    pub fn texture<'a>(&self, assets: &'a mut Assets) -> &'a mut Texture {
        match self {
            Self::Grass { .. } => &mut assets.base_tile,
            Self::Farmed { .. } => &mut assets.farm_tile,
        }
    }

    #[inline]
    pub fn draw(&self, ctx: &mut Render2dCtx, tile_pos: Vec2, assets: &mut Assets, cfg: &Config) {
        match self {
            Self::Farmed {
                plant: Some(plant), ..
            } => {
                for x in 0..5 {
                    for y in 0..5 {
                        // calculate plant position
                        let plant_pos =
                            from_iso(Vec2::new(x as f32, y as f32) / 4.0 - 0.5, Vec2::splat(23.0));

                        // offset plant position by tile position
                        let pos = plant_pos + tile_pos;

                        // generate plant texture hash
                        let p = tile_pos.round().as_i32() + IVec2::new(y, x);

                        let mut hasher = DefaultHasher::default();

                        p.hash(&mut hasher);

                        let p = hasher.finish();

                        let texture = plant.texture(assets, cfg, p);

                        let sprite = Sprite {
                            view: texture
                                .texture(ctx.render_ctx)
                                .create_view(&Default::default()),
                            transform: Transform2d::from_translation(
                                pos + Vec2::new(0.0, texture.height() as f32 / 2.0),
                            )
                            .matrix(),
                            depth: -pos.y / 0.5f32.asin().tan(),
                            width: texture.width() as f32,
                            height: texture.height() as f32,
                            min: Vec2::ZERO,
                            max: Vec2::ONE,
                            texture_id: texture.id(),
                        };

                        ctx.draw_sprite(sprite);
                    }
                }
            }
            Self::Grass {
                structure: Some(structure),
                ..
            } => {
                let texture = structure.texture(assets);

                if let Some(texture) = texture {
                    let sprite = Sprite {
                        view: texture
                            .texture(ctx.render_ctx)
                            .create_view(&Default::default()),
                        transform: Transform2d::from_translation(tile_pos + Vec2::new(0.0, 14.0))
                            .matrix(),
                        depth: -(tile_pos.y - 2.0) / 0.5f32.asin().tan(),
                        width: texture.width() as f32,
                        height: texture.height() as f32,
                        min: Vec2::ZERO,
                        max: Vec2::ONE,
                        texture_id: texture.id(),
                    };

                    ctx.draw_sprite(sprite);
                }
            }
            _ => {}
        }
    }

    #[inline]
    pub fn render_mesh(
        &self,
        ctx: &mut Ctx,
        position: Vec3,
        transform: &Transform3d,
        instanced_cloth: &Cloth,
        cfg: &Config,
    ) {
        match self {
            Self::Grass {
                structure: Some(structure),
                ..
            } => {
                structure.mesh_render(ctx, position, transform, instanced_cloth, cfg);
            }
            _ => {}
        }
    }

    #[inline]
    pub fn hovered(
        &mut self,
        ctx: &mut UpdateCtx,
        cfg: &Config,
        position: Vec2,
        items: &mut Items,
    ) {
        match self {
            Self::Grass {
                structure, ..
            } if structure.is_none() => {
                if ctx.mouse_input.down(&cfg.controls.secondary) {
                    match items.drag_ty() {
                        None => {
                            let mut rng = rand::thread_rng();

                            if rng.gen_range(0..5) == 0 {
                                items.spawn(ItemType::WheatSeed, position, 1);
                            }

                            *self = Self::Farmed {
                                time: cfg.tile.grass_growth_time,
                                plant: None,
                            };
                        }
                        Some(ItemType::Pole) => {
                            items.consume();
                            *structure = Some(Structure::pole()); 
                        }
                        Some(ItemType::Sapling) => {
                            items.consume();
                            *structure = Some(Structure::tree());
                        }
                        _ => {}
                    }
                }
            }
            Self::Grass { structure, destruction } => {
                if ctx.mouse_input.pressed(&cfg.controls.secondary) {
                    *destruction += 1.0;
                }

                if *destruction > 3.0 && ctx.mouse_input.released(&cfg.controls.secondary) {
                    *destruction = 0.0;
                    structure.take().unwrap().destroy(position, ctx, items, cfg);
                }
            }
            Self::Farmed { plant, time, .. } => {
                if let Some(farm_plant) = plant {
                    if farm_plant.harvestable() && items.drag.is_none() {
                        if ctx.mouse_input.down(&cfg.controls.secondary) {
                            *time = cfg.tile.grass_growth_time;
                            *plant = None;

                            let mut rng = rand::thread_rng();

                            items.spawn(
                                ItemType::WheatSeed,
                                position + Vec2::new(-4.0, -2.0),
                                1 + rng.gen_range(0..=8) / 8,
                            );
                            items.spawn(ItemType::Wheat, position + Vec2::new(4.0, 2.0), 1);
                        }
                    }
                } else {
                    if let Some(&ItemType::WheatSeed) = items.drag_ty() {
                        if ctx.mouse_input.down(&cfg.controls.secondary) {
                            *plant = Some(FarmPlant::Wheat { growth: 0.0 });
                            items.consume();
                        }
                    }
                }
            }
            _ => {}
        }
    }

    #[inline]
    pub fn update(&mut self, ctx: &mut UpdateCtx, items: &mut Items, cfg: &Config) {
        match self {
            Self::Grass { structure, destruction } => {
                *destruction = (*destruction - ctx.delta_time).max(0.0);

                if let Some(s) = structure {
                    s.update(ctx, cfg); 
                }
            }
            Self::Farmed { time, plant } => {
                if let Some(plant) = plant {
                    plant.update(ctx, cfg);
                } else {
                    *time -= ctx.delta_time;

                    if *time <= 0.0 {
                        *self = Tile::grass();
                    }
                }
            }
        }
    }
}

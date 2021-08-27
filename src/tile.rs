use ike::{
    d2::{render::Render2dCtx, sprite::Sprite, transform2d::Transform2d},
    prelude::*,
};

use crate::{assets::Assets, config::Config, iso::from_iso};

#[derive(Debug)]
pub enum FarmPlant {
    Wheat { growth: f32 },
}

impl FarmPlant {
    #[inline]
    pub fn texture<'a>(&self, assets: &'a mut Assets, _p: u64) -> &'a mut Texture {
        match self {
            FarmPlant::Wheat { growth, .. } => match *growth {
                x if x < 1.0 => &mut assets.wheat_0,
                x if x < 2.0 => &mut assets.wheat_1,
                x if x < 3.0 => &mut assets.wheat_2,
                _ => &mut assets.wheat_3,
            },
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

pub enum Tile {
    Grass,
    Farmed { time: f32, plant: Option<FarmPlant> },
}

impl Tile {
    #[inline]
    pub fn texture<'a>(&self, assets: &'a mut Assets) -> &'a mut Texture {
        match self {
            Self::Grass => &mut assets.base_tile,
            Self::Farmed { .. } => &mut assets.farm_tile,
        }
    }

    #[inline]
    pub fn draw(&self, ctx: &mut Render2dCtx, tile_pos: Vec2, assets: &mut Assets, _cfg: &Config) {
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

                        let texture = plant.texture(assets, 0);

                        let sprite = Sprite {
                            view: texture
                                .texture(ctx.render_ctx)
                                .create_view(&Default::default()),
                            transform: Transform2d::from_translation(
                                pos + Vec2::new(0.0, texture.height as f32 / 2.0),
                            )
                            .matrix(),
                            depth: -(pos.y - 5.0) * 0.01,
                            width: texture.width,
                            height: texture.height,
                            min: Vec2::ZERO,
                            max: Vec2::ONE,
                            texture_id: texture.id,
                        };

                        ctx.draw_sprite(sprite);
                    }
                }
            }
            _ => {}
        }
    }

    #[inline]
    pub fn hovered(&mut self, ctx: &mut UpdateCtx, cfg: &Config) {
        match self {
            Self::Grass => {
                if ctx.mouse_input.down(&MouseButton::Right) {
                    *self = Self::Farmed {
                        time: cfg.tile.grass_growth_time,
                        plant: None,
                    };
                }
            }
            Self::Farmed { plant, .. } => {
                if let Some(farm_plant) = plant {
                    if farm_plant.harvestable() {
                        if ctx.mouse_input.down(&MouseButton::Right) {
                            *plant = None;
                        }
                    }
                } else {
                    if ctx.mouse_input.down(&MouseButton::Left) {
                        *plant = Some(FarmPlant::Wheat { growth: 0.0 });
                    }
                }
            }
        }
    }

    #[inline]
    pub fn update(&mut self, ctx: &mut UpdateCtx, cfg: &Config) {
        match self {
            Self::Farmed { time, plant } => {
                if let Some(plant) = plant {
                    plant.update(ctx, cfg);
                } else {
                    *time -= ctx.delta_time;

                    if *time <= 0.0 {
                        *plant = None;
                    }
                }
            }
            _ => {}
        }
    }
}

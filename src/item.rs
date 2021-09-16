use std::collections::HashMap;

use ike::prelude::*;

use crate::{assets::Assets, config::Config, game_state::GameState, iso::to_iso, tile::Tile};

#[derive(PartialEq, Eq)]
pub enum ItemType {
    WheatSeed,
    Wheat,
    Pole,
    Wood,
    Sapling,
}

impl ItemType {
    pub fn texture<'a>(&self, assets: &'a Assets) -> &'a Texture {
        match self {
            Self::WheatSeed => &assets.wheat_seed,
            Self::Wheat => &assets.wheat_item,
            Self::Pole => &assets.pole_item,
            Self::Wood => &assets.wood_item,
            Self::Sapling => &assets.sapling_item,
        }
    }
}

pub struct Item {
    pub position: Vec3,
    pub ty: ItemType,
    pub velocity: Vec3,
    pub count: u32,
}

pub struct Drag {
    pub id: Id<Item>,
    pub offset: Vec2,
}

#[derive(Default)]
pub struct Items {
    pub drag: Option<Drag>,
    pub items: HashMap<Id<Item>, Item>,
}

impl Items {
    #[inline]
    pub fn spawn(&mut self, ty: ItemType, position: Vec2, count: u32) -> Id<Item> {
        let id = Id::new();

        let item = Item {
            position: (position - Vec2::Y * 4.0).extend(8.0),
            ty,
            velocity: Vec3::Z * -32.0,
            count,
        };

        self.items.insert(id, item);
        id
    }

    #[inline]
    pub fn consume(&mut self) {
        if let Some(ref drag) = self.drag {
            let item = self.items.get_mut(&drag.id).unwrap();

            if item.count > 1 {
                item.count -= 1;
            } else {
                self.items.remove(&drag.id);
                self.drag = None;
            }
        }
    }

    #[inline]
    pub fn drag_ty(&self) -> Option<&ItemType> {
        Some(&self.items.get(&self.drag.as_ref()?.id)?.ty)
    }

    #[inline]
    pub fn drag_mut(&mut self) -> Option<&mut Item> {
        self.items.get_mut(&self.drag.as_ref()?.id)
    }

    #[inline]
    pub fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        tiles: &HashMap<IVec2, Tile>,
        mouse: Vec2,
        time: f32,
        cfg: &Config,
    ) {
        if !ctx.mouse_input.down(&cfg.controls.primary) {
            self.drag = None;
        }

        let mut despawn = Vec::new();

        for (id, item) in &mut self.items {
            let iso = to_iso(item.position.truncate(), Vec2::splat(40.0))
                .round()
                .as_i32();

            item.velocity.z -= 64.0 * ctx.delta_time;

            if tiles.contains_key(&iso) {
                let d = iso.x as f32 + iso.y as f32;

                let offset = (d * 2.0 + time * 0.5).sin();

                if item.position.z <= offset {
                    item.position.z = offset;
                    item.velocity.z *= -0.1;
                }
            } else {
                if item.velocity.z < -512.0 {
                    despawn.push(*id);
                }
            }

            item.position += item.velocity * ctx.delta_time;
        }

        for id in despawn {
            self.items.remove(&id);
        }

        if let Some(ref drag) = self.drag {
            let item = self.items.get_mut(&drag.id).unwrap();
            item.position = (mouse + drag.offset).extend(0.0);
            item.velocity.z = 0.0;

            let mut merge = Vec::new();
            let drag_item = self.items.get(&drag.id).unwrap();

            for (id, item) in &self.items {
                if *id == drag.id {
                    continue;
                }

                if drag_item.position.distance(item.position) < 8.0 && drag_item.ty == item.ty {
                    merge.push(*id);
                }
            }

            for id in merge {
                let item = self.items.remove(&id).unwrap();

                self.items.get_mut(&drag.id).unwrap().count += item.count;
            }
        } else {
            for (id, item) in &mut self.items {
                if mouse.x >= item.position.x - 8.0
                    && mouse.x <= item.position.x + 8.0
                    && mouse.y >= item.position.y + item.position.z
                    && mouse.y <= item.position.y + 16.0 + item.position.z
                    && ctx.mouse_input.pressed(&cfg.controls.primary)
                {
                    item.velocity.z = item.velocity.z.max(0.0);

                    let offset = if cfg.controls.item_offset {
                        item.position.truncate() - mouse
                    } else {
                        Vec2::Y * -8.0
                    };

                    self.drag = Some(Drag { id: *id, offset });
                }
            }
        }
    }

    #[inline]
    pub fn render(
        &self,
        ctx: &mut UpdateCtx,
        assets: &mut Assets,
        config: &Config,
    ) {
        for (_id, item) in &self.items {
            if item.count > 1 || config.graphics.always_show_stack_size {
                let mut text = TextSprite::new(
                    &assets.font,
                    Transform2d::from_translation(
                        item.position.truncate() + Vec2::Y * item.position.z + Vec2::Y * 18.0,
                    ),
                );

                text.depth = -item.position.y / 0.5f32.asin().tan();

                ctx.draw(&text);
            }

            let texture = item.ty.texture(assets);

            let position = item.position.truncate() + Vec2::Y * item.position.z;

            let mut sprite = Sprite::new(
                texture,
                Transform2d::from_translation(position + Vec2::Y * 8.0),
            );

            sprite.depth = -item.position.y / 0.5f32.asin().tan();

            ctx.draw(&sprite);
        }
    }
}

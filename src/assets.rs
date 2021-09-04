use ike::prelude::*;

pub struct Assets {
    pub font: Font,
    pub cursor: Texture,
    pub base_tile: Texture,
    pub farm_tile: Texture,
    pub wheat_seed: Texture,
    pub wheat_item: Texture,
    pub wheat_0: Texture,
    pub wheat_1: Texture,
    pub wheat_2: Texture,
    pub wheat_3: Texture,
    pub pole: Texture,
}

impl Assets {
    #[inline]
    pub fn load() -> ike::anyhow::Result<Self> {
        Ok(Self {
            font: Font::load("assets/font.ttf", 30.0)?,
            cursor: Texture::load("assets/cursor.png")?,
            base_tile: Texture::load("assets/base_tile.png")?,
            farm_tile: Texture::load("assets/farm_tile.png")?,
            wheat_seed: Texture::load("assets/items/wheat_seed.png")?,
            wheat_item: Texture::load("assets/items/wheat_item.png")?,
            wheat_0: Texture::load("assets/plants/wheat_0.png")?,
            wheat_1: Texture::load("assets/plants/wheat_1.png")?,
            wheat_2: Texture::load("assets/plants/wheat_2.png")?,
            wheat_3: Texture::load("assets/plants/wheat_3.png")?,
            pole: Texture::load("assets/structures/pole.png")?,
        })
    }
}

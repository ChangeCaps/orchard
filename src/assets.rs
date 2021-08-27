use ike::prelude::*;

pub struct Assets {
    pub cursor: Texture,
    pub base_tile: Texture,
    pub farm_tile: Texture,
    pub wheat_0: Texture,
    pub wheat_1: Texture,
    pub wheat_2: Texture,
    pub wheat_3: Texture,
}

impl Assets {
    #[inline]
    pub fn load() -> ike::anyhow::Result<Self> {
        Ok(Self {
            cursor: Texture::load("assets/cursor.png")?,
            base_tile: Texture::load("assets/base_tile.png")?,
            farm_tile: Texture::load("assets/farm_tile.png")?,
            wheat_0: Texture::load("assets/plants/wheat_0.png")?,
            wheat_1: Texture::load("assets/plants/wheat_1.png")?,
            wheat_2: Texture::load("assets/plants/wheat_2.png")?,
            wheat_3: Texture::load("assets/plants/wheat_3.png")?,
        })
    }
}

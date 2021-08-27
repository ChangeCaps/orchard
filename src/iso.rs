use ike::prelude::*;

#[inline]
pub fn to_iso(screen: Vec2, tile_size: Vec2) -> Vec2 {
    Vec2::new(
        (screen.x / (tile_size.x / 2.0) + screen.y / (tile_size.y / 4.0)) / 2.0,
        (screen.y / (tile_size.y / 4.0) - screen.x / (tile_size.x / 2.0)) / 2.0,
    )
}

#[inline]
pub fn from_iso(iso: Vec2, tile_size: Vec2) -> Vec2 {
    Vec2::new(
        (iso.x - iso.y) * tile_size.x / 2.0,
        (iso.x + iso.y) * tile_size.y / 4.0,
    )
}

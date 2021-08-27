use ike::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Controls {
    pub up: Key,
    pub down: Key,
    pub left: Key,
    pub right: Key,
    pub move_speed: f32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Tile {
    pub grass_growth_time: f32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Plants {
    pub wheat: Wheat,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Wheat {
    pub growth_time: f32,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub controls: Controls,
    pub tile: Tile,
    pub plants: Plants,
}

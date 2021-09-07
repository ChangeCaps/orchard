use ike::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Controls {
    pub up: Key,
    pub down: Key,
    pub left: Key,
    pub right: Key,
    pub toggle_fullscreen: Key,
    pub primary: MouseButton,
    pub secondary: MouseButton,
    pub camera_speed: f32,
    pub item_offset: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Window {
    pub cursor_grab: bool,
    pub maximized_default: bool,
    pub fullscreen_default: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Graphics {
    pub custom_cursor: bool,
    pub always_show_stack_size: bool,
    pub d3_scale: u32,
    pub instance_cloth: bool,
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
    pub growth_variance: f32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub controls: Controls,
    pub window: Window,
    pub graphics: Graphics,
    pub tile: Tile,
    pub plants: Plants,
}

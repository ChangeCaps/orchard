#![allow(dead_code)]

mod assets;
mod cloth;
mod config;
mod game_state;
mod iso;
mod item;
mod render;
mod tile;

use game_state::GameState;
use ike::{d2::render::SpriteNode2d, prelude::*};
use render::RenderNode;

const CLEAR_COLOR: Color = Color::rgb(123.0 / 255.0, 216.0 / 255.0, 213.0 / 255.0);

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Error)
        .init()
        .unwrap();

    let mut app = App::new();

    let mut main_pass = app.renderer.pass_mut::<MainPass>().unwrap();

    main_pass.push(SpriteNode2d::new());
    main_pass.push(RenderNode::default());
    main_pass.clear_color = CLEAR_COLOR;
    main_pass.sample_count = 4;

    app.run(GameState::load().unwrap())
}

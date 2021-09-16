#![allow(dead_code)]

mod assets;
mod audio;
mod cloth;
mod config;
mod game_state;
mod iso;
mod item;
mod render;
mod tile;
mod tree;

use game_state::GameState;
use ike::{d2::render::SpriteNode2d, d3::D3Node, prelude::*};
use render::{D3Pass, RenderNode};

const CLEAR_COLOR: Color = Color::rgb(123.0 / 255.0, 216.0 / 255.0, 213.0 / 255.0);

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Error)
        .init()
        .unwrap();

    let mut app = App::new();

    let mut d3_pass = Pass::new(D3Pass::default());

    d3_pass.push(D3Node::default());

    app.renderer.push(d3_pass);

    let mut main_pass = MainPass::default();

    main_pass.clear_color = CLEAR_COLOR;
    main_pass.sample_count = 4;

    let mut main_pass = Pass::new(main_pass);

    main_pass.push(RenderNode::default());
    main_pass.push(SpriteNode2d::new());

    #[cfg(debug_assertions)]
    main_pass.push(DebugNode::default());

    app.renderer.push(main_pass);

    app.run(GameState::load().unwrap())
}

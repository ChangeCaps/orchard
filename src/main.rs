mod assets;
mod config;
mod game_state;
mod iso;
mod tile;

use game_state::GameState;
use ike::{d2::render::Node2d, prelude::*};

const CLEAR_COLOR: Color = Color::rgb(123.0 / 255.0, 216.0 / 255.0, 213.0 / 255.0);

fn main() {
    let mut app = App::new();

    app.renderer.add_node(Node2d::new(CLEAR_COLOR, 4));

    app.run(GameState::load().unwrap())
}

use std::{collections::HashMap, fs::read_to_string};

use ike::prelude::*;
use kira::manager::AudioManager;

use crate::{
    assets::Assets,
    audio::Audio,
    cloth::Cloth,
    config::Config,
    iso::{from_iso, to_iso},
    item::Items,
    tile::Tile,
};

pub struct OrthographicCamera {
    pub projection: OrthographicProjection,
    pub transform: Transform2d,
}

impl OrthographicCamera {
    #[inline]
    pub fn new() -> Self {
        Self {
            projection: OrthographicProjection {
                size: 256.0,
                ..Default::default()
            },
            transform: Transform2d::IDENTITY,
        }
    }

    #[inline]
    pub fn id(&self) -> Id<Camera> {
        self.projection.id
    }

    #[inline]
    pub fn camera(&self) -> Camera {
        Camera {
            id: self.projection.id(),
            position: self.transform.translation.extend(0.0),
            proj: self.projection.proj_matrix(),
            view: self.transform.matrix4x4(),
        }
    }
}

pub struct GameState {
    pub assets: Assets,
    pub audio: Audio,
    pub audio_manager: AudioManager,
    pub d3_buffer: FrameBuffer,
    pub config: Config,
    pub cloth: Cloth,
    pub main_camera: OrthographicCamera,
    pub items: Items,
    pub tiles: HashMap<IVec2, Tile>,
    pub time: f32,
    pub mouse_position: Vec2,
}

impl State for GameState {
    fn start(&mut self, ctx: &mut StartCtx) {
        ctx.window.maximized = self.config.window.maximized_default;
        ctx.window.cursor_visible = !self.config.graphics.custom_cursor;
        ctx.window.fullscreen = self.config.window.fullscreen_default;
        ctx.window.title = String::from("Orchard");
    }

    fn update(&mut self, ctx: &mut UpdateCtx) {
        // scale camera to screen
        self.main_camera.projection.scale(ctx.window.size);

        // advance time
        self.time += ctx.delta_time;

        if ctx
            .key_input
            .pressed(&self.config.controls.toggle_fullscreen)
        {
            ctx.window.fullscreen = !ctx.window.fullscreen;

            if self.config.window.cursor_grab {
                ctx.window.cursor_grab = ctx.window.fullscreen;
            }
        }

        // move camera with keys
        let mut camera_movement = Vec2::ZERO;

        if ctx.key_input.down(&self.config.controls.left) {
            camera_movement -= Vec2::X;
        }

        if ctx.key_input.down(&self.config.controls.right) {
            camera_movement += Vec2::X;
        }

        if ctx.key_input.down(&self.config.controls.down) {
            camera_movement -= Vec2::Y;
        }

        if ctx.key_input.down(&self.config.controls.up) {
            camera_movement += Vec2::Y;
        }

        if camera_movement != Vec2::ZERO {
            self.main_camera.transform.translation +=
                camera_movement.normalize() * ctx.delta_time * self.config.controls.camera_speed;
        }

        // move camera by mouse
        let mut delta = (ctx.mouse.position - ctx.mouse.prev_position) / ctx.window.size.y as f32
            * self.main_camera.projection.size;
        delta.x *= -1.0;

        if ctx.mouse_input.down(&MouseButton::Middle) {
            self.main_camera.transform.translation += delta;
        }

        // select tile with mouse
        let mut mouse_pos =
            (ctx.mouse.position - ctx.window.size.as_f32() / 2.0) / ctx.window.size.y as f32;
        mouse_pos.y *= -1.0;

        self.mouse_position =
            mouse_pos * self.main_camera.projection.size + self.main_camera.transform.translation;

        let mouse = to_iso(self.mouse_position, Vec2::splat(40.0));

        // update tiles
        for (position, tile) in self.tiles.iter_mut() {
            // if tile hovered
            if mouse.x > position.x as f32 - 0.5
                && mouse.x < position.x as f32 + 0.5
                && mouse.y > position.y as f32 - 0.5
                && mouse.y < position.y as f32 + 0.5
            {
                let position = from_iso(position.as_f32(), Vec2::splat(40.0));

                tile.hovered(
                    ctx,
                    &self.config,
                    &mut self.audio,
                    position,
                    &mut self.items,
                );
            }

            tile.update(ctx, &mut self.items, &self.config);
        }

        self.items.update(
            ctx,
            &self.tiles,
            self.mouse_position,
            self.time,
            &self.config,
        );

        if self.config.graphics.instance_cloth {
            self.cloth.update(
                ctx.delta_time,
                Vec3::new(-1.25, 0.0, -1.25),
                Vec2::new(
                    (self.time * 10.0).cos() - 1.0,
                    (self.time * 10.0).sin() - 1.0,
                ),
            );
        }

        self.items
            .render(ctx, &mut self.assets, &self.config);

        if self.config.graphics.custom_cursor {
            let mut sprite = Sprite::new(
                &self.assets.cursor,
                Transform2d::from_translation(self.mouse_position),
            );
            sprite.depth = 500.0;

            ctx.draw(&sprite);
        }

        // draw tiles
        for (position, tile) in &self.tiles {
            let d = position.x as f32 + position.y as f32;

            // calculate tile floating offset
            let offset = (d * 2.0 + self.time * 0.5).sin();

            // convert from isometric to cartesian
            let mut tile_pos = from_iso(position.as_f32(), Vec2::splat(40.0));
            tile_pos += Vec2::new(0.0, offset);

            let texture = tile.texture(&mut self.assets);

            let mut sprite = Sprite::new(
                texture,
                Transform2d::from_translation(tile_pos + Vec2::new(0.0, -8.0)),
            );

            sprite.depth = -(tile_pos.y + 8.0) / 0.5f32.asin().tan();

            ctx.draw(&sprite); 

            // draw plants on tile
            tile.draw(
                ctx,
                tile_pos,
                &mut self.assets,
                &self.config,
            );
        }

        // 3d
        let mut transform = Transform3d::IDENTITY;
        transform.rotation = Quat::from_rotation_x(0.5f32.asin());
        transform.rotation *= Quat::from_rotation_y(std::f32::consts::FRAC_PI_4);

        for (position, tile) in &self.tiles {
            let d = position.x as f32 + position.y as f32;

            // calculate tile floating offset
            let offset = (d * 2.0 + self.time * 0.5).sin() * 1.0;

            let position = Vec3::new(
                position.x as f32 * 40.0 * std::f32::consts::FRAC_1_SQRT_2,
                offset,
                -position.y as f32 * 40.0 * std::f32::consts::FRAC_1_SQRT_2,
            );

            tile.render_mesh(
                ctx,
                position,
                &transform,
                &self.cloth,
                &self.config,
            );
        }

        ctx.views.render_main_view(
            self.main_camera.camera(),
        );
    }
}

impl GameState {
    pub fn load() -> ike::anyhow::Result<Self> {
        let mut audio_manager = AudioManager::new(Default::default())?;
        let mut tiles = HashMap::new();

        for x in -1..=1 {
            for y in -1..=1 {
                tiles.insert(IVec2::new(x, y), Tile::grass_plain());
            }
        }

        let config = read_to_string("./config.toml")?;

        let audio = Audio::load(&mut audio_manager)?;
        let assets = Assets::load()?;

        Ok(Self {
            assets,
            audio,
            audio_manager,
            d3_buffer: Default::default(),
            config: toml::from_str(&config)?,
            cloth: Cloth::generate(15, 4),
            main_camera: OrthographicCamera::new(),
            items: Default::default(),
            tiles,
            time: 0.0,
            mouse_position: Default::default(),
        })
    }
}

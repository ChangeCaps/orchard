use std::{collections::HashMap, fs::read_to_string};

use ike::{
    d2::{
        render::{Render2d, Render2dCtx},
        sprite::Sprite,
        transform2d::Transform2d,
    },
    prelude::*,
};

use crate::{
    assets::Assets,
    cloth::Cloth,
    config::Config,
    iso::{from_iso, to_iso},
    render::{Ctx, Mesh, Vertex},
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
    pub fn view_proj(&self) -> Mat4 {
        self.projection.proj_matrix() * self.transform.matrix4x4().inverse()
    }
}

pub struct GameState {
    pub assets: Assets,
    pub config: Config,
    pub cloth: Cloth,
    pub main_camera: OrthographicCamera,
    pub tiles: HashMap<IVec2, Tile>,
    pub time: f32,
    pub mouse_position: Vec2,
}

impl Render2d for GameState {
    fn render(&mut self, ctx: &mut Render2dCtx) {
        ctx.draw_texture_depth(
            &mut self.assets.cursor,
            &Transform2d::from_translation(self.mouse_position),
            500.0,
        );

        // draw tiles
        for (position, tile) in &self.tiles {
            let d = position.x as f32 + position.y as f32;

            // calculate tile floating offset
            let offset = (d * 2.0 + self.time * 0.5).sin();

            // convert from isometric to cartesian
            let mut tile_pos = from_iso(position.as_f32(), Vec2::splat(40.0));
            tile_pos += Vec2::new(0.0, offset);

            let texture = tile.texture(&mut self.assets);

            let sprite = Sprite {
                view: texture
                    .texture(ctx.render_ctx)
                    .create_view(&Default::default()),
                // offset tile by 8 pixels so it lines up correctly
                transform: Transform2d::from_translation(tile_pos + Vec2::new(0.0, -8.0)).matrix(),
                depth: -(tile_pos.y + 8.0) * 0.01,
                width: texture.width,
                height: texture.height,
                min: Vec2::ZERO,
                max: Vec2::ONE,
                texture_id: texture.id,
            };

            ctx.draw_sprite(sprite);

            // draw plants on tile
            tile.draw(ctx, tile_pos, &mut self.assets, &self.config);
        }
    }
}

impl State for GameState {
    fn start(&mut self, ctx: &mut StartCtx) {
        ctx.window.maximized = true;
        //ctx.window.cursor_visible = false;
    }

    fn update(&mut self, ctx: &mut UpdateCtx) {
        // scale camera to screen
        self.main_camera.projection.scale(ctx.window.size);

        // advance time
        self.time += ctx.delta_time;

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
                camera_movement.normalize() * ctx.delta_time * self.config.controls.move_speed;
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
                tile.hovered(ctx, &self.config);
            }

            tile.update(ctx, &self.config);
        }

        self.cloth.update(
            ctx.delta_time,
            Vec3::new(-1.25, 0.0, -1.25),
            Vec2::new((self.time * 10.0).cos() - 1.0, (self.time * 10.0).sin() - 1.0),
        );
    }

    #[inline]
    fn render(&mut self, views: &mut Views) {
        views.render_main_view(self.main_camera.id(), self.main_camera.view_proj());
    }
}

impl GameState {
    pub fn load() -> ike::anyhow::Result<Self> {
        let mut tiles = HashMap::new();

        for x in -1..=1 {
            for y in -1..=1 {
                tiles.insert(IVec2::new(x, y), Tile::Grass);
            }
        }

        let config = read_to_string("./config.toml")?;

        Ok(Self {
            assets: Assets::load()?,
            config: toml::from_str(&config)?,
            cloth: Cloth::generate(20, 5),
            main_camera: OrthographicCamera::new(),
            tiles,
            time: 0.0,
            mouse_position: Default::default(),
        })
    }

    #[inline]
    pub fn render(&mut self, ctx: &mut Ctx) {
        let mut transform = Transform3d::IDENTITY;
        transform.rotation = Quat::from_rotation_x(std::f32::consts::FRAC_PI_4);
        transform.rotation *= Quat::from_rotation_y(std::f32::consts::FRAC_PI_4);

        let mut cloth_transform = Transform3d::from_xyz(0.0, 10.0, 0.0);
        cloth_transform = &transform * cloth_transform; 

        ctx.render_mesh(&self.cloth.mesh, cloth_transform.matrix());
    }
}

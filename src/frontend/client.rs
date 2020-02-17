use crate::geom::{Point, Rect, Vector};
use legion::systems::resource::Fetch;
use crate::frontend::grid::Grid;
use crate::frontend::glyph::Glyph;
use crate::frontend::{tileset, grid};
use crate::frontend::tileset::Tileset;
use crate::frontend::camera::render_camera;
use crate::frontend::camera::get_screen_bounds;
use crate::frontend::ui::{draw_box, print_glyphs};

use crate::client::network_client::NetworkClient;
use crate::component;
use crate::component::{Name, Position};
use crate::message::Message;
use crate::server::turn_system;
use crate::resources::log::GameLog;

use quicksilver::graphics::{Graphics, Color};
use quicksilver::lifecycle::{Key, EventStream, Window, Event, ElementState};

use legion::prelude::*;
use crate::server::map::Map;


pub struct RenderContext {
    map_region: Rect,
    tile_ctx: TileContext,
    screen_size: Vector,
    mouse_position: Vector,
    focus: Vector,
    gfx: Graphics,
    window: Window,
}

impl RenderContext {
    pub fn draw(&mut self, glyph: &Glyph, position: impl Into<Point>) {
        self.tile_ctx.draw(&mut self.gfx, glyph, position);
    }

    pub fn show(&mut self) {
        self.gfx.present(&self.window).expect("Failed to present");
    }
}

pub struct Client {
    log: GameLog,
    events: EventStream,
    pub(crate) render_context: RenderContext,
    pub network_client: NetworkClient,
}

impl Client {
    pub async fn new(window: Window, mut gfx: Graphics, mut events: EventStream) -> Self {
        let x = 80;
        let y = 50;
        let tileset = tileset::Tileset::from_font(&gfx, "Px437_Wyse700b-2y.ttf", 16.0 / 8.0)
            .await
            .expect("oof");
        let grid = grid::Grid::from_screen_size((x, y), (800, 600));
        let screen_size = Vector::new(x, y);
        let tile_ctx = TileContext { tileset, grid };
        let map_region = Rect::new((0, 0).into(), (x, y - 8).into());
        Client {
            events,
            log: GameLog::with_length(5),
            render_context: RenderContext {
                window,
                map_region,
                tile_ctx,
                screen_size,
                gfx,
                mouse_position: (0, 0).into(),
                focus: (0, 0).into(),
            },
            network_client: NetworkClient::new()
        }
    }

    pub fn sync(&mut self) {
        let mut query = <(
            Read<component::Player>,
            Read<component::Position>)>::query();
        for (_, position) in query.iter(self.network_client.world()) {
            self.render_context.focus = (position.x, position.y).into();
        }

    }

    pub async fn tick(&mut self) {
        while let Some(event) = self.events.next_event().await {
            self.handle_event(event);
        }
    }

    pub fn process_messages(&mut self, messages: Vec<Message>) {
        for message in messages {
            match message {
                Message::GameEvent(msg, fg, bg) => self.log.push(
                    msg.as_str(),
                    fg,
                    bg,
                ),
                _ => unimplemented!()
            }
        }
    }

    pub fn handle_event(&mut self, event: Event) -> bool {
        match event {
            Event::KeyboardInput { key, state } => {
                self.handle_key(key, state);
                true
            }
            Event::MouseMoved { pointer: _, position } => {
                let scale = self.render_context.window.scale_factor();
                self.render_context.mouse_position.x = position.x as i32 / scale as i32;
                self.render_context.mouse_position.y = position.y as i32 / scale as i32;
                false
            }
            Event::MouseInput {
                pointer: _,
                state,
                button: _,
            } => {
                if state == ElementState::Pressed {
                    let pos = self.render_context.tile_ctx.grid.point_to_grid(self.render_context.mouse_position);
                    self.handle_click(pos);
                }
                state == ElementState::Pressed
            }
            _ => false,
        }
    }

    pub fn handle_key(&mut self, key: Key, state: ElementState) {
        if state == ElementState::Pressed {
            match key {
                Key::W => self.handle_move((0, -1)),
                Key::A => self.handle_move((-1, 0)),
                Key::S => self.handle_move((0, 1)),
                Key::D => self.handle_move((1, 0)),
                _ => {}
            }
        }
    }

    pub fn handle_move(&mut self, delta: impl Into<Vector>) {
        let delta = delta.into();
        if self.network_client.try_move_player(delta) {
            self.render_context.focus += delta;
        }
    }


    pub fn handle_click(&mut self, point: impl Into<Point>) {
        let point = point.into();
        let (min_x, _max_x, min_y, _max_y) = get_screen_bounds(self);
        let point: Point = (
            point.x + min_x + self.render_context.map_region.origin.x,
            point.y + min_y + self.render_context.map_region.origin.y,
        )
            .into();
        let mut query = <(
            Read<component::Name>,
            Read<component::Position>)>::query();
        for (name, position) in query.iter(self.network_client.world()) {
            if position.x == point.x && position.y == point.y {
                self.log.push(
                    &format!("You clicked on {}", name.name),
                    Some(Color::GREEN),
                    None,
                );
            }
        }
    }

    pub fn render(&mut self) {
        let (x, y) = self.render_context.screen_size.to_tuple();
        self.render_context.gfx.clear(Color::BLACK);
        draw_box(
                &mut self.render_context,
            Rect::new((0, y - 7).into(), (x - 1, 6).into()),
            None,
            None,
        );
        for (index, glyphs) in self.log.iter().enumerate() {
            print_glyphs(&mut self.render_context, &glyphs, (1, (y - 6 + index as i32) as i32));
        }

        render_camera(self);

        self.render_context.show();
    }

    pub fn focus(&self) -> Vector {
        self.render_context.focus
    }

    pub fn mouse_position(&self) -> Vector {
        self.render_context.mouse_position
    }

    pub fn resources(&self) -> &Resources {
        self.network_client.resources()
    }

    pub fn map_region(&self) -> Rect {
        self.render_context.map_region
    }

    pub fn tile_context(&self) -> &TileContext {
        &self.render_context.tile_ctx
    }
}

pub struct TileContext {
    pub grid: Grid,
    pub tileset: Tileset,
}

impl TileContext {
    pub fn draw(&self, gfx: &mut Graphics, glyph: &Glyph, pos: impl Into<Point>) {
        let rect = self.grid.rect(pos);
        self.tileset.draw(gfx, &glyph, rect);
    }
}


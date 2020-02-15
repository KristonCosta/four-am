use crate::client::grid::Grid;
use crate::client::tileset::Tileset;
use quicksilver::graphics::{Graphics, Color};
use crate::client::glyph::Glyph;
use crate::geom::{Point, Rect, Vector};
use specs::{World, WorldExt};
use quicksilver::lifecycle::{Key, EventStream, Window, Event, ElementState};
use crate::client::{tileset, grid};
use crate::client::ui::{draw_box, print_glyphs};
use crate::client::camera::render_camera;
use crate::server::turn_system;
use crate::resources::log::GameLog;
use crate::message::Message;
use crate::client::camera::get_screen_bounds;
use crate::component;
use crate::component::{Name, Position};
use crate::server::gamestate::{try_move_player, move_player, clear_pickable};
use specs::join::Join;



pub struct Client {
    window: Window,
    gfx: Graphics,
    events: EventStream,
    map_region: Rect,
    tile_ctx: TileContext,
    screen_size: Vector,
    log: GameLog,
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
            window,
            gfx,
            events,
            map_region,
            tile_ctx,
            screen_size,
            log: GameLog::with_length(5)
        }
    }

    pub async fn tick(&mut self, ecs: &mut World) {
        while let Some(event) = self.events.next_event().await {
            self.handle_event(ecs, event);
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

    pub fn handle_event(&mut self, ecs: &mut World, event: Event) -> bool {
        match event {
            Event::KeyboardInput { key, state } => {
                self.handle_key(ecs, key, state);
                true
            }
            Event::MouseMoved { pointer: _, position } => {
                let scale = self.window.scale_factor();

                let mut mouse = ecs.write_resource::<MouseState>();
                mouse.x = position.x as i32 / scale as i32;
                mouse.y = position.y as i32 / scale as i32;
                false
            }
            Event::MouseInput {
                pointer: _,
                state,
                button: _,
            } => {
                if state == ElementState::Pressed {
                    let pos;
                    {
                        let mouse = ecs.fetch::<MouseState>();
                        pos = self.tile_ctx.grid.point_to_grid((mouse.x, mouse.y));
                    }

                    self.handle_click(ecs,pos);
                }
                state == ElementState::Pressed
            }
            _ => false,
        }
    }

    pub fn handle_key(&mut self, ecs: &mut World, key: Key, state: ElementState) {
        if state == ElementState::Pressed {
            match key {
                Key::W => try_move_player(0, -1, ecs),
                Key::A => try_move_player(-1, 0, ecs),
                Key::S => try_move_player(0, 1, ecs),
                Key::D => try_move_player(1, 0, ecs),
                _ => {}
            }
        }
    }


    pub fn handle_click(&mut self, ecs: &mut World, point: impl Into<Point>) {
        let point = point.into();
        let (min_x, _max_x, min_y, _max_y) = get_screen_bounds(&ecs, &self.tile_ctx);
        let point: Point = (
            point.x + min_x + self.map_region.origin.x,
            point.y + min_y + self.map_region.origin.y,
        )
            .into();
        let names = ecs.read_storage::<Name>();
        let positions = ecs.read_storage::<Position>();
        for (name, position) in (&names, &positions).join() {
            if position.x == point.x && position.y == point.y {
                self.log.push(
                    &format!("You clicked on {}", name.name),
                    Some(Color::GREEN),
                    None,
                );
            }
        }
    }

    pub fn render(&mut self, ecs: &mut World) {
        let gfx = &mut self.gfx;
        let tile_ctx = &self.tile_ctx;
        let map_region = &self.map_region;
        let window = &self.window;
        let (x, y) = self.screen_size.to_tuple();
        gfx.clear(Color::BLACK);
        draw_box(
            gfx,
            tile_ctx,
            Rect::new((0, y - 7).into(), (x - 1, 6).into()),
            None,
            None,
        );
        for (index, glyphs) in self.log.iter().enumerate() {
            print_glyphs(gfx, &self.tile_ctx, &glyphs, (1, (y - 6 + index as i32) as i32));
        }

        render_camera(ecs,  map_region, tile_ctx, gfx);

        gfx.present(window).expect("Failed to present");
    }
}


pub struct MouseState {
    pub x: i32,
    pub y: i32,
}

pub struct Focus {
    pub x: i32,
    pub y: i32,
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


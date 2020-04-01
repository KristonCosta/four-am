use crate::frontend::camera::render_camera;
use crate::frontend::glyph::Glyph;
use crate::frontend::grid::Grid;
use crate::frontend::tileset::Tileset;
use crate::frontend::ui::{print_glyphs, draw_ui};
use crate::frontend::{grid, tileset};
use crate::geom::{Point, Rect, Vector};

use crate::client::network_client::{NetworkClient, WorldType};
use crate::component;
use crate::message::Message;
use crate::resources::log::GameLog;

use quicksilver::graphics::{Color, Graphics};
use quicksilver::lifecycle::{Event, EventStream, Key, Window};

use legion::prelude::*;

pub struct MapRegion {
    pub(crate) region: Rect,
    focus: Point,
}

impl MapRegion {
    pub fn project(&self, point: Point) -> Point {
        let (min_x, _, min_y, _) = self.get_screen_bounds();
        let x = point.x - min_x + self.region.origin.x;
        let y = point.y - min_y + self.region.origin.y;
        (x, y).into()
    }

    pub fn unproject(&self, point: Point) -> Point {
        let (min_x, _, min_y, _) = self.get_screen_bounds();
        let x = point.x + min_x - self.region.origin.x;
        let y = point.y + min_y - self.region.origin.y;
        (x, y).into()
    }

    pub fn get_screen_bounds(&self) -> (i32, i32, i32, i32) {
        let focus = self.focus;
        let (x_chars, y_chars) = self.region.size.to_tuple();   //.grid.size.to_tuple();

        let center_x = x_chars / 2;
        let center_y = y_chars / 2;

        let min_x = focus.x - center_x;
        let max_x = min_x + x_chars;

        let min_y = focus.y - center_y;
        let max_y = min_y + y_chars;
        (min_x, max_x, min_y, max_y)
    }

}

pub struct RenderContext {
    map_region: MapRegion,
    tile_ctx: TileContext,
    screen_size: Vector,
    mouse_position: Vector,
    pub(crate) targeted_entity: Option<Entity>,
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
    pub async fn new(window: Window, gfx: Graphics, events: EventStream) -> Self {
        let x = 80;
        let y = 60;
        let tileset = tileset::Tileset::from_font(&gfx, "Px437_Wyse700b-2y.ttf", 16.0 / 8.0)
            .await
            .expect("oof");
        let grid = grid::Grid::from_screen_size((x, y), (1200, 900));
        let screen_size = Vector::new(x, y);
        let tile_ctx = TileContext { tileset, grid };
        let map_region = Rect::new((1, 1).into(), (48, 44).into());
        Client {
            events,
            log: GameLog::with_length(5),
            render_context: RenderContext {
                window,
                map_region: MapRegion {
                    region: map_region,
                    focus: (x / 2, y / 2).into()
                },
                tile_ctx,
                screen_size,
                gfx,
                targeted_entity: None,
                mouse_position: (0, 0).into()
            },
            network_client: NetworkClient::new(),
        }
    }

    pub fn sync(&mut self) {
        let query = <Read<component::Position>>::query()
            .filter(tag::<component::Player>());
        println!("Syncing");
        for position in query.iter(self.network_client.world()) {
            println!("Found player");
            self.render_context.map_region.focus = (position.x, position.y).into();
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
                Message::GameEvent(msg, fg, bg) => self.log.push(msg.as_str(), fg, bg),
            }
        }
    }

   // #[cfg(cargo_web)]
    pub fn handle_pointer_moved(&mut self,  x: i32, y: i32) -> bool {
        let scale = self.render_context.window.scale_factor();
        self.render_context.mouse_position.x = x as i32 * scale as i32; // / scale as i32;
        self.render_context.mouse_position.y = y as i32 * scale as i32; // / scale as i32;
        false
    }
/*
    #[cfg(not(cargo_web))]
    pub fn handle_pointer_moved(&mut self, x: i32, y: i32) -> bool {
        self.render_context.mouse_position.x = x as i32; // / scale as i32;
        self.render_context.mouse_position.y = y as i32; // / scale as i32;
        false
    }
*/
    pub fn handle_event(&mut self, event: Event) -> bool {
        match event {
            Event::KeyboardInput(event) => {
                self.handle_key(event.key(), event.is_down());
                true
            }
            Event::PointerMoved(event) => {
                let location = event.location();
                self.handle_pointer_moved(location.x as i32, location.y as i32)
            }
            Event::PointerInput(event) => {
                if event.is_down() {
                    let pos = self
                        .render_context
                        .tile_ctx
                        .grid
                        .point_to_grid(self.render_context.mouse_position);
                    self.handle_click(pos);
                }
                event.is_down()
            }
            _ => false,
        }
    }

    pub fn handle_key(&mut self, key: Key, is_down: bool) {
        if is_down {
            match key {
                Key::W => self.handle_move((0, -1)),
                Key::A => self.handle_move((-1, 0)),
                Key::S => self.handle_move((0, 1)),
                Key::D => self.handle_move((1, 0)),
                Key::Up => self.handle_focus((0, -1)),
                Key::Left => self.handle_focus((-1, 0)),
                Key::Down => self.handle_focus((0, 1)),
                Key::Right => self.handle_focus((1, 0)),
                Key::Q => self.network_client.reload_world(WorldType::Drunken),
                Key::E => self.network_client.reload_world(WorldType::Room),
                Key::C => self.sync(),
                Key::Space => self.handle_move((0, 0)),
                _ => {}
            }
        }
    }


    pub fn handle_focus(&mut self, delta: impl Into<Vector>) {
        self.render_context.map_region.focus += delta.into();
    }

    pub fn handle_move(&mut self, delta: impl Into<Vector>) {
        let delta = delta.into();
        if self.network_client.try_move_player(delta) {
            self.render_context.map_region.focus += delta;
        }
    }

    pub fn handle_click(&mut self, point: impl Into<Point>) {
        let point = self.map_region().unproject(point.into());
        let query = <(Read<component::Name>, Read<component::Position>)>::query();
        let mut found = false;
        for (entity, (name, position)) in query.iter_entities(self.network_client.world()) {
            if position.x == point.x && position.y == point.y {
                self.render_context.targeted_entity = Some(entity.clone());
                self.log.push(
                    &format!("You clicked on {}", name.name),
                    Some(Color::GREEN),
                    None,
                );
                found = true;
            }
        }
        if !found {
            self.render_context.targeted_entity = None;
        }
    }

    pub fn render(&mut self) {
        let (_, y) = self.render_context.screen_size.to_tuple();
        self.render_context.gfx.clear(Color::BLACK);
        draw_ui(&mut self.render_context, &self.network_client.world());

        for (index, glyphs) in self.log.iter().enumerate() {
            print_glyphs(
                &mut self.render_context,
                &glyphs,
                (1, (y - 6 + index as i32) as i32),
            );
        }

        render_camera(self);

        self.render_context.show();
    }

    pub fn focus(&self) -> Point {
        self.render_context.map_region.focus
    }

    pub fn mouse_position(&self) -> Vector {
        self.render_context.mouse_position
    }

    pub fn resources(&self) -> &Resources {
        self.network_client.resources()
    }

    pub fn map_region(&self) -> &MapRegion {
        &self.render_context.map_region
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

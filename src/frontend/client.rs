use crate::frontend::camera::Camera;
use crate::frontend::glyph::Glyph;
use crate::frontend::screen::grid::Grid;
use crate::frontend::tileset;
use crate::frontend::tileset::Tileset;
use crate::frontend::ui::{draw_ui};
use crate::geom::{Point, Vector};

use crate::client::network_client::NetworkClient;
use crate::component;
use crate::message::Message;
use crate::{map::Map, resources::log::GameLog};

use quicksilver::graphics::{Color, Graphics};
use quicksilver::lifecycle::{Event, EventStream, Key, Window};

use super::screen::terminal::Terminal;
use legion::prelude::*;

pub struct RenderContext {
    tile_ctx: TileContext,
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

pub struct LayoutManager {
    pub main: Terminal,
    pub map: Terminal,
    pub log: Terminal,
    pub player: Terminal,
    pub status: Terminal,
    pub overlay: Terminal,
}

impl LayoutManager {
    pub fn render(&mut self, context: &mut RenderContext) {
        self.main.blit(&mut self.map);
        self.main.blit(&mut self.log);
        self.main.blit(&mut self.player);
        self.main.blit(&mut self.status);
        self.main.blit(&mut self.overlay);
        self.main.render(context);
    }
}

pub enum UIMode {
    None,
    Interact
}

pub struct Client {
    log: GameLog,
    events: EventStream,
    pub(crate) render_context: RenderContext,
    pub network_client: NetworkClient,
    layout: LayoutManager,
    camera: Camera,
    mode: UIMode,
}

impl Client {
    pub async fn new(window: Window, gfx: Graphics, events: EventStream) -> Self {
        let x = 110;
        let y = 90;
        let tileset = tileset::Tileset::from_font(&gfx, "Px437_Wyse700b-2y.ttf", 16.0 / 8.0)
            .await
            .expect("oof");
        let grid = Grid::from_screen_size((x, y), (1100, 900));
        let dimensions = (x, y);
        let tile_ctx = TileContext { tileset, grid };

        let main = Terminal::new(dimensions);
        let map = main.subterminal((0, 0), (91, 81));
        let log = main.subterminal((0, 80), (110, 10));
        let player = main.subterminal((90, 0), (20, 10));
        let status = main.subterminal((90, 9), (20, 50));
        let mut overlay = main.subterminal(main.region.origin, main.region.size);
        overlay.min_layer = 1;
        overlay.num_layers = 1;
        let layout = LayoutManager {
            main,
            map,
            log,
            player,
            status,
            overlay,
        };

        Client {
            events,
            log: GameLog::with_length(30),
            render_context: RenderContext {
                window,
                tile_ctx,
                gfx,
                targeted_entity: None,
                mouse_position: (0, 0).into(),
            },
            camera: Camera::new((50, 46), (x / 2, y / 2)),
            network_client: NetworkClient::new(),
            layout,
            mode: UIMode::None
        }
    }

    pub fn sync(&mut self) {
        let query = <Read<component::Position>>::query().filter(tag::<component::Player>());
        for position in query.iter(self.network_client.world()) {
            self.camera.set_focus(*position);
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

    #[cfg(cargo_web)]
    pub fn handle_pointer_moved(&mut self, x: i32, y: i32) -> bool {
        self.render_context.mouse_position.x = x as i32;
        self.render_context.mouse_position.y = y as i32;
        false
    }

    #[cfg(not(cargo_web))]
    pub fn handle_pointer_moved(&mut self, x: i32, y: i32) -> bool {
        let scale = self.render_context.window.scale_factor();
        self.render_context.mouse_position.x = x as i32 * scale as i32;
        self.render_context.mouse_position.y = y as i32 * scale as i32;
        false
    }

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
            match self.mode {
                UIMode::None => {
                    match key {
                        Key::W => self.handle_move((0, -1)),
                        Key::A => self.handle_move((-1, 0)),
                        Key::S => self.handle_move((0, 1)),
                        Key::D => self.handle_move((1, 0)),
                        Key::Up => self.handle_focus((0, -1)),
                        Key::Left => self.handle_focus((-1, 0)),
                        Key::Down => self.handle_focus((0, 1)),
                        Key::Right => self.handle_focus((1, 0)),
                        Key::C => self.sync(),
                        Key::E => self.mode = UIMode::Interact,
                        Key::Space => self.handle_move((0, 0)),
                        Key::Escape => panic!("DIE DIE DIE"),
                        _ => {}
                    }
                },
                UIMode::Interact => {
                    match key {
                        Key::W => self.handle_interact((0, -1)),
                        Key::A => self.handle_interact((-1, 0)),
                        Key::S => self.handle_interact((0, 1)),
                        Key::D => self.handle_interact((1, 0)),
                        _ => {}
                    }
                }
            }
        }
    }



    pub fn handle_interact(&mut self, delta: impl Into<Vector>) {
        self.mode = UIMode::None;
        let delta = delta.into();
        let query = <Read<component::Position>>::query().filter(tag::<component::Player>());
        let player: Entity = query.iter_entities(&self.network_client.world()).take(1).next().expect("Couldn't find player").0;
        let position = *self.network_client.world().get_component::<component::Position>(player).expect("Player didn't have a position.");
        let map = self.network_client.resources().get::<Map>().unwrap();
        let position: Vector = (position.x, position.y).into();
        let index = map.point_to_index((position + delta.into()).to_tuple().into());
        let mut found_entity = None;
        if let Some(entity) = map.tile_content.get(index) {
            if let Some(entity) = entity {
                let name = self.network_client.world().get_component::<component::Name>(*entity).expect("This entity didn't have a name");
                self.log.push(
                    &format!("You interacted with {}", name.name),
                    Some(Color::GREEN),
                    None,
                );
                found_entity = Some(*entity);
            }
        }
        std::mem::drop(map);
        if let Some(entity) = found_entity {
            self.network_client.try_interact(entity);
        } else {
            self.log.push(
                &format!("You failed to interact with anything"),
                Some(Color::RED),
                None,
            );
        }

    }

    pub fn handle_focus(&mut self, delta: impl Into<Vector>) {
        self.camera.move_focus(delta);
    }

    pub fn handle_move(&mut self, delta: impl Into<Vector>) {
        let delta = delta.into();
        if self.network_client.try_move_player(delta) {
            self.camera.move_focus(delta);
        }
    }

    pub fn handle_click(&mut self, point: impl Into<Point>) {
        let point = self.camera.unproject(point.into());
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
        self.render_context.gfx.clear(Color::BLACK);
        self.camera.set_dimensions(self.layout.map.region.size.into());
        self.camera
            .render(&self.network_client, &mut self.layout.map);
        draw_ui(
            &mut self.layout,
            &self.network_client.world(),
            &mut self.render_context,
            &mut self.log,
            &self.mode
        );
        self.layout.render(&mut self.render_context);
        self.render_context.show();
    }

    pub fn focus(&self) -> Point {
        self.camera.focus()
    }

    pub fn mouse_position(&self) -> Vector {
        self.render_context.mouse_position
    }

    pub fn resources(&self) -> &Resources {
        self.network_client.resources()
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

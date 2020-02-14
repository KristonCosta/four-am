use specs::prelude::*;
use crate::{GameLog, Focus, component, TileContext, MouseState};
use crate::component::{Name, Position, TurnState, Killed};
use specs::{World, WorldExt, Builder};
use std::cmp::{min, max};
use crate::map::{Map, TilePos};
use quicksilver::graphics::Color;
use pathfinding::prelude::dijkstra_all;
use crate::glyph::Glyph;
use quicksilver::lifecycle::{ElementState, Key, Event, Window};
use crate::geom::{Rect, Point};
use crate::camera::get_screen_bounds;

pub struct GameState {
    pub(crate) ecs: World,
    pub(crate) runstate: RunState,
    pub(crate) tile_ctx: TileContext,
    pub(crate) map_region: Rect,
}


#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    Paused,
    Running,
}

pub fn handle_key(gs: &mut GameState, key: Key, state: ElementState) {
    if state == ElementState::Pressed {
        match key {
            Key::W => try_move_player(0, -1, &mut gs.ecs),
            Key::A => try_move_player(-1, 0, &mut gs.ecs),
            Key::S => try_move_player(0, 1, &mut gs.ecs),
            Key::D => try_move_player(1, 0, &mut gs.ecs),
            _ => {}
        }
    }
}

pub fn handle_click(gs: &mut GameState, point: impl Into<Point>) {
    let point = point.into();
    let GameState {
        ecs,
        runstate: _,
        tile_ctx,
        map_region,
    } = gs;
    let (min_x, _max_x, min_y, _max_y) = get_screen_bounds(&ecs, &tile_ctx);
    let point: Point = (
        point.x + min_x + map_region.origin.x,
        point.y + min_y + map_region.origin.y,
    )
        .into();
    let mut was_teleported = false;
    let mut desired_pos = None;
    {
        let positions = ecs.read_storage::<component::Position>();
        let tiles = ecs.read_storage::<component::PickableTile>();

        for (position, _tile) in (&positions, &tiles).join() {
            if position.x == point.x && position.y == point.y {
                desired_pos = Some(point);
                break;
            }
        }
    }
    if let Some(desired_pos) = desired_pos {
        move_player(ecs, desired_pos);
        was_teleported = true;
    }

    {
        clear_pickable(ecs);
    }
    if was_teleported {
        return;
    }

    let mut log = ecs.write_resource::<GameLog>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();
    for (name, position) in (&names, &positions).join() {
        if position.x == point.x && position.y == point.y {
            log.push(
                &format!("You clicked on {}", name.name),
                Some(Color::GREEN),
                None,
            );
        }
    }
}

pub fn clear_pickable(ecs: &mut World) {
    let mut rm_entities = vec![];
    {
        let tiles = ecs.read_storage::<component::PickableTile>();
        let entities = ecs.entities();
        for (entity, _tile) in (&entities, &tiles).join() {
            rm_entities.push(entity);
        }
    }
    for entity in rm_entities {
        ecs.delete_entity(entity);
    }
}

pub fn generate_pickable(ecs: &mut World, start: TilePos) {
    let result;
    {
        let map = ecs.fetch::<Map>();
        result = dijkstra_all(&start, |p| p.successors(&map, 5));
    }
    for (pos, _) in result.iter() {
        let glyph = Glyph {
            ch: '▒',
            foreground: Some(Color::from_rgba(233, 212, 96, 0.5)),
            background: None,
            render_order: 1,
        };
        ecs.create_entity()
            .with(component::Position { x: pos.0, y: pos.1 })
            .with(component::Renderable { glyph })
            .with(component::PickableTile)
            .build();
    }
}

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<component::Position>();
    let mut players = ecs.write_storage::<component::Player>();
    let mut active = ecs.write_storage::<component::ActiveTurn>();
    let mut killed = ecs.write_storage::<component::Killed>();
    let names = ecs.read_storage::<component::Name>();
    for (_player, pos, turn) in (&mut players, &mut positions, &mut active).join() {
        let map = ecs.fetch::<Map>();
        let desired_x = min(map.size.0, max(0, pos.x + delta_x));
        let desired_y = min(map.size.1, max(0, pos.y + delta_y));

        let coord = map.coord_to_index(desired_x, desired_y);
        if map.blocked[coord] {
            {
                let mut log = ecs.write_resource::<GameLog>();
                log.push(&format!("Ouch, you hit a wall!"), Some(Color::RED), None);
            }
        } else if let Some(entity) = map.tile_content[coord] {
            {
                let mut log = ecs.write_resource::<GameLog>();
                let name = names.get(entity);
                if let Some(name) = name {
                    log.push(&format!("Ouch, you killed {}", name.name), Some(Color::RED), None);
                }
                killed.insert(entity, Killed).expect("failed to insert killed");
            }
        } else {
            {
                let mut focus = ecs.write_resource::<Focus>();
                focus.x = desired_x;
                focus.y = desired_y;
            }
            pos.x = desired_x;
            pos.y = desired_y;
            turn.state = TurnState::DONE;
        }
    }
}

pub fn move_player(ecs: &mut World, desired_pos: impl Into<Point>) {
    let desired_pos = desired_pos.into();
    let mut positions = ecs.write_storage::<component::Position>();
    let mut players = ecs.write_storage::<component::Player>();
    let mut active = ecs.write_storage::<component::ActiveTurn>();
    for (_player, pos, active) in (&mut players, &mut positions, &mut active).join() {
        {
            let mut focus = ecs.write_resource::<Focus>();
            focus.x = desired_pos.x;
            focus.y = desired_pos.y;
        }
        pos.x = desired_pos.x;
        pos.y = desired_pos.y;
        active.state = TurnState::DONE;
        break;
    }
}

pub fn handle_event(window: &Window, gs: &mut GameState, event: Event) -> bool {
    match event {
        Event::KeyboardInput { key, state } => {
            handle_key(gs, key, state);
            true
        }
        Event::MouseMoved { pointer: _, position } => {
            let scale = window.scale_factor();

            let mut mouse = gs.ecs.write_resource::<MouseState>();
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
                    let mouse = gs.ecs.fetch::<MouseState>();
                    pos = gs.tile_ctx.grid.point_to_grid((mouse.x, mouse.y));
                }

                handle_click(gs,pos);
            }
            state == ElementState::Pressed
        }
        _ => false,
    }
}

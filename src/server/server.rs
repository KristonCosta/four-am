use crate::component::{Killed, Name, Position, TurnState};
use crate::frontend::glyph::Glyph;
use crate::geom::{Point, Vector};
use crate::message::Message;
use crate::resources::log::GameLog;
use crate::{color, component};

use legion::prelude::*;
use quicksilver::graphics::Color;
use rand::Rng;
use std::cmp::{max, min};
use crate::server::systems::indexer::map_indexer;
use crate::server::systems::turn_system::{turn_system, PendingMoves};
use crate::server::systems::ai::monster_ai;
use crate::map::Map;
use crate::server::map_builders::factories::{random_builder, drunk_builder};
use crate::server::map_builders::BuiltMap;
use crate::server::gamestate::RunState;
use instant::Instant;

pub struct Server {
    pub(crate) world: World,
    pub(crate) resources: Resources,
    pub(crate) universe: Universe,
    schedule: Schedule,
    run_state: RunState,
    map_state: MapState,
}

pub struct MapState {
    mapgen_index: usize,
    mapgen_built_map: BuiltMap,
    mapgen_timer: Instant,
}


pub struct MessageQueue {
    messages: Vec<Message>,
}

impl MessageQueue {
    pub fn push(&mut self, message: Message) {
        self.messages.push(message);
    }
}

impl Server {
    fn setup_ecs() -> (Universe, World, Resources) {
        let universe = Universe::new();
        let mut world = universe.create_world();

        let mut resources = Resources::default();
        let turn = PendingMoves::new();

        let mut message_queue = MessageQueue { messages: vec![] };
        resources.insert(turn);

        resources.insert(message_queue);

        (universe, world, resources)
    }

    pub fn new() -> Self {
        let (mut universe, mut world, mut resources) = Self::setup_ecs();
        let mut rng = rand::thread_rng();
        let built_map = drunk_builder((80, 43).into(), 10, &mut rng);
        let BuiltMap {
            spawn_list,
            map,
            starting_position,
            rooms,
            history,
            with_history
        } = &built_map;

        let position = match starting_position {
            Some(pos) => pos,
            None => panic!("No starting position in map!"),
        };

        if *with_history {
            resources.insert(history[0].clone())
        } else {
            resources.insert(map.clone());
        }

        let schedule = Schedule::builder()
            .add_system(map_indexer())
            .add_system(turn_system())
            .flush()
            .add_system(monster_ai())
            .flush()
            .add_system(sweep())
            .build();

        Server {
            world,
            resources,
            schedule,
            universe,
            run_state: RunState::MapGeneration,
            map_state: MapState {
                mapgen_index: 0,
                mapgen_built_map: built_map,
                mapgen_timer: Instant::now(),
            }
        }
    }

    fn reload_world(&mut self, built_map: BuiltMap) {
        let mut world = self.universe.create_world();
        std::mem::swap(&mut self.world, &mut world);
        self.run_state = RunState::MapGeneration;
        let BuiltMap {
            spawn_list,
            map,
            starting_position,
            rooms,
            history,
            with_history
        } = &built_map;

        let position = match starting_position {
            Some(pos) => pos,
            None => panic!("No starting position in map!"),
        };

        if *with_history {
            self.resources.insert(history[0].clone())
        } else {
            self.resources.insert(map.clone());
        }
        std::mem::swap(&mut self.map_state, &mut MapState {
            mapgen_index: 0,
            mapgen_built_map: built_map,
            mapgen_timer: Instant::now(),
        });
    }

    pub fn reload_drunken_world(&mut self) {
        let mut rng = rand::thread_rng();
        let built_map = drunk_builder((80, 43).into(), 10, &mut rng);
        self.reload_world(built_map);
    }

    pub fn reload_room_world(&mut self) {
        let mut rng = rand::thread_rng();
        let built_map = random_builder((80, 43).into(), 10, &mut rng);
        self.reload_world(built_map);
    }

    fn insert_entities(&mut self) {
        let mut command_buffer = CommandBuffer::new(&self.world);
        let map = self.resources.get::<Map>().unwrap();
        for i in 1..100 {
            generate_centipede(&map, &mut command_buffer, i);
        }
        let position = self.map_state.mapgen_built_map.starting_position.unwrap().clone();
        command_buffer
            .start_entity()
            .with_component(component::Position {
                x: position.x,
                y: position.y,
            })
            .with_component(component::Renderable {
                glyph: Glyph {
                    ch: '@',
                    foreground: Some(Color::YELLOW),
                    background: None,
                    render_order: 3,
                },
            })
            .with_component(component::Player)
            .with_component(component::Name {
                name: "Player".to_string(),
            })
            .with_component(component::Priority { value: 100 })
            .with_component(component::TileBlocker)
            .build();
        command_buffer.write(&mut self.world);
    }

    pub fn messages(&mut self) -> Vec<Message> {
        let mut queue = self.resources.get_mut::<MessageQueue>();
        queue
            .expect("failed to get resource")
            .messages
            .drain(..)
            .collect()
    }

    pub fn tick(&mut self) {
        match self.run_state {
            RunState::Running => {
                let world = &mut self.world;
                let resources = &mut self.resources;
                let schedule = &mut self.schedule;
                schedule.execute(world, resources);
            },
            RunState::MapGeneration => {
                if self.map_state.mapgen_timer.elapsed().as_millis() > 200 {
                    let resources = &mut self.resources;
                    self.map_state.mapgen_index += 1;
                    if self.map_state.mapgen_index >= self.map_state.mapgen_built_map.history.len() {
                        resources.insert(self.map_state.mapgen_built_map.map.clone());
                        self.run_state = RunState::Initializing;

                    } else {
                        let index = self.map_state.mapgen_index;
                        resources.insert(self.map_state.mapgen_built_map.history[index].clone());
                        self.map_state.mapgen_timer = Instant::now();
                    }
                }
            },
            RunState::Initializing => {
                let resources = &mut self.resources;
                let mut map = resources.get_mut::<Map>().unwrap();
                map.refresh_blocked();
                std::mem::drop(map);
                self.insert_entities();
                self.run_state = RunState::Running;
            },
            _ => panic!("Unhandled runstate!")
        }
    }

    pub fn try_move_player(&mut self, delta_x: i32, delta_y: i32) -> bool {
        if self.run_state != RunState::Running {
            return false
        }
        let world = &mut self.world;
        let resources = &mut self.resources;
        let mut message_queue = resources.get_mut::<MessageQueue>().unwrap();
        let mut map = resources.get_mut::<Map>().unwrap();
        let mut query = <(
            Write<component::Position>,
            Write<component::Player>,
            Write<component::ActiveTurn>,
        )>::query();

        let mut command_buffer = CommandBuffer::new(&world);
        let mut killed = vec![];
        let mut moved = false;
        for (mut pos, _, mut turn) in query.iter_mut(world) {
            let desired_x = min(map.size.x, max(0, pos.x + delta_x));
            let desired_y = min(map.size.y, max(0, pos.y + delta_y));

            let coord = map.coord_to_index(desired_x, desired_y);
            if map.blocked[coord] {
                message_queue.push(Message::GameEvent(
                    format!("Ouch, you hit a wall!"),
                    Some(Color::RED),
                    None,
                ));
            } else if let Some(entity) = map.tile_content[coord] {
                killed.push(entity);
                command_buffer.add_component(entity, Killed);
            } else {
                pos.x = desired_x;
                pos.y = desired_y;
                turn.state = TurnState::DONE;
                moved = true;
            }
        }
        for entity in killed {
            let name = world.get_component::<Name>(entity).unwrap();
            message_queue.push(Message::GameEvent(
                format!("Ouch, you killed {}", name.name),
                Some(Color::RED),
                None,
            ));
        }

        command_buffer.write(world);
        moved
    }

    pub fn move_player(&mut self, desired_pos: impl Into<Point>) {
        let desired_pos = desired_pos.into();
        let world = &mut self.world;
        let resources = &mut self.resources;
        let mut query = <(
            Write<component::Position>,
            Write<component::Player>,
            Write<component::ActiveTurn>,
        )>::query();
        for (mut pos, _, mut turn) in query.iter_mut(world) {
            pos.x = desired_pos.x;
            pos.y = desired_pos.y;
            turn.state = TurnState::DONE;
            break;
        }
    }
}

fn generate_centipede(map: &Map, buffer: &mut CommandBuffer, i: u32) {
    let mut rng = rand::thread_rng();
    let mut position_x;
    let mut position_y;
    let size = map.size.clone();
    loop {
        position_x = rng.gen_range(0, size.x - 1);
        position_y = rng.gen_range(0, size.y - 1);
        if !map.is_blocked((position_x, position_y).into()) {
            break
        }
    };

    buffer
        .start_entity()
        .with_component(component::Position {
            x: position_x,
            y: position_y,
        })
        .with_component(component::Renderable {
            glyph: Glyph {
                ch: 'C',
                foreground: Some(color::TAN),
                background: None,
                render_order: 3,
            },
        })
        .with_component(component::Name {
            name: format!("Giant Centipede {}", i),
        })
        .with_component(component::Priority { value: 1 })
        .with_component(component::TileBlocker)
        .with_component(component::Monster)
        .build();
}

pub fn generate_blood(buffer: &mut CommandBuffer, pos: Vector) {
    buffer
        .start_entity()
        .with_component(component::Position { x: pos.x, y: pos.y })
        .with_component(component::Renderable {
            glyph: Glyph {
                ch: '%',
                foreground: Some(color::RED),
                background: None,
                render_order: 100,
            },
        })
        .build();
}

pub fn sweep() -> Box<dyn Schedulable> {
    SystemBuilder::new("sweep_system")
        .with_query(<(Read<Killed>, Read<Position>)>::query())
        .build(move |command_buffer, mut world, _, query| {
            let mut killed = vec![];
            for (entity, (_, position)) in query.iter_entities(&mut world) {
                killed.push((entity, (position.x, position.y)));
            }
            for (entity, position) in killed {
                command_buffer.delete(entity);
                generate_blood(command_buffer, (position.0, position.1).into());
            }
        })
}

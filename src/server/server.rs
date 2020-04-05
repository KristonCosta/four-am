use crate::component::{Hurt, Killed, Name, Position, TurnState};
use crate::frontend::glyph::Glyph;
use crate::geom::{Point, Vector};
use crate::message::Message;
use crate::{color, component};

use crate::map::Map;
use crate::server::gamestate::RunState;
use crate::server::map_builders::factories::{drunk_builder, random_builder};
use crate::server::map_builders::BuiltMap;
use crate::server::systems::ai_system::ai_system;
use crate::server::systems::index_system::index_system;
use crate::server::systems::turn_system::{turn_system, PendingMoves};
use crate::server::systems::vision_system::vision_system;
use instant::Instant;
use legion::prelude::*;
use quicksilver::graphics::Color;
use quicksilver::load_file;
use rand::Rng;
use serde::Deserialize;
use std::cmp::{max, min};

pub struct Server {
    pub(crate) world: World,
    pub(crate) resources: Resources,
    pub(crate) universe: Universe,
    pub(crate) data: Data,
    schedule: Schedule,
    run_state: RunState,
    map_state: MapState,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Data {
    pub mobs: Vec<Monster>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Monster {
    pub name: String,
    pub renderable: Renderable,
    pub health: Health,
    pub priority: Priority,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Renderable {
    pub glyph: GlyphData,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GlyphData {
    pub ch: char,
    pub foreground: Option<String>,
    pub background: Option<String>,
    pub render_order: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Health {
    pub current: u32,
    pub max: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Priority {
    pub value: u32,
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
        let world = universe.create_world();

        let mut resources = Resources::default();
        let turn = PendingMoves::new();

        let message_queue = MessageQueue { messages: vec![] };
        resources.insert(turn);

        resources.insert(message_queue);

        (universe, world, resources)
    }

    pub async fn new() -> Self {
        let (universe, world, mut resources) = Self::setup_ecs();
        let mut rng = rand::thread_rng();
        let built_map = drunk_builder((80, 43).into(), 10, &mut rng);
        let BuiltMap {
            spawn_list: _,
            map,
            starting_position: _,
            rooms: _,
            history,
            with_history,
        } = &built_map;

        if *with_history {
            resources.insert(history[0].clone())
        } else {
            resources.insert(map.clone());
        }

        let schedule = Schedule::builder()
            .add_system(index_system())
            .add_system(turn_system())
            .add_system(vision_system())
            .flush()
            .add_system(ai_system())
            .flush()
            .add_system(damage_resolution())
            .flush()
            .add_system(sweep())
            .build();

        Server {
            world,
            resources,
            schedule,
            universe,
            data: Server::load_resources().await,
            run_state: RunState::MapGeneration,
            map_state: MapState {
                mapgen_index: 0,
                mapgen_built_map: built_map,
                mapgen_timer: Instant::now(),
            },
        }
    }

    async fn load_resources() -> Data {
        let file_contents = load_file("data/monsters.json").await.expect("ahh");
        let raw_string = std::str::from_utf8(&file_contents).expect("Ono");
        serde_json::from_str(raw_string).expect("oooof")
    }

    fn reload_world(&mut self, built_map: BuiltMap) {
        let mut world = self.universe.create_world();
        std::mem::swap(&mut self.world, &mut world);
        self.run_state = RunState::MapGeneration;
        let BuiltMap {
            spawn_list: _,
            map,
            starting_position: _,
            rooms: _,
            history,
            with_history,
        } = &built_map;

        if *with_history {
            self.resources.insert(history[0].clone())
        } else {
            self.resources.insert(map.clone());
        }
        std::mem::swap(
            &mut self.map_state,
            &mut MapState {
                mapgen_index: 0,
                mapgen_built_map: built_map,
                mapgen_timer: Instant::now(),
            },
        );
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
            generate_centipede(&map, &mut command_buffer, &self.data, i);
        }
        let position = self
            .map_state
            .mapgen_built_map
            .starting_position
            .unwrap()
            .clone();
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
            .with_component(component::FieldOfView {
                visible_tiles: Vec::new(),
                range: 8,
                previous_position: (-1, -1).into(),
            })
            .with_component(component::Name {
                name: "Player".to_string(),
            })
            .with_component(component::Health {
                current: 10,
                max: 10,
            })
            .with_component(component::Priority { value: 100 })
            .with_component(component::TileBlocker)
            .with_tag(())
            .with_tag(component::Player)
            .build();
        command_buffer.write(&mut self.world);
    }

    pub fn messages(&mut self) -> Vec<Message> {
        let queue = self.resources.get_mut::<MessageQueue>();
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
            }
            RunState::MapGeneration => {
                if !self.map_state.mapgen_built_map.with_history {
                    self.resources
                        .insert(self.map_state.mapgen_built_map.map.clone());
                    self.run_state = RunState::Initializing;
                } else if self.map_state.mapgen_timer.elapsed().as_millis() > 200 {
                    let resources = &mut self.resources;
                    self.map_state.mapgen_index += 1;
                    if self.map_state.mapgen_index >= self.map_state.mapgen_built_map.history.len()
                    {
                        resources.insert(self.map_state.mapgen_built_map.map.clone());
                        self.run_state = RunState::Initializing;
                    } else {
                        let index = self.map_state.mapgen_index;
                        resources.insert(self.map_state.mapgen_built_map.history[index].clone());
                        self.map_state.mapgen_timer = Instant::now();
                    }
                }
            }
            RunState::Initializing => {
                let resources = &mut self.resources;
                let mut map = resources.get_mut::<Map>().unwrap();
                map.refresh_blocked();
                std::mem::drop(map);
                self.insert_entities();
                self.run_state = RunState::Running;
            }
            _ => panic!("Unhandled runstate!"),
        }
    }

    pub fn try_move_player(&mut self, delta_x: i32, delta_y: i32) -> bool {
        if self.run_state != RunState::Running {
            return false;
        }
        let world = &mut self.world;
        let resources = &mut self.resources;
        let mut message_queue = resources.get_mut::<MessageQueue>().unwrap();
        let map = resources.get_mut::<Map>().unwrap();
        let query = <(Write<component::Position>, Write<component::ActiveTurn>)>::query()
            .filter(tag::<component::Player>());

        let mut command_buffer = CommandBuffer::new(&world);
        let mut killed = vec![];
        let mut moved = false;
        for (entity, (mut pos, mut turn)) in query.iter_entities_mut(world) {
            let desired_x = min(map.size.x, max(0, pos.x + delta_x));
            let desired_y = min(map.size.y, max(0, pos.y + delta_y));

            let coord = map.coord_to_index(desired_x, desired_y);
            println!("{:?}{:?}", desired_x, desired_y);
            if map.blocked[coord] {
                message_queue.push(Message::GameEvent(
                    format!("Ouch, you hit a wall!"),
                    Some(Color::RED),
                    None,
                ));
            } else if let Some(other) = map.tile_content[coord] {
                if entity != other {
                    killed.push(other);
                    command_buffer.add_component(other, Hurt);
                }
            } else {
                pos.x = desired_x;
                pos.y = desired_y;
                moved = true;
            }
            turn.state = TurnState::DONE;
        }
        for entity in killed {
            let name = world.get_component::<Name>(entity).unwrap();
            message_queue.push(Message::GameEvent(
                format!("Ouch, you hurt {}", name.name),
                Some(Color::RED),
                None,
            ));
        }

        command_buffer.write(world);
        moved
    }
}

fn generate_monster(buffer: &mut CommandBuffer, monster: &Monster, position: Point) {
    buffer
        .start_entity()
        .with_component(component::Position {
            x: position.x,
            y: position.y,
        })
        .with_component(component::Renderable {
            glyph: Glyph {
                ch: monster.renderable.glyph.ch,
                foreground: monster
                    .renderable
                    .glyph
                    .foreground
                    .as_ref()
                    .map(|color| Color::from_hex(&color)),
                background: None,
                render_order: monster.renderable.glyph.render_order as i32,
            },
        })
        .with_component(component::Health {
            current: monster.health.current,
            max: monster.health.max,
        })
        .with_component(component::Name {
            name: monster.name.clone(),
        })
        .with_component(component::Priority {
            value: monster.priority.value as u8,
        })
        .with_component(component::TileBlocker)
        .with_component(component::Monster)
        .build();
}

fn generate_centipede(map: &Map, buffer: &mut CommandBuffer, data: &Data, _: u32) {
    let mut rng = rand::thread_rng();
    let mut position_x;
    let mut position_y;
    let size = map.size.clone();
    loop {
        position_x = rng.gen_range(0, size.x - 1);
        position_y = rng.gen_range(0, size.y - 1);
        if !map.is_blocked((position_x, position_y).into()) {
            break;
        }
    }
    generate_monster(buffer, &data.mobs[0], (position_x, position_y).into())
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

pub fn damage_resolution() -> Box<dyn Schedulable> {
    SystemBuilder::new("damage_system")
        .with_query(<(Read<Hurt>, Write<component::Health>)>::query())
        .build(move |command_buffer, mut world, _, query| {
            for (entity, (_, mut health)) in query.iter_entities_mut(&mut world) {
                println!("Running damage resolution");
                health.current -= 1;
                if health.current <= 0 {
                    health.current = 0;
                    command_buffer.add_component(entity, Killed);
                }
                command_buffer.remove_component::<Hurt>(entity);
            }
        })
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

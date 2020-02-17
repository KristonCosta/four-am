use crate::component::{Killed, Name, Position, TurnState};
use crate::frontend::glyph::Glyph;
use crate::geom::{Point, Vector};
use crate::message::Message;
use crate::resources::log::GameLog;
use crate::server::ai::monster_ai;
use crate::server::map::{map_indexer, Map, MapBuilder, RoomMapBuilder};
use crate::server::turn_system::{turn_system, PendingMoves};
use crate::server::{ai, map};
use crate::{color, component};

use legion::prelude::*;
use quicksilver::graphics::Color;
use rand::Rng;
use std::cmp::{max, min};

pub struct Server {
    pub(crate) world: World,
    pub(crate) resources: Resources,
    schedule: Schedule,
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
        let (map, position) = RoomMapBuilder::build((60, 30), 10);
        resources.insert(map);
        let mut command_buffer = CommandBuffer::new(&world);
        for i in 1..100 {
            generate_centipede(&mut command_buffer, i);
        }
        command_buffer
            .start_entity()
            .with_component(component::Position {
                x: position.0,
                y: position.1,
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
        command_buffer.write(&mut world);

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
        }
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
        let world = &mut self.world;
        let resources = &mut self.resources;
        let schedule = &mut self.schedule;
        let (sender, receiver) = crossbeam_channel::unbounded();
        world.subscribe(sender, any());

        schedule.execute(world, resources);
        for event in receiver.try_iter() {
            println!("{:?}", event);
        }
    }

    pub fn try_move_player(&mut self, delta_x: i32, delta_y: i32) -> bool {
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
            let desired_x = min(map.size.0, max(0, pos.x + delta_x));
            let desired_y = min(map.size.1, max(0, pos.y + delta_y));

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

fn generate_centipede(buffer: &mut CommandBuffer, i: u32) {
    let mut rng = rand::thread_rng();
    let position_x = rng.gen_range(10, 50);
    let position_y = rng.gen_range(10, 20);
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

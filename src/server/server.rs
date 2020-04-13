use crate::component::TurnState;
use crate::message::Message;
use crate::component;

use crate::map::Map;
use crate::server::gamestate::RunState;
use crate::server::map_builders::factories::drunk_builder;
use crate::server::map_builders::BuiltMap;
use crate::server::systems::index_system::index_system;
use crate::server::systems::turn_system::{turn_system, PendingMoves};

use instant::Instant;
use legion::prelude::*;
use quicksilver::graphics::Color;
use std::cmp::{max, min};
use super::{map_builders::factories::shop_builder, serializers::{entity_factory}};

pub struct Server {
    pub(crate) world: World,
    pub(crate) resources: Resources,
    pub(crate) universe: Universe,
    schedule: Schedule,
    run_state: RunState,
    map_state: MapState,
    factory: entity_factory::EntityFactory
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
        let built_map = shop_builder((20, 20).into(), &mut rng);
        let BuiltMap {
            spawn_list: _,
            map,
            starting_position: _,
            rooms: _,
            history,
            with_history,
        } = &built_map;
        let factory = entity_factory::EntityFactory::load().await;
        if *with_history {
            resources.insert(history[0].clone())
        } else {
            resources.insert(map.clone());
        }

        let schedule = Schedule::builder()
            .add_system(index_system())
            .add_system(turn_system())
            .build();

        Server {
            world,
            resources,
            schedule,
            universe,
            run_state: RunState::Initializing,
            map_state: MapState {
                mapgen_index: 0,
                mapgen_built_map: built_map,
                mapgen_timer: Instant::now(),
            },
            factory
        }
    }

    fn insert_entities(&mut self) {
        let mut command_buffer = CommandBuffer::new(&self.world);
        let position = self
            .map_state
            .mapgen_built_map
            .starting_position
            .unwrap()
            .clone();
        let player = self.factory.build("player", Some(position), &mut command_buffer);
        command_buffer.add_tag(player, component::Player);
        let entity = self.factory.build("display", Some((position.x + 1, position.y).into()), &mut command_buffer);
        self.factory.build("display", Some((position.x + 1, position.y + 1).into()), &mut command_buffer);
        self.factory.build("display", Some((position.x + 1, position.y + 2).into()), &mut command_buffer);
        let love = self.factory.build("love", None, &mut command_buffer);
        command_buffer.write(&mut self.world);
        self.world.get_component_mut::<component::Inventory>(entity).unwrap().contents.replace(love);
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

    pub fn try_interact(&mut self, entity: Entity) -> bool {
        let mut renderable = self.world.get_component_mut::<component::Renderable>(entity).unwrap();
        if renderable.glyph.foreground != Some(Color::RED) {
            renderable.glyph.foreground = Some(Color::RED);
        } else {
            renderable.glyph.foreground = Some(Color::GREEN);
        }
        true
    }

    pub fn try_put(&mut self, entity: Entity, id: &str) -> bool {
        let mut command_buffer = CommandBuffer::new(&self.world);
        let obj = self.factory.build(id, None, &mut command_buffer);
        command_buffer.write(&mut self.world);
        self.world.get_component_mut::<component::Inventory>(entity).unwrap().contents.replace(obj);
        true
    }

    pub fn try_take(&mut self, entity: Entity) -> bool {
        let contents = {
            let mut inv = self.world.get_component_mut::<component::Inventory>(entity).unwrap();
            inv.as_mut().contents.take()
        };
        if let Some(contents) = contents {
            let world = &mut self.world;
            let resources = &mut self.resources;
            let name = world.get_component_mut::<component::Name>(contents).unwrap();
            let mut message_queue = resources.get_mut::<MessageQueue>().unwrap();
            message_queue.push(Message::GameEvent(
                format!("You took {:?}", name.name),
                Some(Color::GREEN),
                None,
            ));
            true
        } else {
            false
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

        let mut moved = false;
        for (entity, (mut pos, mut turn)) in query.iter_entities_mut(world) {
            let desired_x = min(map.size.x, max(0, pos.x + delta_x));
            let desired_y = min(map.size.y, max(0, pos.y + delta_y));

            let coord = map.coord_to_index(desired_x, desired_y);
            if map.blocked[coord] {
                message_queue.push(Message::GameEvent(
                    format!("Ouch, you hit a wall!"),
                    Some(Color::RED),
                    None,
                ));
            } else if let Some(other) = map.tile_content[coord] {
                if entity != other {

                }
            } else {
                pos.x = desired_x;
                pos.y = desired_y;
                moved = true;
            }
            turn.state = TurnState::DONE;
        }

        command_buffer.write(world);
        moved
    }
}


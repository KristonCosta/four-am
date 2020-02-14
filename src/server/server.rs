use specs::{World, WorldExt, Builder, RunNow};
use rand::Rng;
use crate::{component, color};
use crate::component::{Killed, Position, register_components};
use crate::client::glyph::Glyph;

use crate::geom::Vector;
use crate::server::{ai, turn_system, map};
use quicksilver::graphics::Color;
use crate::server::map::{RoomMapBuilder, MapBuilder};
use crate::client::client::{Focus, MouseState};
use crate::resources::log::GameLog;
use specs::join::Join;

pub struct Server {
    logs: Vec<String>,
    pub(crate) world: World,
}

impl Server {
    pub fn new() -> Self {
        println!("Starting.");

        let mut ecs = World::new();
        register_components(&mut ecs);
        register_resources(&mut ecs);
        let (map, position) = RoomMapBuilder::build((60, 30), 10);
        ecs.insert(map);
        let focus = Focus {
            x: position.0,
            y: position.1,
        };
        ecs.insert(focus);

        for i in 1..100 {
            generate_centipede(&mut ecs, i);
        }

        ecs.create_entity()
            .with(component::Position {
                x: position.0,
                y: position.1,
            })
            .with(component::Renderable {
                glyph: Glyph {
                    ch: '@',
                    foreground: Some(Color::YELLOW),
                    background: None,
                    render_order: 3,
                },
            })
            .with(component::Player {})
            .with(component::Name {
                name: "Player".to_string(),
            })
            .with(component::Priority{
                value: 100
            })
            .with(component::TileBlocker)
            .build();
        Server {
            logs: vec![],
            world: ecs
        }
    }

    pub fn tick(&mut self) {
        let mut ms = ai::MonsterAi;
        let mut ts = turn_system::TurnSystem;
        let mut indexer = map::MapIndexer;
        let mut ecs = &mut self.world;
        indexer.run_now(ecs);
        ms.run_now(ecs);
        ts.run_now(ecs);
        sweep(ecs);
        ecs.maintain();
    }
}

pub fn register_resources(ecs: &mut World) {
    let mut log = GameLog::with_length(5);
    log.push("Hello, world!", Some(Color::GREEN), None);

    let mouse = MouseState { x: 0, y: 0 };

    let turn = turn_system::PendingMoves::new();
    ecs.insert(turn);
    ecs.insert(log);
    ecs.insert(mouse);
}

fn generate_centipede(ecs: &mut World, i :u32) {
    let mut rng = rand::thread_rng();
    let position_x = rng.gen_range(10, 50);
    let position_y = rng.gen_range(10, 20);
    ecs.create_entity()
        .with(component::Position {
            x: position_x,
            y: position_y,
        })
        .with(component::Renderable {
            glyph: Glyph {
                ch: 'C',
                foreground: Some(color::TAN),
                background: None,
                render_order: 3,
            },
        })
        .with(component::Name {
            name: format!("Giant Centipede {}", i),
        })
        .with(component::Priority {
            value: 1,
        })
        .with(component::TileBlocker)
        .with(component::Monster)
        .build();
}

pub fn generate_blood(ecs: &mut World, pos: Vector) {
    ecs.create_entity()
        .with(component::Position {
            x: pos.x,
            y: pos.y,
        })
        .with(component::Renderable {
            glyph: Glyph {
                ch: '%',
                foreground: Some(color::RED),
                background: None,
                render_order: 100
            }
        })
        .build();
}

pub fn sweep(ecs: &mut World) {
    let mut killed = vec![];
    {
        let combat_stats = ecs.read_storage::<Killed>();
        let positions = ecs.read_storage::<Position>();
        let entities = ecs.entities();
        for (entity, _stats, position) in (&entities, &combat_stats, &positions).join() {
            killed.push((entity, (position.x, position.y)));
        }
    }

    for (entity, position) in killed {
        ecs.delete_entity(entity).expect("Failed to delete entity");
        generate_blood(ecs, (position.0, position.1).into());
    }
}

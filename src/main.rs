#[macro_use]
extern crate specs_derive;

use quicksilver::Result;
use quicksilver::lifecycle::{Settings, run, Window, EventStream};
use instant::Instant;
use quicksilver::graphics::Graphics;
use crate::server::server::Server;
use crate::client::client::Client;

pub mod server;
pub mod resources;
pub mod client;
pub mod common;
pub mod component;
pub mod geom;
pub mod color;
pub mod error;

fn main() {
    run(
        Settings {
            size: quicksilver::geom::Vector::new(800.0, 600.0).into(),
            title: "Whoa",
            vsync: true,
            ..Settings::default()
        },
        app,
    );
}

type FP = f32;
const MS_PER_UPDATE: FP = 1.0;

#[derive(Debug)]
pub struct TimeStep {
    last_time:   Instant,
    delta_time:  FP,
    frame_count: u32,
    frame_time:  FP,
}

impl TimeStep {
    // https://gitlab.com/flukejones/diir-doom/blob/master/game/src/main.rs
    // Grabbed this from here
    pub fn new() -> TimeStep {
        TimeStep {
            last_time:   Instant::now(),
            delta_time:  0.0,
            frame_count: 0,
            frame_time:  0.0,
        }
    }

    pub fn delta(&mut self) -> FP {
        let current_time = Instant::now();
        let delta = current_time.duration_since(self.last_time).as_micros()
            as FP
            * 0.001;
        self.last_time = current_time;
        self.delta_time = delta;
        delta
    }

    // provides the framerate in FPS
    pub fn frame_rate(&mut self) -> Option<u32> {
        self.frame_count += 1;
        self.frame_time += self.delta_time;
        let tmp;
        // per second
        if self.frame_time >= 1000.0 {
            tmp = self.frame_count;
            self.frame_count = 0;
            self.frame_time = 0.0;
            return Some(tmp);
        }
        None
    }
}


async fn app(window: Window, mut gfx: Graphics, mut events: EventStream) -> Result<()> {
    let mut timestep = TimeStep::new();
    let mut lag: f32 = 0.0;
    let mut turns = 0;
    let mut server = Server::new();
    let mut client = Client::new(window, gfx, events).await;
    loop {

        client.tick(&mut server.world).await;
        lag += timestep.delta();
        while lag >= MS_PER_UPDATE {
            turns += 1;
            server.tick();
            lag -= MS_PER_UPDATE;

        }
        if let Some(fps) = timestep.frame_rate() {
            println!("FPS {}", fps);
            println!("TPS {}", turns);
            turns = 0;
        }
        client.render(&mut server.world);
    }
}

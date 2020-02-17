use crate::geom::Vector;
use crate::server::map::Map;
use crate::server::server::Server;
use legion::prelude::*;
use std::ops::Deref;

pub struct NetworkClient {
    server: Option<Server>,
}

impl NetworkClient {
    pub fn new() -> Self {
        NetworkClient { server: None }
    }

    pub fn unbind(&mut self) -> Server {
        self.server.take().expect("Tried to unbind unbound server")
    }

    pub fn bind(&mut self, server: Server) {
        self.server = Some(server);
    }

    pub fn world(&self) -> &World {
        &self.server.as_ref().unwrap().world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.server.as_mut().unwrap().world
    }

    pub fn resources(&self) -> &Resources {
        &self.server.as_ref().unwrap().resources
    }

    pub fn try_move_player(&mut self, delta: impl Into<Vector>) -> bool {
        let delta = delta.into();
        self.server
            .as_mut()
            .unwrap()
            .try_move_player(delta.x, delta.y)
    }
}

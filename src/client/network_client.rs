use crate::geom::Vector;
use crate::server::server::Server;
use legion::prelude::*;

pub struct NetworkClient {
    server: Option<Server>,
}

pub enum WorldType {
    Drunken,
    Room,
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

    pub fn resources(&self) -> &Resources {
        &self.server.as_ref().unwrap().resources
    }

    pub fn get_player_inventory(&self) -> Vec<Entity> {
        self.server.as_ref().unwrap().get_player_inventory()
    }

    pub fn try_move_player(&mut self, delta: impl Into<Vector>) -> bool {
        let delta = delta.into();
        self.server
            .as_mut()
            .unwrap()
            .try_move_player(delta.x, delta.y)
    }

    pub fn try_interact(&mut self, entity: Entity) -> bool {
        self.server.as_mut().unwrap().try_interact(entity)
    }

    pub fn try_player_take(&mut self, entity: Entity) -> bool {
        self.server.as_mut().unwrap().try_player_take(entity)
    }

    pub fn try_player_put(&mut self, entity: Entity, player_inv: Entity) -> bool {
        self.server.as_mut().unwrap().try_player_put(entity, player_inv)
    }
}

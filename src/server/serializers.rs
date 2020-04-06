
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Data {
    builder: Vec<EntityBuilder>
}

#[derive(Deserialize, Debug, Clone)]
pub struct EntityBuilder {
    id: String,
    renderable: Renderable,
    name: String,
    priority: Option<Priority>,
    display_cabinet: Option<bool>,
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
    pub render_order: i32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Priority {
    pub value: u8,
}


pub mod entity_factory {
    use std::collections::HashMap;
    use super::{Data, EntityBuilder};
    use quicksilver::{graphics::Color, load_file};
    use serde_json::from_str;
    use crate::{component, geom::Point, frontend::glyph::Glyph};
    use legion::prelude::{Entity, CommandBuffer};

    pub struct EntityFactory {
        registry: HashMap<String, EntityBuilder>
    }

    impl EntityFactory {

        pub async fn load() -> Self {
            let file_contents = load_file("data/entities.json").await.expect("Couldn't find entity factory file");
            let raw_string = std::str::from_utf8(&file_contents).expect("Couldn't get raw string from factory file");
            let mut data: Data = from_str(raw_string).expect("Invalid entity factory file");
            let mut registry = HashMap::new();
            for factory in data.builder.drain(..) {
                registry.insert(factory.id.clone(), factory);
            }
            Self {
                registry
            }
        }

        fn deserialize_color(color: Option<String>) -> Option<Color> {
            match color {
                None => None, 
                Some(color) => Some(Color::from_hex(&color))
            }
        }

        pub fn build(&self, id: &str, position: impl Into<Point>, buffer: &mut CommandBuffer) -> Entity {
            let position = position.into();
            let options = self.registry.get(id).expect(&format!("Could not find {:?}", id));
            let builder = buffer.start_entity();
            let builder = builder
                .with_component(component::Position {
                    x: position.x,
                    y: position.y,
                })
                .with_component(component::Renderable {
                    glyph: Glyph {
                        ch: options.renderable.glyph.ch,
                        foreground: Self::deserialize_color(options.renderable.glyph.foreground.clone()),
                        background: Self::deserialize_color(options.renderable.glyph.background.clone()),
                        render_order: options.renderable.glyph.render_order,
                    },
                })
                .with_component(component::Name {
                    name: options.name.clone(),
                })
                .with_component(component::TileBlocker);

            let entity = builder.build();
            if let Some(priority) = options.priority.clone() {
                buffer.add_component(entity, component::Priority{value: priority.value})
            }

            if let Some(display) = options.display_cabinet {
                if display {
                    buffer.add_component(entity, component::DisplayCabinet{contents: None})
                }
            }

            entity

        }

    }
}
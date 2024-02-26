//! Terrain type that grows to cover the world but may also be eaten by rabbits.
use super::*;

const GRASS_DELTA: u8 = 8;

pub struct Grass {
    height: u8,
}
register_type!(Grass);

pub fn add_grass(world: &mut World, loc: Point) {
    let mut actor = Component::new();
    add_object!(actor, Grass, Grass::new(), [Action, Render, Fodder]);
    world.add(loc, actor);
}

impl Grass {
    pub fn new() -> Grass {
        Grass { height: 1 }
    }
}

impl Fodder for Grass {
    fn eat(&mut self, world: &mut World, id: ComponentId, loc: Point, percent: i32) {
        if self.height <= percent as u8 {
            // TODO: use as percent
            world.remove(id, loc);
        } else {
            self.height -= percent as u8;
        }
    }
}

impl Action for Grass {
    fn act(&mut self, world: &mut World, _component: &Component, loc: Point) -> bool {
        // Grass grows slowly.
        if self.height < u8::MAX - GRASS_DELTA {
            self.height += GRASS_DELTA;
        }

        // Once grass has grown enough it starts spreading.
        if self.height > 2 * GRASS_DELTA {
            for neighbor in world.all(loc, 1, |pt| {
                world
                    .cell(pt)
                    .iter()
                    .all(|id| !has_trait!(world.get(*id), Fodder))
            }) {
                add_grass(world, neighbor);
            }
        }
        true
    }
}

impl Render for Grass {
    fn render(&self) -> char {
        assert!(self.height > 0);

        // can't use match with math
        if self.height <= 2 * GRASS_DELTA {
            '~'
        } else if self.height <= 4 * GRASS_DELTA {
            '|'
        } else {
            '!'
        }
    }
}

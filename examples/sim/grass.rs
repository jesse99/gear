//! Terrain type that grows to cover the world but may also be eaten by rabbits.
use super::*;

const GRASS_DELTA: u8 = 8;

pub struct Grass {
    height: u8,
}
register_type!(Grass);

pub fn add_grass(world: &mut World, store: &Store, loc: Point) {
    let mut component = Component::new();
    add_object!(component, Grass, Grass::new(), [Action, Render, Fodder]);
    world.add(store, loc, component);
}

impl Grass {
    pub fn new() -> Grass {
        Grass { height: 1 }
    }
}

impl Fodder for Grass {
    fn eat<'a, 'b>(&mut self, context: Context<'a, 'b>, percent: i32) {
        let delta = (percent * 255 / 100) as u8;
        if self.height <= delta {
            context.world.remove(context.store, context.id, context.loc);
        } else {
            self.height -= delta;
        }
    }
}

impl Action for Grass {
    fn act<'a, 'b>(&mut self, context: Context<'a, 'b>) -> bool {
        // Grass grows slowly.
        if self.height < u8::MAX - GRASS_DELTA {
            self.height += GRASS_DELTA;
        }

        // Once grass has grown enough it starts spreading.
        if self.height > 2 * GRASS_DELTA {
            for neighbor in context.world.all(context.loc, 1, |pt| {
                context
                    .world
                    .cell(pt)
                    .iter()
                    .all(|id| pt != context.loc && !has_trait!(context.store.get(*id), Fodder))
            }) {
                add_grass(context.world, context.store, neighbor);
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

//! Fodder type that grows to cover the world but may also be eaten by rabbits.
use super::*;
use colored::*;

const GRASS_DELTA: u8 = 4; // amount by which grass grows each tick
const INITIAL_HEIGHT: u8 = 48;
const SPREAD_HEIGHT: u8 = 48;

struct Grass {
    height: u8,
}
register_type!(Grass);

pub fn add_grass(world: &mut World, store: &Store, loc: Point) {
    let mut component = Component::new();
    add_object!(
        component,
        Grass,
        Grass::new(INITIAL_HEIGHT),
        [Action, Render, Fodder]
    );
    world.add_front(store, loc, component);
}

pub fn spread_grass(world: &mut World, store: &Store, loc: Point) {
    let mut component = Component::new();
    add_object!(component, Grass, Grass::new(1), [Action, Render, Fodder]);
    world.add_front(store, loc, component);
}

impl Grass {
    pub fn new(height: u8) -> Grass {
        Grass { height }
    }
}

impl Fodder for Grass {
    fn height(&self) -> u8 {
        self.height
    }

    fn eat<'a, 'b>(&mut self, context: Context<'a, 'b>, percent: i32) {
        let delta = (percent * 255 / 100) as u8;
        if self.height <= delta {
            if context.world.verbose >= 2 {
                println!("   grass is now gone");
            }
            context.world.remove(context.store, context.id, context.loc);
        } else {
            self.height -= delta;
            if context.world.verbose >= 2 {
                println!(
                    "   grass went from height {} to {}",
                    self.height + delta,
                    self.height
                );
            }
        }
    }
}

impl Action for Grass {
    fn act<'a, 'b>(&mut self, context: Context<'a, 'b>) -> LifeCycle {
        let mut details = String::new();

        // Grass grows slowly.
        if self.height < u8::MAX - GRASS_DELTA {
            self.height += GRASS_DELTA;
            if context.world.verbose >= 3 {
                details += &format!(" grew to {}", self.height);
            }
        }

        // Once grass has grown enough it starts spreading.
        if self.height > SPREAD_HEIGHT {
            for neighbor in context.world.all(context.loc, 1, |pt| {
                context
                    .world
                    .cell(pt)
                    .iter()
                    .all(|id| pt != context.loc && !has_trait!(context.store.get(*id), Fodder))
            }) {
                if context.world.rng().gen_range(0..16) == 0 {
                    spread_grass(context.world, context.store, neighbor);
                    if context.world.verbose >= 2 {
                        details += &format!(" spread to {neighbor}");
                    }
                }
            }
        }

        if !details.is_empty() {
            println!("grass at {} {}", context.loc, details);
        }
        LifeCycle::Alive
    }
}

impl Render for Grass {
    fn render(&self) -> ColoredString {
        assert!(self.height > 0);
        if self.height < SPREAD_HEIGHT {
            "~".normal()
        } else {
            "|".normal()
        }
    }
}

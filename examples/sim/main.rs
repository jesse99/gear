#![feature(lazy_cell)]
#![feature(ptr_metadata)]
#![feature(unsize)]

use core::sync::atomic::Ordering;
use gear::*;
use paste::paste;

mod point;
mod world;

use point::*;
use world::*;

trait Action {
    /// Returns true if the actor is still alive.
    fn act(&mut self, world: &mut World, loc: Point) -> bool; // TODO: use an enum instead of a bool
}
register_type!(Action);

trait Render {
    fn render(&self) -> char;
}
register_type!(Render);

/// Marker trait for things like grass. Could turn this into a real trait by adding
/// something like an is_passable method.
trait Terrain {}
register_type!(Terrain);

struct Grass {
    height: u8,
}
register_type!(Grass);

impl Grass {
    fn new() -> Grass {
        Grass { height: 1 }
    }
}

impl Terrain for Grass {}

const GRASS_DELTA: u8 = 8;

impl Action for Grass {
    fn act(&mut self, world: &mut World, loc: Point) -> bool {
        // Grass grows slowly.
        if self.height < u8::MAX - GRASS_DELTA {
            self.height += GRASS_DELTA;
        }

        // Once grass has grown enough it starts spreading.
        if self.height > 2 * GRASS_DELTA {
            for neighbor in world.all(loc, 1, |p| world.get_terrain(p).is_none()) {
                add_grass(world, neighbor);
            }
        }
        true
    }
}

impl Render for Grass {
    fn render(&self) -> char {
        assert!(self.height > 0);
        if self.height <= 2 * GRASS_DELTA {
            // can't use match with math
            '~'
        } else if self.height <= 4 * GRASS_DELTA {
            '|'
        } else {
            '!'
        }
    }
}

fn add_grass(world: &mut World, loc: Point) {
    let grass = Grass::new();

    let mut actor = Component::new();
    add_object!(actor, Grass, grass, [Action, Render, Terrain]);
    world.add(loc, actor);
}

// TODO:
// populate with some random grass
// add rabbits
// rabbits should eat grass
// rabbits should reproduce
// add wolves
// wolves should eat rabbits
// wolves should reproduce
// use termion?
fn main() {
    let mut world = World::new(20, 20);

    add_grass(&mut world, Point::new(5, 5));
    add_grass(&mut world, Point::new(6, 5));
    add_grass(&mut world, Point::new(7, 5));
    add_grass(&mut world, Point::new(6, 6));

    for _ in 0..20 {
        world.render();
        world.step();
    }
}

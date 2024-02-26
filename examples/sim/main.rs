#![feature(lazy_cell)]
#![feature(ptr_metadata)]
#![feature(unsize)]

use core::sync::atomic::Ordering;
use gear::*;
use paste::paste;
use rand::rngs::StdRng;
use rand::Rng;
use rand::{RngCore, SeedableRng};

mod grass;
mod point;
mod rabbit;
mod traits;
mod world;

use grass::*;
use point::*;
use rabbit::*;
use traits::*;
use world::*;

fn add_grass_patch(world: &mut World, center: Point, radius: i32) {
    for dy in -radius..=radius {
        let y = center.y + dy;
        if y >= 0 && y < world.height {
            for dx in -radius..=radius {
                let x = center.x + dx;
                if x >= 0 && x < world.width {
                    let loc = Point::new(x, y);
                    if center.distance2(loc) < radius {
                        if world.cell(loc).is_empty() {
                            add_grass(world, loc);
                        }
                    }
                }
            }
        }
    }
}

// TODO:
// add rabbits, think we want a Fodder trait (possibly get rid of Terrain)
// rabbits should eat grass (will need to fixup step if grass is all eaten up)
// rabbits should starve (leave a skeleton behind for a bit? kind of interferes tho)
// rabbits should reproduce
// add wolves
// wolves should eat rabbits
// wolves should starve (leave a skeleton behind for a bit?)
// wolves should reproduce
// use termion? or just use command line options to configure?
//    seed, width/height, maybe debug (prints extra state)
//    grass patch params?
// track stats over time?
// add some sort of readme
fn main() {
    let rng = StdRng::seed_from_u64(1);
    let mut world = World::new(20, 20, Box::new(rng));

    for _ in 0..4 {
        let radius: i32 = world.rng.gen_range(1..10);
        let center = Point::new(
            world.rng.gen_range(0..world.width),
            world.rng.gen_range(0..world.height),
        );
        add_grass_patch(&mut world, center, radius);
    }

    for _ in 0..8 {
        let loc = Point::new(
            world.rng.gen_range(0..world.width),
            world.rng.gen_range(0..world.height),
        );
        add_rabbit(&mut world, loc);
    }

    for _ in 0..20 {
        world.render();
        world.step();
    }
}

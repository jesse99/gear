#![feature(lazy_cell)]
#![feature(ptr_metadata)]
#![feature(unsize)]

use core::sync::atomic::Ordering;
use gear::*;
use paste::paste;
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};

mod grass;
mod point;
mod traits;
mod world;

use grass::*;
use point::*;
use traits::*;
use world::*;

// TODO:
// radomize step function, use flex in dependency specs
// randomize grass location, should add patches (can overlap)
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
// track stats over time?
// add some sort of readme
fn main() {
    let rng = StdRng::seed_from_u64(1);
    let mut world = World::new(20, 20, Box::new(rng));

    add_grass(&mut world, Point::new(5, 5));
    add_grass(&mut world, Point::new(6, 5));
    add_grass(&mut world, Point::new(7, 5));
    add_grass(&mut world, Point::new(6, 6));

    for _ in 0..10 {
        world.render();
        world.step();
    }
}

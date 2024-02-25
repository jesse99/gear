#![feature(lazy_cell)]
#![feature(ptr_metadata)]
#![feature(unsize)]

use core::sync::atomic::Ordering;
use gear::*;
use paste::paste;

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
// add rabbits
// rabbits should eat grass
// rabbits should starve (leave a skeleton behind for a bit? kind of interferes tho)
// rabbits should reproduce
// add wolves
// wolves should eat rabbits
// wolves should starve (leave a skeleton behind for a bit?)
// wolves should reproduce
// use termion? or just use command line options to configure?
// track stats over time?
// add some sort of readme
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

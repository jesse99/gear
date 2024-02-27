#![feature(lazy_cell)]
#![feature(ptr_metadata)]
#![feature(unsize)]

use chrono::Utc;
use clap::Parser;
use core::sync::atomic::Ordering;
use gear::*;
use paste::paste;
use rand::rngs::StdRng;
use rand::Rng;
use rand::{RngCore, SeedableRng};

mod grass;
mod point;
mod rabbit;
mod skeleton;
mod store;
mod traits;
mod world;

use grass::*;
use point::*;
use rabbit::*;
use skeleton::*;
use store::*;
use traits::*;
use world::*;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Number of grass patchs to start with
    #[clap(long, value_name = "COUNT", default_value_t = 4)]
    grass: i32,

    /// Number of rabbits to start with
    #[clap(long, value_name = "COUNT", default_value_t = 8)]
    rabbits: i32,

    /// Random number seed (defaults to random)
    #[clap(long, value_name = "NUM")]
    seed: Option<u64>,

    /// Number of times to run the sim
    #[clap(long, value_name = "NUM", default_value_t = 10)]
    ticks: i32,

    /// Print extra information (up to -vvv)
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn add_grass_patch(world: &mut World, store: &Store, center: Point, radius: i32) {
    for dy in -radius..=radius {
        let y = center.y + dy;
        if y >= 0 && y < world.height {
            for dx in -radius..=radius {
                let x = center.x + dx;
                if x >= 0 && x < world.width {
                    let loc = Point::new(x, y);
                    if center.distance2(loc) < radius {
                        if world.cell(loc).is_empty() {
                            add_grass(world, store, loc);
                        }
                    }
                }
            }
        }
    }
}

// TODO:
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
    let options = Args::parse();

    let seed = options.seed.unwrap_or(Utc::now().timestamp_millis() as u64);
    let mut rng = StdRng::seed_from_u64(seed);
    let mut world = World::new(20, 20, Box::new(rng.clone()), options.verbose);
    let mut store = Store::new();

    for _ in 0..options.grass {
        let radius: i32 = rng.gen_range(1..10);
        let center = Point::new(
            rng.gen_range(0..world.width),
            rng.gen_range(0..world.height),
        );
        add_grass_patch(&mut world, &store, center, radius);
    }

    for _ in 0..options.rabbits {
        let loc = Point::new(
            rng.gen_range(0..world.width),
            rng.gen_range(0..world.height),
        );
        add_rabbit(&mut world, &store, loc);
    }

    store.sync();
    world.render(&store);
    for _ in 0..options.ticks {
        world.step(&mut store);
        world.render(&store);
    }
}

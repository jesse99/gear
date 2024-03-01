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
mod hungers;
mod mover;
mod point;
mod rabbit;
mod skeleton;
mod store;
mod traits;
mod wolf;
mod world;

use grass::*;
use hungers::*;
use mover::*;
use point::*;
use rabbit::*;
use skeleton::*;
use store::*;
use traits::*;
use wolf::*;
use world::*;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Number of grass patches to start with
    #[clap(long, value_name = "COUNT", default_value_t = 20)]
    grass: i32,

    /// Describe map symbols and exit
    #[clap(long)]
    legend: bool,

    /// Number of rabbits to start with
    #[clap(long, value_name = "COUNT", default_value_t = 12)]
    rabbits: i32,

    /// Random number seed [default: random]
    #[clap(long, value_name = "NUM")]
    seed: Option<u64>,

    /// Number of times to run the sim
    #[clap(long, value_name = "NUM", default_value_t = 10)]
    ticks: i32,

    /// Print extra information (up to -vvv)
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Number of wolves to start with
    #[clap(long, value_name = "COUNT", default_value_t = 3)]
    wolves: i32,
}

fn add_grass_patch(world: &mut World, store: &Store, center: Point, radius: i32) {
    for dy in -radius..=radius {
        let y = center.y + dy;
        for dx in -radius..=radius {
            let x = center.x + dx;
            let loc = Point::new(x, y);
            if world.distance2(center, loc) < radius {
                if world.cell(loc).is_empty() {
                    add_grass(world, store, loc);
                }
            }
        }
    }
}

fn print_legend() {
    println!("~ is short grass");
    println!("| is tall grass");
    println!("r is a rabbit");
    println!("w is a wolf");
    println!("* is the skeleton of a rabbit or wolf");
    println!();
    println!("Newborn rabbits and wolves are green.");
    println!("New skeletons are red.");
}

// Sim is loosely based on http://www.shodor.org/interactivate/activities/RabbitsAndWolves
// (there's not quite enough there to fully specify how the sim should behave).
fn run_sim(options: Args) {
    const WIDTH: i32 = 30;
    const HEIGHT: i32 = 20;

    let seed = options.seed.unwrap_or(Utc::now().timestamp_millis() as u64);
    let mut rng = StdRng::seed_from_u64(seed);
    let mut world = World::new(WIDTH, HEIGHT, Box::new(rng.clone()), options.verbose);
    let mut store = Store::new();

    for _ in 0..options.grass {
        let radius: i32 = rng.gen_range(1..20);
        let center = Point::new(rng.gen_range(0..WIDTH), rng.gen_range(0..HEIGHT));
        add_grass_patch(&mut world, &store, center, radius);
    }

    for _ in 0..options.rabbits {
        let loc = Point::new(rng.gen_range(0..WIDTH), rng.gen_range(0..HEIGHT));
        add_rabbit(&mut world, &store, loc);
    }

    for _ in 0..options.wolves {
        let loc = Point::new(rng.gen_range(0..WIDTH), rng.gen_range(0..HEIGHT));
        add_wolf(&mut world, &store, loc);
    }

    store.sync();
    world.render(&store);
    for _ in 0..options.ticks {
        world.step(&mut store);
        if world.render(&store) == LifeCycle::Dead {
            println!("Stopping early: world has stabilized");
            break;
        }
    }
}

fn main() {
    let options = Args::parse();
    if options.legend {
        print_legend();
    } else {
        run_sim(options);
    }
}

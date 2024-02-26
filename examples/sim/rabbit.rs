//! Animal that eats grass and is eaten by wolves.
use rand::seq::IteratorRandom;

use super::*;

const VISION_RADIUS: i32 = 4; // rabbits don't have great vision

pub struct Rabbit {}
register_type!(Rabbit);

pub fn add_rabbit(world: &mut World, loc: Point) {
    let mut actor = Component::new();
    add_object!(actor, Rabbit, Rabbit::new(), [Action, Animal, Render]);
    world.add(loc, actor);
}

impl Rabbit {
    pub fn new() -> Rabbit {
        Rabbit {}
    }

    fn has_grass(&self, world: &mut World, loc: Point) -> bool {
        world
            .cell(loc)
            .iter()
            .any(|id| has_trait!(world.get(*id), Terrain)) // TODO: use Fodder
    }

    fn move_towards_grass(&self, world: &mut World, loc: Point) -> Option<Point> {
        let mut dst = None;
        let mut dist = i32::MAX;

        for neighbor in world.all(loc, VISION_RADIUS, |pt| {
            world
                .cell(pt)
                .iter()
                .any(|id| has_trait!(world.get(*id), Terrain)) // TODO: use Fodder
        }) {
            let candidate = neighbor.distance2(loc);
            if candidate < dist {
                dst = Some(neighbor);
                dist = candidate;
            }
        }
        dst
    }

    fn random_move(&self, world: &mut World, loc: Point) -> Option<Point> {
        // First try to move to a cell without another rabbit (or wolf).
        let neighbors = world.all(loc, 1, |pt| {
            world
                .cell(pt)
                .iter()
                .all(|id| !has_trait!(world.get(*id), Animal))
        });
        let choice = neighbors.iter().choose(&mut world.rng).copied();
        if choice.is_some() {
            return choice;
        }

        // Then try to move anywhere.
        let neighbors = world.all(loc, 1, |_pt| true);
        neighbors.iter().choose(&mut world.rng).copied()
    }

    fn move_towards(&self, world: &World, loc: Point, dst: Point) -> Option<Point> {
        let mut new_loc = None;
        let mut dist = i32::MAX;

        for dy in -1..=1 {
            let y = loc.y + dy;
            if y >= 0 && y < world.height {
                for dx in -1..=1 {
                    let x = loc.x + dx;
                    if x >= 0 && x < world.width {
                        let candidate = Point::new(x, y);
                        let d = candidate.distance2(dst);
                        if d < dist {
                            new_loc = Some(candidate);
                            dist = d;
                        }
                    }
                }
            }
        }
        new_loc
    }
}

// TODO: finish this up
impl Action for Rabbit {
    // TODO: might want to add some logging
    fn act(&mut self, world: &mut World, id: ComponentId, loc: Point) -> bool {
        // if hunger is maxed then die

        // if wolves are seen then attempt to move to a square furthest from the wolves
        //    (compare total distance to all the wolves with adjacent cells)

        // if hunger is low then reproduce
        //    both rabbits should be hungry afterwards

        // if there is grass in the cell then eat it
        if self.has_grass(world, loc) {
            // TODO: eat the grass, if full don't eat: just bail
            // println!("rabbit at {loc} is eating grass");
            return true;
        }
        // TODO: otherwise get hungrier

        // move closer to grass
        if let Some(dst) = self.move_towards_grass(world, loc) {
            if let Some(new_loc) = self.move_towards(&world, loc, dst) {
                // println!("rabbit at {loc} is moving to {new_loc} towards {dst}");
                world.move_to(id, loc, new_loc);
                return true;
            } else {
                // println!("rabbit at {loc} can't move to {dst}");
            }
        }

        // random move
        if let Some(new_loc) = self.random_move(world, loc) {
            // println!("rabbit at {loc} is doing random move to {new_loc}");
            world.move_to(id, loc, new_loc);
            return true;
        }

        // do nothing
        // println!("rabbit at {loc} did nothing");
        true // TODO: use an enum
    }
}

impl Render for Rabbit {
    fn render(&self) -> char {
        'r' // TODO: maybe use 'R' if there is grass in the cell
    }
}

impl Animal for Rabbit {}

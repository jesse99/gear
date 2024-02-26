//! Animal that eats grass and is eaten by wolves.
use rand::seq::IteratorRandom;

use super::*;

const VISION_RADIUS: i32 = 4; // rabbits don't have great vision

const MAX_HUNGER: i32 = 100;

pub struct Rabbit {
    hunger: i32, // [0, MAX_HUNGER)
}
register_type!(Rabbit);

pub fn add_rabbit(world: &mut World, store: &Store, loc: Point) {
    let mut component = Component::new();
    add_object!(component, Rabbit, Rabbit::new(), [Action, Animal, Render]);
    world.add(store, loc, component);
}

impl Rabbit {
    pub fn new() -> Rabbit {
        Rabbit {
            hunger: MAX_HUNGER / 2,
        }
    }

    fn find_grass<'a, 'b>(&self, context: &Context<'a, 'b>) -> Option<ComponentId> {
        context
            .world
            .cell(context.loc)
            .iter()
            .copied()
            .find(|id| has_trait!(context.store.get(*id), Fodder))
    }

    fn move_towards_grass<'a, 'b>(&self, context: &Context<'a, 'b>) -> Option<Point> {
        let mut dst = None;
        let mut dist = i32::MAX;

        for neighbor in context.world.all(context.loc, VISION_RADIUS, |pt| {
            context
                .world
                .cell(pt)
                .iter()
                .any(|id| has_trait!(context.store.get(*id), Fodder))
        }) {
            let candidate = neighbor.distance2(context.loc);
            if candidate < dist {
                dst = Some(neighbor);
                dist = candidate;
            }
        }
        dst
    }

    fn random_move<'a, 'b>(&self, context: &Context<'a, 'b>) -> Option<Point> {
        // First try to move to a cell without another rabbit (or wolf).
        let neighbors = context.world.all(context.loc, 1, |pt| {
            context
                .world
                .cell(pt)
                .iter()
                .all(|id| !has_trait!(context.store.get(*id), Animal))
        });
        let choice = neighbors
            .iter()
            .choose(context.world.rng().as_mut())
            .copied();
        if choice.is_some() {
            return choice;
        }

        // Then try to move anywhere.
        let neighbors = context.world.all(context.loc, 1, |_pt| true);
        neighbors
            .iter()
            .choose(context.world.rng().as_mut())
            .copied()
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
    fn act<'a, 'b>(&mut self, context: Context<'a, 'b>) -> bool {
        // if hunger is maxed then die

        // if wolves are seen then attempt to move to a square furthest from the wolves
        //    (compare total distance to all the wolves with adjacent cells)

        // if hunger is low then reproduce
        //    both rabbits should be hungry afterwards

        // if there is grass in the cell then eat it
        if let Some(grass_id) = self.find_grass(&context) {
            // TODO: eat the grass, if full don't eat: just bail
            // println!("rabbit at {loc} is eating grass");
            let new_context = Context {
                id: grass_id,
                ..context
            };
            let component = context.store.get(grass_id);
            let fodder = find_trait_mut!(component, Fodder).unwrap();
            fodder.eat(new_context, 25);
            return true;
        }
        // TODO: otherwise get hungrier

        // move closer to grass
        if let Some(dst) = self.move_towards_grass(&context) {
            if let Some(new_loc) = self.move_towards(context.world, context.loc, dst) {
                // println!("rabbit at {loc} is moving to {new_loc} towards {dst}");
                context.world.move_to(context.id, context.loc, new_loc);
                return true;
            } else {
                // println!("rabbit at {loc} can't move to {dst}");
            }
        }

        // random move
        if let Some(new_loc) = self.random_move(&context) {
            // println!("rabbit at {loc} is doing random move to {new_loc}");
            context.world.move_to(context.id, context.loc, new_loc);
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

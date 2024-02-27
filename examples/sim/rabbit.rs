//! Animal that eats grass and is eaten by wolves.
use rand::seq::IteratorRandom;

use super::*;

const VISION_RADIUS: i32 = 4; // rabbits don't have great vision

const MAX_HUNGER: i32 = 300;
const INITAL_HUNGER: i32 = 180;
const REPRO_HUNGER: i32 = 90;
const EAT_DELTA: i32 = -30;
const BASAL_DELTA: i32 = 5;

pub struct Rabbit {
    hunger: i32, // [0, MAX_HUNGER)
}
register_type!(Rabbit);

pub fn add_rabbit(world: &mut World, store: &Store, loc: Point) -> ComponentId {
    let mut component = Component::new();
    let id = component.id;
    add_object!(component, Rabbit, Rabbit::new(), [Action, Animal, Render]);
    world.add(store, loc, component);
    id
}

impl Rabbit {
    pub fn new() -> Rabbit {
        Rabbit {
            hunger: INITAL_HUNGER,
        }
    }

    pub fn adjust_hunger(&mut self, delta: i32) {
        if delta > 0 {
            if self.hunger <= MAX_HUNGER - delta {
                self.hunger += delta;
            } else {
                self.hunger = MAX_HUNGER;
            }
        } else {
            if self.hunger >= -delta {
                self.hunger += delta;
            } else {
                self.hunger = 0;
            }
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
        let mut height = 0;

        for neighbor in context.world.all(context.loc, VISION_RADIUS, |pt| {
            context
                .world
                .cell(pt)
                .iter()
                .any(|id| has_trait!(context.store.get(*id), Fodder))
        }) {
            for id in context.world.cell(neighbor) {
                let component = context.store.get(*id);
                if let Some(fodder) = find_trait!(component, Fodder) {
                    if fodder.height() > height {
                        // move towards cells that have more grass
                        dst = Some(neighbor);
                        dist = neighbor.distance2(context.loc);
                        height = fodder.height();
                    } else if fodder.height() == height {
                        // or to the closest cell for a particular height
                        let candidate = neighbor.distance2(context.loc);
                        if candidate < dist {
                            dst = Some(neighbor);
                            dist = candidate;
                        }
                    }
                }
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
                .all(|id| pt != context.loc && !has_trait!(context.store.get(*id), Animal))
        });
        let choice = neighbors
            .iter()
            .choose(context.world.rng().as_mut())
            .copied();
        if choice.is_some() {
            return choice;
        }

        // Then try to move anywhere.
        let neighbors = context.world.all(context.loc, 1, |pt| pt != context.loc);
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

impl Action for Rabbit {
    fn act<'a, 'b>(&mut self, context: Context<'a, 'b>) -> LifeCycle {
        // if wolves are seen then attempt to move to a square furthest from the wolves
        //    (compare total distance to all the wolves with adjacent cells)

        // If we're not hungry then reproduce.
        if self.hunger <= REPRO_HUNGER {
            self.hunger = INITAL_HUNGER;
            let new_id = add_rabbit(context.world, context.store, context.loc);
            if context.world.verbose >= 1 {
                println!(
                    "rabbit{} at {} is reproduced new rabbit{} (hunger is {})",
                    context.id, context.loc, new_id, self.hunger
                );
            }
            return LifeCycle::Alive;
        }

        // if we're hungry and there is grass in the cell then eat it
        if let Some(grass_id) = self.find_grass(&context) {
            self.adjust_hunger(EAT_DELTA);
            if context.world.verbose >= 1 {
                print!(
                    "rabbit{} at {} is eating grass (hunger is {})",
                    context.id, context.loc, self.hunger
                );
            }
            let new_context = Context {
                id: grass_id,
                ..context
            };
            let component = context.store.get(grass_id);
            let fodder = find_trait_mut!(component, Fodder).unwrap();
            fodder.eat(new_context, 25);
            return LifeCycle::Alive;
        } else {
            self.adjust_hunger(BASAL_DELTA);
            if self.hunger == MAX_HUNGER {
                if context.world.verbose >= 1 {
                    println!(
                        "rabbit{} at {} has starved to death",
                        context.id, context.loc
                    );
                }
                return LifeCycle::Dead;
            }
        }

        // move closer to grass
        if let Some(dst) = self.move_towards_grass(&context) {
            if let Some(new_loc) = self.move_towards(context.world, context.loc, dst) {
                if context.world.verbose >= 1 {
                    println!(
                        "rabbit{} at {} is moving to {new_loc} towards {dst} (hunger is {})",
                        context.id, context.loc, self.hunger
                    );
                }
                context.world.move_to(context.id, context.loc, new_loc);
                return LifeCycle::Alive;
            } else {
                if context.world.verbose >= 1 {
                    println!(
                        "rabbit{} at {} can't move to {dst} (hunger is {})",
                        context.id, context.loc, self.hunger
                    );
                }
            }
        }

        // random move
        if let Some(new_loc) = self.random_move(&context) {
            if context.world.verbose >= 1 {
                println!(
                    "rabbit{} at {} is doing random move to {new_loc} (hunger is {})",
                    context.id, context.loc, self.hunger
                );
            }
            context.world.move_to(context.id, context.loc, new_loc);
            return LifeCycle::Alive;
        }

        // do nothing
        if context.world.verbose >= 1 {
            println!(
                "rabbit{} at {} did nothing (hunger is {})",
                context.id, context.loc, self.hunger
            );
        }
        LifeCycle::Alive
    }
}

impl Render for Rabbit {
    fn render(&self) -> char {
        'r' // TODO: maybe use 'R' if there is grass in the cell
    }
}

impl Animal for Rabbit {}

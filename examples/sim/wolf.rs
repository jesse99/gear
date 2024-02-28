//! Animal that eats grass and is eaten by wolves.
use std::option;

use super::*;
use rand::seq::IteratorRandom;

const VISION_RADIUS: i32 = 8; // wolves see quite a bit better than rabbits

const MAX_HUNGER: i32 = 400;
const INITAL_HUNGER: i32 = 300;
const REPRO_HUNGER: i32 = 200;
const EAT_DELTA: i32 = -50;
const BASAL_DELTA: i32 = 5;

pub struct Wolf {}
register_type!(Wolf);

pub fn add_wolf(world: &mut World, store: &Store, loc: Point) -> ComponentId {
    let mut component = Component::new();
    let id = component.id;
    add_object!(
        component,
        Wolf,
        Wolf::new(),
        [Action, Animal, Predator, Render]
    );
    add_object!(component, Mover, Mover::new(), [Moveable]);
    add_object!(
        component,
        Hungers,
        Hungers::new(INITAL_HUNGER, MAX_HUNGER),
        [Hunger]
    );
    world.add(store, loc, component);
    id
}

pub fn find_prey(world: &World, store: &Store, loc: Point) -> Option<ComponentId> {
    world
        .cell(loc)
        .iter()
        .copied()
        .find(|id| has_trait!(store.get(*id), Prey))
}

pub fn find_prey_cell<'a, 'b>(context: &Context<'a, 'b>) -> Option<(Point, ComponentId)> {
    let mut candidates = Vec::new();
    for dy in -1..=1 {
        for dx in -1..=1 {
            let candidate = Point::new(context.loc.x + dx, context.loc.y + dy);
            if candidate != context.loc
                && candidate.x >= 0
                && candidate.y >= 0
                && candidate.x < context.world.width
                && candidate.y < context.world.height
            {
                if let Some(id) = find_prey(context.world, context.store, candidate) {
                    candidates.push((candidate, id));
                }
            }
        }
    }
    candidates
        .iter()
        .copied()
        .choose(context.world.rng().as_mut())
}

impl Wolf {
    pub fn new() -> Wolf {
        Wolf {}
    }

    fn move_towards_prey<'a, 'b>(&self, context: &Context<'a, 'b>) -> Option<Point> {
        let mut dst = None;
        let mut dist = i32::MAX;

        for neighbor in context.world.all(context.loc, VISION_RADIUS, |pt| {
            context
                .world
                .cell(pt)
                .iter()
                .any(|id| has_trait!(context.store.get(*id), Prey))
        }) {
            let candidate = neighbor.distance2(context.loc);
            if candidate < dist {
                dst = Some(neighbor);
                dist = candidate;
            }
        }
        dst
    }
}

impl Action for Wolf {
    fn act<'a, 'b>(&mut self, context: Context<'a, 'b>) -> LifeCycle {
        // If we're not hungry then reproduce.
        let component = context.store.get(context.id);
        let hunger = find_trait_mut!(component, Hunger).unwrap();
        if hunger.get() <= REPRO_HUNGER {
            if let Some(neighbor) = find_empty_cell(context.world, context.store, context.loc) {
                hunger.set(INITAL_HUNGER);
                let new_id = add_wolf(context.world, context.store, neighbor);
                if context.world.verbose >= 1 {
                    println!(
                        "wolf{} at {} reproduced new wolf{} (hunger is {})",
                        context.id,
                        context.loc,
                        new_id,
                        hunger.get()
                    );
                }
                return LifeCycle::Alive;
            }
        }

        // if we're hungry and there is prey nearby then eat it
        if let Some((neighbor, prey_id)) = find_prey_cell(&context) {
            hunger.adjust(EAT_DELTA);
            if context.world.verbose >= 1 {
                println!(
                    "wolf{} at {} is eating a rabbit (hunger is {})",
                    context.id,
                    context.loc,
                    hunger.get()
                );
            }
            context.world.remove(context.store, prey_id, neighbor);
            return LifeCycle::Alive;
        } else {
            hunger.adjust(BASAL_DELTA);
            if hunger.get() == MAX_HUNGER {
                if context.world.verbose >= 1 {
                    println!("wolf{} at {} has starved to death", context.id, context.loc);
                }
                add_skeleton(context.world, context.store, context.loc);
                return LifeCycle::Dead;
            }
        }

        // move closer to prey
        let movable = find_trait!(component, Moveable).unwrap();
        if let Some(dst) = self.move_towards_prey(&context) {
            if let Some(new_loc) = movable.move_towards(context.world, context.loc, dst) {
                if context.world.verbose >= 1 {
                    println!(
                        "wolf{} at {} is moving to {new_loc} towards {dst} (hunger is {})",
                        context.id,
                        context.loc,
                        hunger.get()
                    );
                }
                context.world.move_to(context.id, context.loc, new_loc);
                return LifeCycle::Alive;
            } else {
                if context.world.verbose >= 1 {
                    println!(
                        "wolf{} at {} can't move to {dst} (hunger is {})",
                        context.id,
                        context.loc,
                        hunger.get()
                    );
                }
            }
        }

        // random move
        if let Some(new_loc) = movable.random_move(&context) {
            if context.world.verbose >= 1 {
                println!(
                    "wolf{} at {} is doing random move to {new_loc} (hunger is {})",
                    context.id,
                    context.loc,
                    hunger.get()
                );
            }
            context.world.move_to(context.id, context.loc, new_loc);
            return LifeCycle::Alive;
        }

        // do nothing
        if context.world.verbose >= 1 {
            println!(
                "rabbit{} at {} did nothing (hunger is {})",
                context.id,
                context.loc,
                hunger.get()
            );
        }
        LifeCycle::Alive
    }
}

impl Render for Wolf {
    fn render(&self) -> char {
        'w' // TODO: maybe use 'W' if there is a rabbit in the cell
    }
}

impl Animal for Wolf {}
impl Predator for Wolf {}

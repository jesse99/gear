//! Animal that eats grass and is eaten by wolves.
use super::*;

const VISION_RADIUS: i32 = 8; // wolves see quite a bit better than rabbits

const MAX_HUNGER: i32 = 400;
const INITAL_HUNGER: i32 = 300;
const REPRO_HUNGER: i32 = 200;
const EAT_DELTA: i32 = -50;
const BASAL_DELTA: i32 = 5;

pub struct Wolf {
    hunger: i32, // [0, MAX_HUNGER)
}
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
    world.add(store, loc, component);
    id
}

impl Wolf {
    pub fn new() -> Wolf {
        Wolf {
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

    fn find_prey<'a, 'b>(&self, context: &Context<'a, 'b>) -> Option<ComponentId> {
        context
            .world
            .cell(context.loc)
            .iter()
            .copied()
            .find(|id| has_trait!(context.store.get(*id), Prey))
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
        if self.hunger <= REPRO_HUNGER {
            self.hunger = INITAL_HUNGER;
            let new_id = add_wolf(context.world, context.store, context.loc);
            if context.world.verbose >= 1 {
                println!(
                    "wolf{} at {} is reproduced new wolf{} (hunger is {})",
                    context.id, context.loc, new_id, self.hunger
                );
            }
            return LifeCycle::Alive;
        }

        // if we're hungry and there is prey in the cell then eat it
        if let Some(prey_id) = self.find_prey(&context) {
            self.adjust_hunger(EAT_DELTA);
            if context.world.verbose >= 1 {
                println!(
                    "wolf{} at {} is eating a rabbit (hunger is {})",
                    context.id, context.loc, self.hunger
                );
            }
            context.world.remove(context.store, prey_id, context.loc);
            return LifeCycle::Alive;
        } else {
            self.adjust_hunger(BASAL_DELTA);
            if self.hunger == MAX_HUNGER {
                if context.world.verbose >= 1 {
                    println!("wolf{} at {} has starved to death", context.id, context.loc);
                }
                add_skeleton(context.world, context.store, context.loc);
                return LifeCycle::Dead;
            }
        }

        // move closer to prey
        let component = context.store.get(context.id);
        let movable = find_trait!(component, Moveable).unwrap();
        if let Some(dst) = self.move_towards_prey(&context) {
            if let Some(new_loc) = movable.move_towards(context.world, context.loc, dst) {
                if context.world.verbose >= 1 {
                    println!(
                        "wolf{} at {} is moving to {new_loc} towards {dst} (hunger is {})",
                        context.id, context.loc, self.hunger
                    );
                }
                context.world.move_to(context.id, context.loc, new_loc);
                return LifeCycle::Alive;
            } else {
                if context.world.verbose >= 1 {
                    println!(
                        "wolf{} at {} can't move to {dst} (hunger is {})",
                        context.id, context.loc, self.hunger
                    );
                }
            }
        }

        // random move
        if let Some(new_loc) = movable.random_move(&context) {
            if context.world.verbose >= 1 {
                println!(
                    "wolf{} at {} is doing random move to {new_loc} (hunger is {})",
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

impl Render for Wolf {
    fn render(&self) -> char {
        'w' // TODO: maybe use 'W' if there is a rabbit in the cell
    }
}

impl Animal for Wolf {}
impl Predator for Wolf {}

//! Animal that eats grass and is eaten by wolves.
use super::*;
use colored::*;
use rand::seq::IteratorRandom;

const VISION_RADIUS: i32 = 8; // wolves see quite a bit better than rabbits

const MAX_HUNGER: i32 = 200; // starves when hit this
const INITAL_HUNGER: i32 = 120;
const REPRO_HUNGER: i32 = 80;
const EAT_DELTA: i32 = -20;
const BASAL_DELTA: i32 = 2;

const REPRO_AGE: i32 = 10;
const MAX_AGE: i32 = 50;

#[derive(Debug)]
struct Wolf {
    age: i32,
}
register_type!(Wolf);

pub fn add_wolf(world: &mut World, store: &Store, loc: Point) -> ComponentId {
    use core::fmt::Debug;
    let mut component = Component::new("wolf");
    let id = component.id;
    add_object!(
        component,
        Wolf,
        Wolf::new(),
        [Action, Animal, Predator, Render],
        [Debug]
    );
    add_object!(component, Mover, Mover::new(), [Moveable]);
    add_object!(
        component,
        Hungers,
        Hungers::new(INITAL_HUNGER, MAX_HUNGER),
        [Hunger],
        [Debug]
    );
    world.add_back(store, loc, component);
    id
}

fn find_prey(world: &World, store: &Store, loc: Point) -> Option<ComponentId> {
    world
        .cell(loc)
        .iter()
        .copied()
        .find(|id| has_trait!(store.get(*id), Prey))
}

fn find_prey_cell<'a, 'b>(context: &Context<'a, 'b>) -> Option<(Point, ComponentId)> {
    let mut candidates = Vec::new();
    for dy in -1..=1 {
        for dx in -1..=1 {
            let candidate = Point::new(context.loc.x + dx, context.loc.y + dy);
            if candidate != context.loc {
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
        Wolf { age: 0 }
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
            let candidate = context.world.distance2(neighbor, context.loc);
            if candidate < dist && candidate > 2 {
                dst = Some(neighbor);
                dist = candidate;
            }
        }
        dst
    }

    fn log<'a, 'b>(&self, context: &Context<'a, 'b>, suffix: &str) {
        if context.world.verbose >= 1 {
            let component = context.store.get(context.id);
            let hunger = find_trait_mut!(component, Hunger).unwrap();
            println!(
                "wolf{} loc: {} age: {} hunger: {} {}",
                context.id,
                context.loc,
                self.age,
                hunger.get(),
                suffix
            );
        }
    }
}

impl Action for Wolf {
    fn act<'a, 'b>(&mut self, context: Context<'a, 'b>) -> LifeCycle {
        self.age += 1;

        // Wolves can die of old age.
        if self.age >= MAX_AGE {
            self.log(&context, "died of old age");
            add_skeleton(context.world, context.store, context.loc);
            return LifeCycle::Dead;
        }

        // If we're not hungry then reproduce.
        let component = context.store.get(context.id);
        let hunger = find_trait_mut!(component, Hunger).unwrap();
        if hunger.get() <= REPRO_HUNGER
            && self.age >= REPRO_AGE
            && context.world.rng().gen_bool(0.5)
        {
            if let Some(neighbor) = find_empty_cell(context.world, context.store, context.loc) {
                hunger.set(INITAL_HUNGER);
                let new_id = add_wolf(context.world, context.store, neighbor);
                self.log(
                    &context,
                    &format!("reproduced new wolf{new_id} at {neighbor}"),
                );
                return LifeCycle::Alive;
            }
        }

        // if we're hungry and there is prey nearby then eat it
        if let Some((neighbor, prey_id)) = find_prey_cell(&context) {
            hunger.adjust(EAT_DELTA);
            context.world.remove(context.store, prey_id, neighbor);
            self.log(&context, &format!("ate rabbit{prey_id} at {neighbor}"));
            return LifeCycle::Alive;
        } else {
            hunger.adjust(BASAL_DELTA);
            if hunger.get() == MAX_HUNGER {
                self.log(&context, "starved to death");
                add_skeleton(context.world, context.store, context.loc);
                return LifeCycle::Dead;
            }
        }

        // move closer to prey
        let movable = find_trait!(component, Moveable).unwrap();
        if let Some(dst) = self.move_towards_prey(&context) {
            if let Some(new_loc) =
                movable.move_towards(context.world, context.store, context.loc, dst)
            {
                self.log(
                    &context,
                    &format!("moving to {new_loc} towards prey at {dst}"),
                );
                context.world.move_to(context.id, context.loc, new_loc);
                return LifeCycle::Alive;
            } else {
                self.log(&context, &format!("failed to move towards {dst}"));
            }
        }

        // random move
        if let Some(new_loc) = movable.random_move(&context) {
            self.log(&context, &format!("random move to {new_loc}"));
            context.world.move_to(context.id, context.loc, new_loc);
            return LifeCycle::Alive;
        }

        // do nothing
        self.log(&context, "is doing nothing");
        LifeCycle::Alive
    }
}

impl Render for Wolf {
    fn render(&self) -> ColoredString {
        if self.age == 0 {
            "w".green()
        } else {
            "w".normal()
        }
    }
}

impl Animal for Wolf {}
impl Predator for Wolf {}

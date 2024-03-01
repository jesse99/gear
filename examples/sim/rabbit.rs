//! Animal that eats grass and is eaten by wolves.
use super::*;
use colored::*;
use core::fmt::Debug;
use rand::seq::IteratorRandom;

const VISION_RADIUS: i32 = 4; // rabbits don't have great vision

const MAX_HUNGER: i32 = 45; // starves when hit this
const INITAL_HUNGER: i32 = 25;
const REPRO_HUNGER: i32 = 5;
const EAT_DELTA: i32 = -9;
const BASAL_DELTA: i32 = 3;

const REPRO_AGE: i32 = 10;
const MAX_AGE: i32 = 25;

#[derive(Debug)]
struct Rabbit {
    age: i32,
}
register_type!(Rabbit);

pub fn add_rabbit(world: &mut World, store: &Store, loc: Point) -> ComponentId {
    let mut component = Component::new("rabbit");
    let id = component.id;
    add_object!(
        component,
        Rabbit,
        Rabbit::new(),
        [Action, Animal, Prey, Render],
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

pub fn has_animal(world: &World, store: &Store, loc: Point) -> bool {
    world
        .cell(loc)
        .iter()
        .any(|id| has_trait!(store.get(*id), Animal))
}

pub fn find_empty_cell(world: &World, store: &Store, loc: Point) -> Option<Point> {
    let mut candidates = Vec::new();
    for dy in -1..=1 {
        for dx in -1..=1 {
            let candidate = Point::new(loc.x + dx, loc.y + dy);
            if candidate != loc {
                if !has_animal(world, store, candidate) {
                    candidates.push(candidate);
                }
            }
        }
    }
    candidates.iter().copied().choose(world.rng().as_mut())
}

fn find_predator(world: &World, store: &Store, loc: Point) -> Option<ComponentId> {
    world
        .cell(loc)
        .iter()
        .copied()
        .find(|id| has_trait!(store.get(*id), Predator))
}

fn predator_nearby<'a, 'b>(context: &Context<'a, 'b>) -> bool {
    for dy in -1..=1 {
        for dx in -1..=1 {
            let candidate = Point::new(context.loc.x + dx, context.loc.y + dy);
            if candidate != context.loc {
                if find_predator(context.world, context.store, candidate).is_some() {
                    return true;
                }
            }
        }
    }
    false
}

impl Rabbit {
    pub fn new() -> Rabbit {
        Rabbit { age: 0 }
    }

    fn find_grass<'a, 'b>(&self, context: &Context<'a, 'b>) -> Option<ComponentId> {
        context
            .world
            .cell(context.loc)
            .iter()
            .copied()
            .find(|id| has_trait!(context.store.get(*id), Fodder))
    }

    fn move_away_from_wolf<'a, 'b>(&self, context: &Context<'a, 'b>) -> Option<Point> {
        let mut dst = None;
        let mut dist = 0; // want to maximize distance from all visible wolves

        let wolves = context.world.all(context.loc, VISION_RADIUS, |pt| {
            context
                .world
                .cell(pt)
                .iter()
                .any(|id| has_trait!(context.store.get(*id), Predator))
        });

        for dy in -1..=1 {
            for dx in -1..=1 {
                let candidate = Point::new(context.loc.x + dx, context.loc.y + dy);
                if candidate != context.loc {
                    if !has_animal(context.world, context.store, candidate) {
                        let d = wolves
                            .iter()
                            .map(|pt| context.world.distance2(*pt, candidate))
                            .sum();
                        if d > dist {
                            dst = Some(candidate);
                            dist = d;
                        }
                    }
                }
            }
        }

        dst
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
            // If there are wolves around then we shouldn't land here.
            // But if there are rabbits around then it's possible we'll be blocked from
            // moving to the grass. But you could argue that rabbits are pretty dumb...
            if !has_animal(context.world, context.store, neighbor) {
                for id in context.world.cell(neighbor) {
                    let component = context.store.get(*id);
                    if let Some(fodder) = find_trait!(component, Fodder) {
                        if fodder.height() > height {
                            // move towards cells that have more grass
                            dst = Some(neighbor);
                            dist = context.world.distance2(neighbor, context.loc);
                            height = fodder.height();
                        } else if fodder.height() == height {
                            // or to the closest cell for a particular height
                            let candidate = context.world.distance2(neighbor, context.loc);
                            if candidate < dist {
                                dst = Some(neighbor);
                                dist = candidate;
                            }
                        }
                    }
                }
            }
        }
        dst
    }

    fn log<'a, 'b>(&self, context: &Context<'a, 'b>, suffix: &str) {
        if context.world.verbose >= 1 {
            let component = context.store.get(context.id);
            let hunger = find_trait_mut!(component, Hunger).unwrap();
            println!(
                "rabbit{} loc: {} age: {} hunger: {} {}",
                context.id,
                context.loc,
                self.age,
                hunger.get(),
                suffix
            );
        }
    }
}

impl Action for Rabbit {
    fn act<'a, 'b>(&mut self, context: Context<'a, 'b>) -> LifeCycle {
        self.age += 1;

        // Rabbits can die of old age.
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
            && !predator_nearby(&context)
            && context.world.rng().gen_bool(0.5)
        {
            if let Some(neighbor) = find_empty_cell(context.world, context.store, context.loc) {
                hunger.set(INITAL_HUNGER);
                let new_id = add_rabbit(context.world, context.store, neighbor);
                self.log(
                    &context,
                    &format!("reproduced new rabbit{new_id} at {neighbor}"),
                );
                return LifeCycle::Alive;
            }
        }

        // If there are visible wolves then move as far away as possible from them.
        if let Some(new_loc) = self.move_away_from_wolf(&context) {
            // It's hard for wolves to catch rabbits when they always flee so
            // occasionally we'll consider the rabbits too distracted to see wolves.
            if context.world.rng().gen_bool(0.8) {
                self.log(&context, &format!("moving away from wolves to {new_loc}"));
                context.world.move_to(context.id, context.loc, new_loc);
                return LifeCycle::Alive;
            }
        }

        // If we're hungry and there is grass in the cell then eat it.
        if let Some(grass_id) = self.find_grass(&context) {
            hunger.adjust(EAT_DELTA);
            self.log(&context, "ate grass");
            let new_context = Context {
                id: grass_id,
                ..context
            };
            let component = context.store.get(grass_id);
            let fodder = find_trait_mut!(component, Fodder).unwrap();
            fodder.eat(new_context, 25); // grass may die here
            return LifeCycle::Alive;
        } else {
            hunger.adjust(BASAL_DELTA);
            if hunger.get() == MAX_HUNGER {
                self.log(&context, "starved to death");
                add_skeleton(context.world, context.store, context.loc);
                return LifeCycle::Dead;
            }
        }

        // move closer to grass
        let movable = find_trait!(component, Moveable).unwrap();
        if let Some(dst) = self.move_towards_grass(&context) {
            if let Some(new_loc) =
                movable.move_towards(context.world, context.store, context.loc, dst)
            {
                self.log(
                    &context,
                    &format!("moving to {new_loc} towards grass at {dst}"),
                );
                context.world.move_to(context.id, context.loc, new_loc);
                return LifeCycle::Alive;
            } else {
                self.log(&context, &format!("failed to move towards grass at {dst}"));
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

impl Render for Rabbit {
    fn render(&self) -> ColoredString {
        if self.age == 0 {
            "r".green()
        } else {
            "r".normal()
        }
    }
}

impl Animal for Rabbit {}
impl Prey for Rabbit {}

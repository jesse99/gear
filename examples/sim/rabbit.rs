//! Animal that eats grass and is eaten by wolves.
use super::*;

const VISION_RADIUS: i32 = 4; // rabbits don't have great vision

const MAX_HUNGER: i32 = 300;
const INITAL_HUNGER: i32 = 180;
const REPRO_HUNGER: i32 = 90;
const EAT_DELTA: i32 = -30;
const BASAL_DELTA: i32 = 5;

pub struct Rabbit {}
register_type!(Rabbit);

pub fn add_rabbit(world: &mut World, store: &Store, loc: Point) -> ComponentId {
    let mut component = Component::new();
    let id = component.id;
    add_object!(
        component,
        Rabbit,
        Rabbit::new(),
        [Action, Animal, Prey, Render]
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

impl Rabbit {
    pub fn new() -> Rabbit {
        Rabbit {}
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
                if candidate != context.loc
                    && candidate.x >= 0
                    && candidate.y >= 0
                    && candidate.x < context.world.width
                    && candidate.y < context.world.height
                {
                    let d = wolves.iter().map(|pt| pt.distance2(candidate)).sum();
                    if d > dist {
                        dst = Some(candidate);
                        dist = d;
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
}

impl Action for Rabbit {
    fn act<'a, 'b>(&mut self, context: Context<'a, 'b>) -> LifeCycle {
        // If there are visible wolves then move as far as possible from them.
        let component = context.store.get(context.id);
        let hunger = find_trait_mut!(component, Hunger).unwrap();
        if let Some(new_loc) = self.move_away_from_wolf(&context) {
            // It's hard for wolves to catch rabbits when they flee so occasionally
            // we'll consider the rabbits too distracted to see wolves.
            if context.world.rng().gen_bool(0.7) {
                if context.world.verbose >= 1 {
                    println!(
                        "rabbit{} at {} is moving away from wolves to {new_loc} (hunger is {})",
                        context.id,
                        context.loc,
                        hunger.get()
                    );
                }
                context.world.move_to(context.id, context.loc, new_loc);
                return LifeCycle::Alive;
            }
        }

        // If we're not hungry then reproduce.
        if hunger.get() <= REPRO_HUNGER {
            hunger.set(INITAL_HUNGER);
            let new_id = add_rabbit(context.world, context.store, context.loc);
            if context.world.verbose >= 1 {
                println!(
                    "rabbit{} at {} is reproduced new rabbit{} (hunger is {})",
                    context.id,
                    context.loc,
                    new_id,
                    hunger.get()
                );
            }
            return LifeCycle::Alive;
        }

        // if we're hungry and there is grass in the cell then eat it
        if let Some(grass_id) = self.find_grass(&context) {
            hunger.adjust(EAT_DELTA);
            if context.world.verbose >= 1 {
                print!(
                    "rabbit{} at {} is eating grass (hunger is {})",
                    context.id,
                    context.loc,
                    hunger.get()
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
            hunger.adjust(BASAL_DELTA);
            if hunger.get() == MAX_HUNGER {
                if context.world.verbose >= 1 {
                    println!(
                        "rabbit{} at {} has starved to death",
                        context.id, context.loc
                    );
                }
                add_skeleton(context.world, context.store, context.loc);
                return LifeCycle::Dead;
            }
        }

        // move closer to grass
        let movable = find_trait!(component, Moveable).unwrap();
        if let Some(dst) = self.move_towards_grass(&context) {
            if let Some(new_loc) = movable.move_towards(context.world, context.loc, dst) {
                if context.world.verbose >= 1 {
                    println!(
                        "rabbit{} at {} is moving to {new_loc} towards {dst} (hunger is {})",
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
                        "rabbit{} at {} can't move to {dst} (hunger is {})",
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
                    "rabbit{} at {} is doing random move to {new_loc} (hunger is {})",
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

impl Render for Rabbit {
    fn render(&self) -> char {
        'r' // TODO: maybe use 'R' if there is grass in the cell
    }
}

impl Animal for Rabbit {}
impl Prey for Rabbit {}

//! Helper object for components that move around.
use super::*;
use rand::seq::IteratorRandom;

pub struct Mover {}
register_type!(Mover);

impl Mover {
    pub fn new() -> Mover {
        Mover {}
    }
}

impl Moveable for Mover {
    fn random_move<'a, 'b>(&self, context: &Context<'a, 'b>) -> Option<Point> {
        let neighbors = context.world.all(context.loc, 1, |pt| {
            context
                .world
                .cell(pt)
                .iter()
                .all(|id| pt != context.loc && !has_trait!(context.store.get(*id), Animal))
        });
        neighbors
            .iter()
            .choose(context.world.rng().as_mut())
            .copied()
    }

    fn move_towards(&self, world: &World, store: &Store, loc: Point, dst: Point) -> Option<Point> {
        let mut new_loc = None;
        let mut dist = world.distance2(loc, dst);

        for dy in -1..=1 {
            let y = loc.y + dy;
            for dx in -1..=1 {
                let x = loc.x + dx;
                let candidate = Point::new(x, y);
                if !has_animal(world, store, candidate) {
                    let d = world.distance2(candidate, dst);
                    if d < dist {
                        new_loc = Some(candidate);
                        dist = d;
                    }
                }
            }
        }
        new_loc
    }
}

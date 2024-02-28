//! Left behind for a bit after an animal dies.
use super::*;
use colored::*;

const MAX_LIFETIME: i32 = 4;

struct Skeleton {
    lifetime: i32,
}
register_type!(Skeleton);

pub fn add_skeleton(world: &mut World, store: &Store, loc: Point) {
    let mut component = Component::new();
    add_object!(component, Skeleton, Skeleton::new(), [Action, Render]);
    world.add_back(store, loc, component);
}

impl Skeleton {
    pub fn new() -> Skeleton {
        Skeleton {
            lifetime: MAX_LIFETIME,
        }
    }
}

impl Action for Skeleton {
    fn act<'a, 'b>(&mut self, _context: Context<'a, 'b>) -> LifeCycle {
        self.lifetime -= 1;

        if self.lifetime > 0 {
            LifeCycle::Alive
        } else {
            LifeCycle::Dead
        }
    }
}

impl Render for Skeleton {
    fn render(&self) -> ColoredString {
        if self.lifetime == MAX_LIFETIME {
            "*".red()
        } else {
            "*".normal()
        }
    }
}

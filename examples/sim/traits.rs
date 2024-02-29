//! Traits for components added to the world. These are the interfaces that the world
//! uses to interact with components and how components interact with each other.
use super::*;
use colored::ColoredString;

pub struct Context<'a, 'b> {
    pub world: &'a mut World,
    pub store: &'b Store,
    pub loc: Point,
    pub id: ComponentId,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum LifeCycle {
    Alive,
    Dead,
}

// ---------------------------------------------------------------------------------------
/// Every component should include this though it can be a no-op.
pub trait Action {
    fn act<'a, 'b>(&mut self, context: Context<'a, 'b>) -> LifeCycle;
}
register_type!(Action);

// ---------------------------------------------------------------------------------------
/// Every component should include this.
pub trait Render {
    fn render(&self) -> ColoredString;
}
register_type!(Render);

// ---------------------------------------------------------------------------------------
/// Helper interface for something that gets hungry.
pub trait Hunger {
    fn get(&self) -> i32;
    fn set(&mut self, value: i32);
    fn adjust(&mut self, delta: i32);
}
register_type!(Hunger);

// ---------------------------------------------------------------------------------------
/// Helper interface for something that can move around, e.g. rabbits and wolves.
pub trait Moveable {
    fn random_move<'a, 'b>(&self, context: &Context<'a, 'b>) -> Option<Point>;
    fn move_towards(&self, world: &World, store: &Store, loc: Point, dst: Point) -> Option<Point>;
}
register_type!(Moveable);

// ---------------------------------------------------------------------------------------
/// Something rabbits can eat.
pub trait Fodder {
    /// Percent is how much of the fodder rabbits eat at a time.
    fn eat<'a, 'b>(&mut self, context: Context<'a, 'b>, percent: i32);

    /// Amount of fodder. Should only be used for comparisons (shorter or taller).
    fn height(&self) -> u8;
}
register_type!(Fodder);

// ---------------------------------------------------------------------------------------
/// Something predators can eat.
pub trait Prey {}
register_type!(Prey);

// ---------------------------------------------------------------------------------------
/// Something that eats prey.
pub trait Predator {}
register_type!(Predator);

// ---------------------------------------------------------------------------------------
/// Used to identify rabbits and wolves.
pub trait Animal {}
register_type!(Animal);

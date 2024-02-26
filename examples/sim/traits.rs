//! Standard traits for components added to the world.
use super::*;

/// Every component should include this though it can be a no-op.
pub trait Action {
    /// Returns true if the actor is still alive.
    fn act(&mut self, world: &mut World, component: &Component, loc: Point) -> bool; // TODO: use an enum instead of a bool
}
register_type!(Action);
// ---------------------------------------------------------------------------------------

/// Every component should include this.
pub trait Render {
    fn render(&self) -> char;
}
register_type!(Render);
// ---------------------------------------------------------------------------------------

/// Something rabbits can eat.
pub trait Fodder {
    // Percent is how much of the fodder rabbits eat at a time.
    fn eat(&mut self, world: &mut World, id: ComponentId, loc: Point, percent: i32);
}
register_type!(Fodder);
// ---------------------------------------------------------------------------------------

/// Used to identify rabbits and wolves.
pub trait Animal {}
register_type!(Animal);
// ---------------------------------------------------------------------------------------

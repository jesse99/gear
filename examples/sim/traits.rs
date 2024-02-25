//! Standard traits for components added to the world.
use super::*;

/// Every component should include this though it can be a no-op.
pub trait Action {
    /// Returns true if the actor is still alive.
    fn act(&mut self, world: &mut World, loc: Point) -> bool; // TODO: use an enum instead of a bool
}
register_type!(Action);

/// Every component should include this.
pub trait Render {
    fn render(&self) -> char;
}
register_type!(Render);

/// Used to identify things that animals can traverse, e.g. grass.
pub trait Terrain {}
register_type!(Terrain);

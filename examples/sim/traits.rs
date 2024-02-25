use super::*;

pub trait Action {
    /// Returns true if the actor is still alive.
    fn act(&mut self, world: &mut World, loc: Point) -> bool; // TODO: use an enum instead of a bool
}
register_type!(Action);

pub trait Render {
    fn render(&self) -> char;
}
register_type!(Render);

/// Marker trait for things like grass. Could turn this into a real trait by adding
/// something like an is_passable method.
pub trait Terrain {}
register_type!(Terrain);

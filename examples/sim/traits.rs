//! Standard traits for components added to the world.
use super::*;

pub struct Context<'a, 'b> {
    pub world: &'a mut World,
    pub store: &'b Store,
    pub loc: Point,
    pub id: ComponentId,
}

/// Every component should include this though it can be a no-op.
pub trait Action {
    /// Returns true if the component is still alive.
    fn act<'a, 'b>(&mut self, context: Context<'a, 'b>) -> bool; // TODO: use an enum instead of a bool
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
    /// Percent is how much of the fodder rabbits eat at a time.
    fn eat<'a, 'b>(&mut self, context: Context<'a, 'b>, percent: i32);

    /// Amount of fodder. Should only be used for comparisons (shorter or taller).
    fn height(&self) -> u8;
}
register_type!(Fodder);
// ---------------------------------------------------------------------------------------

/// Used to identify rabbits and wolves.
pub trait Animal {}
register_type!(Animal);
// ---------------------------------------------------------------------------------------

use core::sync::atomic::AtomicU32;
use std::fmt::{self, Formatter};
use std::sync::atomic::Ordering;

/// Used to identify components.
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ComponentId(pub u32);

pub static NEXT_COMPONENT_ID: AtomicU32 = AtomicU32::new(1);

pub fn next_component_id() -> ComponentId {
    ComponentId(NEXT_COMPONENT_ID.fetch_add(1, Ordering::Relaxed))
}

impl fmt::Display for ComponentId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

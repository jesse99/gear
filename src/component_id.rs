use arraystring::{typenum::U16, ArrayString};
use core::sync::atomic::AtomicU32;
use std::fmt::{self, Formatter};
use std::sync::atomic::Ordering;

pub type TagStr = ArrayString<U16>;

/// Used to identify components.
#[derive(Copy, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ComponentId {
    #[cfg(debug_assertions)]
    tag: TagStr,

    id: u32,
}

impl ComponentId {
    #[cfg(debug_assertions)]
    pub fn new(tag: &str, value: u32) -> ComponentId {
        ComponentId {
            tag: TagStr::from_str_truncate(tag),
            id: value,
        }
    }

    #[cfg(not(debug_assertions))]
    pub fn new(_tag: &str, value: u32) -> ComponentId {
        Oid { id: value }
    }
}

pub static NEXT_COMPONENT_ID: AtomicU32 = AtomicU32::new(1);

#[cfg(debug_assertions)]
pub fn next_component_id(tag: &str) -> ComponentId {
    ComponentId::new(tag, NEXT_COMPONENT_ID.fetch_add(1, Ordering::Relaxed))
}

#[cfg(not(debug_assertions))]
pub fn next_component_id(_tag: &str) -> ComponentId {
    ComponentId::new(NEXT_COMPONENT_ID.fetch_add(1, Ordering::Relaxed))
}

impl fmt::Debug for ComponentId {
    #[cfg(debug_assertions)]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}#{}", self.tag, self.id)
    }

    #[cfg(not(debug_assertions))]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.id)
    }
}

impl fmt::Display for ComponentId {
    #[cfg(debug_assertions)]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}#{}", self.tag, self.id)
    }

    #[cfg(not(debug_assertions))]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.id)
    }
}

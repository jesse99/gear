use core::sync::atomic::{AtomicU16, Ordering};
use std::sync::LazyLock;

/// Used to identify trait and object types. Note that these are generally not directly
/// used by client code.
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ID(pub u16);

pub static NEXT_ID: AtomicU16 = AtomicU16::new(0);

#[macro_export]
macro_rules! unique_id {
    () => {{
        static LOCAL_ID: std::sync::LazyLock<u16> =
            std::sync::LazyLock::new(|| NEXT_ID.fetch_add(1, Ordering::Relaxed));
        ID(*LOCAL_ID)
    }};
}

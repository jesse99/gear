use core::sync::atomic::AtomicU16;

/// Used to identify trait and object types. Note that these are generally not directly
/// used by client code.
#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TypeId(pub u16);

#[doc(hidden)]
pub static NEXT_TYPE_ID: AtomicU16 = AtomicU16::new(0);

#[doc(hidden)]
#[macro_export]
macro_rules! unique_type_id {
    () => {{
        static LOCAL_ID: std::sync::LazyLock<u16> =
            std::sync::LazyLock::new(|| NEXT_TYPE_ID.fetch_add(1, Ordering::Relaxed));
        TypeId(*LOCAL_ID)
    }};
}

use super::{unique_id, ID};
use fnv::FnvHashMap;
use paste::paste;
use std::any::Any;
use std::marker::Unsize;
use std::mem::transmute;
use std::ptr::{self, DynMetadata, Pointee};

/// The unit of composition for the gear object model.
/// A component consists  of one or more objects. Each object implements one or more
/// traits. Component clients are only allowed to interact with objects via their traits.
/// Note that publicly released traits should be treated as immutable to foster backward
/// compatibility.
pub struct Component {
    objects: FnvHashMap<ID, Box<dyn Any>>, // object id => type erased boxed object
    traits: FnvHashMap<ID, TypeErasedPointer>, // trait id => type erased trait pointer
}

impl Component {
    pub fn new() -> Component {
        Component {
            objects: FnvHashMap::default(),
            traits: FnvHashMap::default(),
        }
    }

    /// Normally the [`add_object`]` macro would be used instead of calling this directly.
    pub fn add_trait<Trait, Object>(&mut self, trait_id: ID, obj_ptr: *mut Object)
    where
        Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
        Object: Unsize<Trait> + 'static,
    {
        let erased = TypeErasedPointer::from_trait::<Object, Trait>(obj_ptr);
        let old = self.traits.insert(trait_id, erased);
        assert!(old.is_none(), "trait was already added to the component");
    }

    /// Normally the [`add_object`]` macro would be used instead of calling this directly.
    pub fn add_object<Object>(&mut self, obj_id: ID, obj_ptr: *mut Object)
    where
        Object: 'static,
    {
        let erased: Box<dyn Any> = unsafe { Box::from_raw(obj_ptr) };

        // Note that the same object type can be added multiple times. Not clear how useful
        // this is but it may be when repeated traits are used.
        self.objects.insert(obj_id, erased);
    }

    /// Normally the [`find_trait`]` macro would be used instead of calling this directly.
    pub fn find<Trait>(&self, trait_id: ID) -> Option<&Trait>
    where
        Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
    {
        if let Some(erased) = self.traits.get(&trait_id) {
            let r = unsafe { erased.to_trait::<Trait>() };
            Some(r)
        } else {
            None
        }
    }

    /// Normally the [`find_trait_mut`]` macro would be used instead of calling this directly.
    pub fn find_mut<Trait>(&mut self, trait_id: ID) -> Option<&mut Trait>
    where
        Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
    {
        if let Some(erased) = self.traits.get(&trait_id) {
            let r = unsafe { erased.to_trait_mut::<Trait>() };
            Some(r)
        } else {
            None
        }
    }
}

/// Use this for all trait and object types used within components.
///
/// # Examples
///
/// ```
/// #![feature(lazy_cell)]
/// use gear::*;
/// use core::sync::atomic::Ordering;
/// use paste::paste;
///
/// trait Fruit {
///     fn eat(&self) -> String;
/// }
/// register_type!(Fruit);
/// ```
#[macro_export]
macro_rules! register_type {
    ($type:ty) => {
        paste! {
            pub fn [<get_ $type:lower _id>]() -> ID {
                unique_id!()
            }
        }
    };
}

/// Use the [`add_object`] macro not this one.
#[macro_export]
macro_rules! add_traits {
    ($component:expr, $obj_type:ty, $obj_ptr:expr, $trait1:ty) => {{
        paste! {
            $component.add_trait::<dyn $trait1, $obj_type>(
                [<get_ $trait1:lower _id>](),
                $obj_ptr);
        }
    }};

    ($component:expr, $obj_type:ty, $obj_ptr:expr, $trait1:ty, $($trait2:ty),+) => {{
        add_traits!($component:expr, $obj_type, $obj_ptr, $trait1)
        add_traits!($component:expr, $obj_type, $obj_ptr, $($trait2:ty),+)
    }};
}

/// Use this to add an object along with its associated traits to a component.
///
/// # Examples
///
/// ```
/// #![feature(lazy_cell)]
/// use gear::*;
/// use core::sync::atomic::Ordering;
/// use paste::paste;
///
/// struct Apple {}
/// register_type!(Apple);
///
/// trait Fruit {
///     fn eat(&self) -> String;
/// }
/// register_type!(Fruit);
///
/// impl Fruit for Apple {
///     fn eat(&self) -> String {
///         "yum!".to_owned()
///     }
/// }
///
/// let apple = Apple {};
/// let mut component = Component::new();
/// add_object!(component, Apple, apple, [Fruit]);
/// ```
#[macro_export]
macro_rules! add_object {
    ($component:expr, $obj_type:ty, $object:expr, [$trait1:ty]) => {{
        paste! {
            let boxed = Box::new($object);
            let obj_ptr = Box::into_raw(boxed);
            add_traits!($component, $obj_type, obj_ptr, $trait1);
            $component.add_object::<$obj_type>(
                [<get_ $obj_type:lower _id>](),
                obj_ptr);
        }
    }};

    ($component:expr, $obj_type:ty, $object:expr, [$trait1:ty, $($trait2:ty),+]) => {{
        paste! {
            let boxed = Box::new($object);
            let obj_ptr = Box::into_raw(boxed);
            add_traits!($component, $obj_type, obj_ptr, $trait1);
            add_traits!($component, $obj_type, obj_ptr, $($trait2),+);
            $component.add_object::<$obj_type>(
                [<get_ $obj_type:lower _id>](),
                obj_ptr);
        }
    }};
}

/// Returns an optional reference to a trait for an object within the component.
///
/// # Examples
///
/// ```
/// #![feature(lazy_cell)]
/// use gear::*;
/// use core::sync::atomic::Ordering;
/// use paste::paste;
///
/// struct Apple {}
/// register_type!(Apple);
///
/// trait Fruit {
///     fn eat(&self) -> String;
/// }
/// register_type!(Fruit);
///
/// impl Fruit for Apple {
///     fn eat(&self) -> String {
///         "yum!".to_owned()
///     }
/// }
///
/// let apple = Apple {};
/// let mut component = Component::new();
/// add_object!(component, Apple, apple, [Fruit]);
///
/// let fruit = find_trait!(component, Fruit);
/// assert_eq!(fruit.unwrap().eat(), "yum!");
/// ```
#[macro_export]
macro_rules! find_trait {
    ($component:expr, $trait:ty) => {{
        paste! {
            $component.find::<dyn $trait>([<get_ $trait:lower _id>]())
        }
    }};
}

#[macro_export]
macro_rules! find_trait_mut {
    ($component:expr, $trait:ty) => {{
        paste! {
            $component.find_mut::<dyn $trait>([<get_ $trait:lower _id>]())
        }
    }};
}

// Decomposed trait pointer.
struct TypeErasedPointer {
    pointer: *mut (),
    metadata: Box<*const ()>,
}

impl TypeErasedPointer {
    fn from_trait<Object, Trait>(pointer: *mut Object) -> Self
    where
        Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
        Object: Unsize<Trait>,
    {
        let (pointer, metadata) = (pointer as *mut Trait).to_raw_parts();
        let metadata = unsafe { transmute(Box::new(metadata)) };

        TypeErasedPointer { pointer, metadata }
    }

    unsafe fn to_trait<'a, Trait>(&self) -> &'a Trait
    where
        Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
    {
        let src = self.metadata.as_ref();
        let metadata = unsafe { *transmute::<_, *const <Trait as Pointee>::Metadata>(src) };
        let typed_ptr = ptr::from_raw_parts_mut::<Trait>(self.pointer, metadata);
        &*typed_ptr
    }

    unsafe fn to_trait_mut<'a, Trait>(&self) -> &'a mut Trait
    where
        Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
    {
        let src = self.metadata.as_ref();
        let metadata = unsafe { *transmute::<_, *const <Trait as Pointee>::Metadata>(src) };
        let typed_ptr = ptr::from_raw_parts_mut::<Trait>(self.pointer, metadata);
        &mut *typed_ptr
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NEXT_ID;
    use std::sync::atomic::{AtomicU8, Ordering};

    trait Fruit {
        fn eat(&self) -> String;
    }
    register_type!(Fruit);

    trait Ball {
        fn throw(&self) -> String;
    }
    register_type!(Ball);

    struct Apple {}
    register_type!(Apple);

    impl Fruit for Apple {
        fn eat(&self) -> String {
            "yum!".to_owned()
        }
    }

    impl Ball for Apple {
        fn throw(&self) -> String {
            "splat".to_owned()
        }
    }

    trait Ripe {
        fn ripeness(&self) -> i32;
        fn ripen(&mut self);
    }
    register_type!(Ripe);
    struct Banana {
        ripeness: i32,
    }
    register_type!(Banana);

    impl Ripe for Banana {
        fn ripeness(&self) -> i32 {
            self.ripeness
        }

        fn ripen(&mut self) {
            self.ripeness += 1;
        }
    }

    impl Fruit for Banana {
        fn eat(&self) -> String {
            "mushy".to_owned()
        }
    }

    static DROP_COUNT: AtomicU8 = AtomicU8::new(0);

    struct Football {}
    register_type!(Football);

    impl Ball for Football {
        fn throw(&self) -> String {
            "touchdown".to_owned()
        }
    }

    impl Drop for Football {
        fn drop(&mut self) {
            DROP_COUNT.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[test]
    fn two_traits() {
        let apple = Apple {};
        let mut component = Component::new();
        add_object!(component, Apple, apple, [Fruit, Ball]);

        let fruit = find_trait!(component, Fruit);
        assert!(fruit.is_some());
        assert_eq!(fruit.unwrap().eat(), "yum!");

        let ball = find_trait!(component, Ball);
        assert!(ball.is_some());
        assert_eq!(ball.unwrap().throw(), "splat");
    }

    #[test]
    fn missing_trait() {
        let banana = Banana { ripeness: 0 };
        let mut component = Component::new();
        add_object!(component, Banana, banana, [Fruit]);

        let fruit = find_trait!(component, Fruit);
        assert!(fruit.is_some());
        assert_eq!(fruit.unwrap().eat(), "mushy");

        let ball = find_trait!(component, Ball);
        assert!(ball.is_none());
    }

    #[test]
    fn dropped_object() {
        assert_eq!(DROP_COUNT.load(Ordering::Relaxed), 0);
        {
            let football = Football {};
            let mut component = Component::new();
            add_object!(component, Football, football, [Ball]);

            let ball = find_trait!(component, Ball);
            assert!(ball.is_some());
            assert_eq!(ball.unwrap().throw(), "touchdown");
        }
        assert_eq!(DROP_COUNT.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn mutable_find() {
        let banana = Banana { ripeness: 0 };
        let mut component = Component::new();
        add_object!(component, Banana, banana, [Fruit, Ripe]);

        let ripe = find_trait!(component, Ripe).unwrap();
        assert_eq!(ripe.ripeness(), 0);

        let mripe = find_trait_mut!(component, Ripe).unwrap();
        mripe.ripen();
        mripe.ripen();

        let ripe = find_trait!(component, Ripe).unwrap(); // grab a new ref to appease the borrow checker
        assert_eq!(ripe.ripeness(), 2);
    }

    // TODO: add support for repeated traits?
    // TODO: support removing objects?
    // TODO: add an example project
    // TODO: fix clippy warnings
    // TODO: review old gear project
    // TODO: would be nice to retain stringified trait and object names
    //       could then have a Debug impl that printed that
    //       does make Components heavier weight, maybe only do this for debug builds?
    //          or can we generate functions to get a string from an id? not sure how we'd call those
    //          maybe ID could include a string in debug
    //       can we use Formatter to optionally delegate to objects?
    // TODO: review docs (especially the item linking)
    // TODO: work on readme
}

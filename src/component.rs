use super::id::ID;
use paste::paste;
use std::any::Any;
use std::collections::HashMap;
use std::marker::Unsize;
use std::mem::transmute;
use std::ptr::{self, DynMetadata, Pointee};

/// The unit of composition for the gear object model.
/// A component consists  of one or more objects. Each object implements one or more
/// traits. Component clients are only allowed to interact with objects via their traits.
/// Note that publicly released traits should be treated as immutable to foster backward
/// compatibility.
pub struct Component {
    objects: HashMap<ID, Box<dyn Any>>, // object id => type erased boxed object
    traits: HashMap<ID, TypeErasedPointer>, // trait id => type erased trait pointer
}

impl Component {
    pub fn new() -> Component {
        Component {
            objects: HashMap::new(),
            traits: HashMap::new(),
        }
    }

    /// Normally the add_object1 macro would be used instead of calling this directly.
    pub fn add_impl1<Trait, Object>(&mut self, trait_id: ID, obj_id: ID, obj: Box<Object>)
    where
        Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
        Object: Unsize<Trait> + 'static,
    {
        let typed_ptr = Box::into_raw(obj);
        let erased = TypeErasedPointer::from_trait::<Object, Trait>(typed_ptr);
        self.traits.insert(trait_id, erased);

        let erased: Box<dyn Any> = unsafe { Box::from_raw(typed_ptr) };
        let old = self.objects.insert(obj_id, erased);
        assert!(old.is_none(), "trait was already added to the component");
    }

    /// Normally the add_object2 macro would be used instead of calling this directly.
    pub fn add_impl2<Trait1, Trait2, Object>(
        &mut self,
        trait1_id: ID,
        trait2_id: ID,
        obj_id: ID,
        obj: Box<Object>,
    ) where
        Trait1: ?Sized + Pointee<Metadata = DynMetadata<Trait1>> + 'static,
        Trait2: ?Sized + Pointee<Metadata = DynMetadata<Trait2>> + 'static,
        Object: Unsize<Trait1> + 'static,
        Object: Unsize<Trait2> + 'static,
    {
        let typed_ptr = Box::into_raw(obj);
        let erased = TypeErasedPointer::from_trait::<Object, Trait1>(typed_ptr);
        self.traits.insert(trait1_id, erased);

        let erased = TypeErasedPointer::from_trait::<Object, Trait2>(typed_ptr);
        self.traits.insert(trait2_id, erased);

        let erased: Box<dyn Any> = unsafe { Box::from_raw(typed_ptr) };
        let old = self.objects.insert(obj_id, erased);
        assert!(old.is_none(), "trait was already added to the component");
    }

    /// Normally the find_trait macro would be used instead of calling this directly.
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
}

/// Use this for all trait and object types used within components.
///
/// # Examples
///
/// ```
/// trait Fruit {
///     fn eat(&self) -> String;
/// }
/// register_type!(Fruit);
/// ```
#[macro_export]
macro_rules! register_type {
    ($type:ty) => {
        paste! {
            pub fn [<get_ $type:lower _id>]() -> crate::id::ID {
                crate::id::unique_id!()
            }
        }
    };
}

/// Use this to add an object along with its associated traits to a component.
///
/// # Examples
///
/// ```
/// let apple = Apple {};
/// let mut component = Component::new();
/// add_object1!(component, Fruit, Apple, apple);
/// ```
#[macro_export]
macro_rules! add_object1 {
    ($component:expr, $trait1:ty, $obj_type:ty, $object:expr) => {{
        paste! {
                $component.add_impl1::<dyn $trait1, $obj_type>(
                    [<get_ $trait1:lower _id>](),
                    [<get_ $obj_type:lower _id>](),
                Box::new($object),
            );
        }
    }};
}

#[macro_export]
macro_rules! add_object2 {
    ($component:expr, $trait1:ty, $trait2:ty, $obj_type:ty, $object:expr) => {{
        paste! {
                $component.add_impl2::<dyn $trait1, dyn $trait2, $obj_type>(
                    [<get_ $trait1:lower _id>](),
                    [<get_ $trait2:lower _id>](),
                    [<get_ $obj_type:lower _id>](),
                Box::new($object),
            );
        }
    }};
}

#[macro_export]
macro_rules! find_trait {
    ($component:expr, $trait:ty) => {{
        paste! {
            $component.find::<dyn $trait>([<get_ $trait:lower _id>]())
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
}

#[cfg(test)]
mod tests {
    use super::*;
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

    struct Banana {}
    register_type!(Banana);

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
        add_object2!(component, Fruit, Ball, Apple, apple);

        let fruit = find_trait!(component, Fruit);
        assert!(fruit.is_some());
        assert_eq!(fruit.unwrap().eat(), "yum!");

        let ball = find_trait!(component, Ball);
        assert!(ball.is_some());
        assert_eq!(ball.unwrap().throw(), "splat");
    }

    #[test]
    fn missing_trait() {
        let banana = Banana {};
        let mut component = Component::new();
        add_object1!(component, Fruit, Banana, banana);

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
            add_object1!(component, Ball, Football, football);

            let ball = find_trait!(component, Ball);
            assert!(ball.is_some());
            assert_eq!(ball.unwrap().throw(), "touchdown");
        }
        assert_eq!(DROP_COUNT.load(Ordering::Relaxed), 1);
    }

    // TODO: use a faster hash
    // TODO: would be nice to retain stringified trait and object names
    //       could then have a Debug impl that printed that
    //       does make Components heavier weight, maybe only do this for debug builds?
    //          or can we generate functions to get a string from an id? not sure how we'd call those
    //       can we use Formatter to optionally delegate to objects?
    // TODO: need a mutable find
    // TODO: add support for repeated traits?
    // TODO: support removing objects?
    // TODO: add support for more than two traits per object
    //       can probably use a build script (build.rs) to generate these
    //       https://doc.rust-lang.org/cargo/reference/build-script-examples.html
    // TODO: work on readme
}

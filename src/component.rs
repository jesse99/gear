#[allow(unused_imports)]
use super::*;
use core::fmt::{self, Debug};
use fnv::FnvHashMap;
#[allow(unused_imports)]
use paste::paste;
use std::any::Any;
use std::hash::{Hash, Hasher};
use std::marker::Unsize;
use std::ptr::{DynMetadata, Pointee};
use type_erased_ptr::*;

/// The unit of composition for the gear object model.
/// A component consists  of one or more objects. Each object implements one or more
/// traits. Component clients are only allowed to interact with objects via their traits.
/// Note that publicly released traits should be treated as immutable to foster backward
/// compatibility.
pub struct Component {
    pub id: ComponentId,
    objects: FnvHashMap<TypeId, Box<dyn Any + Send + Sync>>, // object id => type erased boxed object
    traits: FnvHashMap<TypeId, TypeErasedPointer>, // trait id => type erased trait pointer
    repeated: FnvHashMap<TypeId, Vec<TypeErasedPointer>>, // trait id => [type erased trait pointer]
    refs: FnvHashMap<TypeId, ObjectRefs>, // object id => outstanding trait references on the object
    empty: Vec<TypeErasedPointer>,
}

impl Component {
    /// tag is used by the Debug trait on Component (and ComponentId).
    pub fn new(tag: &str) -> Component {
        Component {
            id: next_component_id(tag),
            objects: FnvHashMap::default(),
            traits: FnvHashMap::default(),
            repeated: FnvHashMap::default(),
            empty: Vec::new(),
            refs: FnvHashMap::default(),
        }
    }

    // Normally the [`add_traits`]` macro would be used instead of calling this directly.
    #[doc(hidden)]
    pub fn add_trait<Trait, Object>(
        &mut self,
        object_id: TypeId,
        trait_id: TypeId,
        obj_ptr: *mut Object,
    ) where
        Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
        Object: Unsize<Trait> + 'static,
    {
        let erased = TypeErasedPointer::from_trait::<Object, Trait>(object_id, obj_ptr);
        let old = self.traits.insert(trait_id, erased);
        assert!(old.is_none(), "trait was already added to the component");
    }

    // Normally the [`add_repeated_traits`]` macro would be used instead of calling this directly.
    #[doc(hidden)]
    pub fn add_repeated_trait<Trait, Object>(
        &mut self,
        object_id: TypeId,
        trait_id: TypeId,
        obj_ptr: *mut Object,
    ) where
        Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
        Object: Unsize<Trait> + 'static,
    {
        let erased = TypeErasedPointer::from_trait::<Object, Trait>(object_id, obj_ptr);
        let pointers = self.repeated.entry(trait_id).or_insert(vec![]);
        pointers.push(erased);
    }

    // Normally the [`add_object`]` macro would be used instead of calling this directly.
    #[doc(hidden)]
    pub fn add_object<Object>(&mut self, obj_id: TypeId, obj_ptr: *mut Object)
    where
        Object: Send + Sync + 'static,
    {
        let erased: Box<dyn Any + Send + Sync> = unsafe { Box::from_raw(obj_ptr) };
        let old = self.objects.insert(obj_id, erased);
        assert!(
            old.is_none(),
            "object type was already added to the component"
        );

        self.refs.entry(obj_id).or_insert(ObjectRefs::new());
    }

    // TODO: May want to support remove_object. Would be kinda slow: probably need to
    // change traits and repeated so that the value includes the object's type id. One
    // nice thing is, that if we did do that, Debug and Display could print the traits
    // associated with the corresponding object.

    // Normally the [`has_trait`]` macro would be used instead of calling this directly.
    #[doc(hidden)]
    pub fn has<Trait>(&self, trait_id: TypeId) -> bool
    where
        Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
    {
        self.traits.get(&trait_id).is_some()
    }

    // Normally the [`find_trait`]` macro would be used instead of calling this directly.
    #[doc(hidden)]
    pub fn find<Trait>(&self, trait_id: TypeId) -> Option<RefTrait<Trait>>
    where
        Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
    {
        if let Some(erased) = self.traits.get(&trait_id) {
            let refs = self.refs.get(&erased.object_id).unwrap();
            let r = unsafe { erased.to_trait::<Trait>(refs) };
            Some(r)
        } else {
            None
        }
    }

    // Normally the [`find_trait_mut`]` macro would be used instead of calling this directly.
    #[doc(hidden)]
    pub fn find_mut<Trait>(&self, trait_id: TypeId) -> Option<RefMutTrait<Trait>>
    where
        Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
    {
        if let Some(erased) = self.traits.get(&trait_id) {
            let refs = self.refs.get(&erased.object_id).unwrap();
            let r = unsafe { erased.to_trait_mut::<Trait>(refs) };
            Some(r)
        } else {
            None
        }
    }

    // Normally the [`find_repeated_trait`]` macro would be used instead of calling this directly.
    #[doc(hidden)]
    pub fn find_repeated<Trait>(&self, trait_id: TypeId) -> impl Iterator<Item = RefTrait<Trait>>
    where
        Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
    {
        self.repeated
            .get(&trait_id)
            .unwrap_or(&self.empty)
            .iter()
            .map(|e| unsafe {
                let refs = self.refs.get(&e.object_id).unwrap();
                e.to_trait::<Trait>(refs)
            })
    }

    // Normally the [`find_repeated_trait_mut`]` macro would be used instead of calling this directly.
    #[doc(hidden)]
    pub fn find_repeated_mut<Trait>(
        &self,
        trait_id: TypeId,
    ) -> impl Iterator<Item = RefMutTrait<Trait>>
    where
        Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
    {
        self.repeated
            .get(&trait_id)
            .unwrap_or(&self.empty)
            .iter()
            .map(|e| unsafe {
                let refs = self.refs.get(&e.object_id).unwrap();
                e.to_trait_mut::<Trait>(refs)
            })
    }
}

/// Use this for all trait and object types used within components.
///
/// # Examples
///
/// ```
/// #![feature(lazy_cell)]
/// use gear_objects::*;
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
            pub fn [<get_ $type:lower _id>]() -> TypeId {
                unique_type_id!()
            }
        }
    };
}

// Use the [`add_object`] macro not this one.
#[doc(hidden)]
#[macro_export]
macro_rules! add_traits {
    ($component:expr, $obj_type:ty, $obj_ptr:expr, $trait1:ty) => {{
        paste! {
            $component.add_trait::<dyn $trait1, $obj_type>(
                [<get_ $obj_type:lower _id>](),
                [<get_ $trait1:lower _id>](),
                $obj_ptr);
        }
    }};

    ($component:expr, $obj_type:ty, $obj_ptr:expr, $trait1:ty, $($trait2:ty),+) => {{
        add_traits!($component, $obj_type, $obj_ptr, $trait1);
        add_traits!($component, $obj_type, $obj_ptr, $($trait2),+)
    }};
}

// Use the [`add_object`] macro not this one.
#[doc(hidden)]
#[macro_export]
macro_rules! add_repeated_traits {
    ($component:expr, $obj_type:ty, $obj_ptr:expr, $trait1:ty) => {{
        paste! {
            $component.add_repeated_trait::<dyn $trait1, $obj_type>(
                [<get_ $obj_type:lower _id>](),
                [<get_ $trait1:lower _id>](),
                $obj_ptr);
        }
    }};

    ($component:expr, $obj_type:ty, $obj_ptr:expr, $trait1:ty, $($trait2:ty),+) => {{
        add_repeated_traits!($component, $obj_type, $obj_ptr, $trait1);
        add_repeated_traits!($component, $obj_type, $obj_ptr, $($trait2),+)
    }};
}

/// Use this to add an object along with its associated traits to a component. Note that
/// repeated traits are listed in a second optional list.
///
/// # Examples
///
/// ```
/// #![feature(lazy_cell)]
/// use gear_objects::*;
/// use core::fmt;
/// use paste::paste;
/// use std::fmt::{Display};
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
/// impl fmt::Display for Apple {
///     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
///         write!(f, "Apple")
///     }
/// }
/// register_type!(Display);
///
/// let apple = Apple {};
/// let mut component = Component::new("apple");
/// add_object!(component, Apple, apple, [Fruit], [Display]);
/// ```
#[macro_export]
macro_rules! add_object {
    ($component:expr, $obj_type:ty, $object:expr, [$trait1:ty]) => {{   // 1 0
        paste! {
            let boxed = Box::new($object);
            let obj_ptr = Box::into_raw(boxed);
            add_traits!($component, $obj_type, obj_ptr, $trait1);
            $component.add_object::<$obj_type>(
                [<get_ $obj_type:lower _id>](),
                obj_ptr);
        }
    }};

    ($component:expr, $obj_type:ty, $object:expr, [$trait1:ty], [$trait2:ty]) => {{ // 1 1
        paste! {
            let boxed = Box::new($object);
            let obj_ptr = Box::into_raw(boxed);
            add_traits!($component, $obj_type, obj_ptr, $trait1);
            add_repeated_traits!($component, $obj_type, obj_ptr, $trait2);
            $component.add_object::<$obj_type>(
                [<get_ $obj_type:lower _id>](),
                obj_ptr);
        }
    }};

    ($component:expr, $obj_type:ty, $object:expr, [$trait1:ty], [$trait2:ty, $($trait3:ty),+]) => {{   // 1 +
        paste! {
            let boxed = Box::new($object);
            let obj_ptr = Box::into_raw(boxed);
            add_traits!($component, $obj_type, obj_ptr, $trait1);
            add_repeated_traits!($component, $obj_type, obj_ptr, $trait2);
            add_repeated_traits!($component, $obj_type, obj_ptr, $($trait3),+);
            $component.add_object::<$obj_type>(
                [<get_ $obj_type:lower _id>](),
                obj_ptr);
        }
    }};

    ($component:expr, $obj_type:ty, $object:expr, [$trait1:ty, $($trait2:ty),+]) => {{  // + 0
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

    ($component:expr, $obj_type:ty, $object:expr, [$trait1:ty, $($trait2:ty),+], [$trait3:ty]) => {{   // + 1
        paste! {
            let boxed = Box::new($object);
            let obj_ptr = Box::into_raw(boxed);
            add_traits!($component, $obj_type, obj_ptr, $trait1);
            add_traits!($component, $obj_type, obj_ptr, $($trait2),+);
            add_repeated_traits!($component, $obj_type, obj_ptr, $trait3);
            $component.add_object::<$obj_type>(
                [<get_ $obj_type:lower _id>](),
                obj_ptr);
        }
    }};

    ($component:expr, $obj_type:ty, $object:expr, [$trait1:ty, $($trait2:ty),+], [$trait3:ty, $($trait4:ty),+]) => {{   // + +
        paste! {
            let boxed = Box::new($object);
            let obj_ptr = Box::into_raw(boxed);
            add_traits!($component, $obj_type, obj_ptr, $trait1);
            add_traits!($component, $obj_type, obj_ptr, $($trait2),+);
            add_repeated_traits!($component, $obj_type, obj_ptr, $trait3);
            add_repeated_traits!($component, $obj_type, obj_ptr, $($trait4),+);
            $component.add_object::<$obj_type>(
                [<get_ $obj_type:lower _id>](),
                obj_ptr);
        }
    }};
}

#[macro_export]
macro_rules! has_trait {
    ($component:expr, $trait:ty) => {{
        paste! {
            $component.has::<dyn $trait>([<get_ $trait:lower _id>]())
        }
    }};
}

/// Returns an optional reference to a trait for an object within the component.
///
/// # Examples
///
/// ```
/// #![feature(lazy_cell)]
/// use gear_objects::*;
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
/// let mut component = Component::new("apple");
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

/// The borrowing rules for components are the standard rust rules: mutable references are
/// exclusive references. But they apply to individual objects within a component so it's
/// possible to simultaneously get two mutable references to two different objects within
/// a component but not two mutable references to the same object (this is checked at
/// runtime).
#[macro_export]
macro_rules! find_trait_mut {
    ($component:expr, $trait:ty) => {{
        paste! {
            $component.find_mut::<dyn $trait>([<get_ $trait:lower _id>]())
        }
    }};
}

/// Returns an iterator over a trait that may be implemented by multiple objects within
/// the component.
#[macro_export]
macro_rules! find_repeated_trait {
    ($component:expr, $trait:ty) => {{
        paste! {
            $component.find_repeated::<dyn $trait>([<get_ $trait:lower _id>]())
        }
    }};
}

/// Returns an iterator over a trait that may be implemented by multiple objects within
/// the component.
#[macro_export]
macro_rules! find_repeated_trait_mut {
    ($component:expr, $trait:ty) => {{
        paste! {
            $component.find_repeated_mut::<dyn $trait>([<get_ $trait:lower _id>]())
        }
    }};
}

impl PartialEq for Component {
    fn eq(&self, other: &Component) -> bool {
        self.id == other.id
    }
}

impl Eq for Component {}

impl Ord for Component {
    fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
        self.id.cmp(&rhs.id)
    }
}

impl PartialOrd for Component {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for Component {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Debug for Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:?}", self.id)?;
        for d in find_repeated_trait!(self, Debug) {
            d.fmt(f)?;
        }
        fmt::Result::Ok(())
    }
}
register_type!(Debug);

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Display;
    use std::sync::atomic::AtomicU8;
    use std::sync::atomic::Ordering;

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

    impl fmt::Display for Apple {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Apple")
        }
    }
    register_type!(Display);

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

    impl fmt::Display for Banana {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Banana")
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
        let mut component = Component::new("apple");
        add_object!(component, Apple, apple, [Fruit, Ball]);

        let fruit = find_trait!(component, Fruit);
        assert!(fruit.is_some());
        assert_eq!(fruit.unwrap().eat(), "yum!");

        let ball = find_trait!(component, Ball);
        assert!(ball.is_some());
        assert_eq!(ball.unwrap().throw(), "splat");
    }

    #[test]
    fn has() {
        let apple = Apple {};
        let mut component = Component::new("apple");
        add_object!(component, Apple, apple, [Fruit, Ball]);

        assert!(has_trait!(component, Fruit));
        assert!(!has_trait!(component, Ripe));
    }

    #[test]
    fn missing_trait() {
        let banana = Banana { ripeness: 0 };
        let mut component = Component::new("banana");
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
            let mut component = Component::new("football");
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
        let mut component = Component::new("banana");
        add_object!(component, Banana, banana, [Fruit, Ripe]);

        {
            let ripe = find_trait!(component, Ripe).unwrap();
            assert_eq!(ripe.ripeness(), 0);
        }

        {
            let mut ripe = find_trait_mut!(component, Ripe).unwrap();
            ripe.ripen();
            ripe.ripen();

            // this will panic
            // let mut fruit = find_trait_mut!(component, Fruit).unwrap();
            // fruit.eat();
        }

        let ripe = find_trait!(component, Ripe).unwrap(); // grab a new ref to appease the borrow checker
        assert_eq!(ripe.ripeness(), 2);
    }

    #[test]
    fn repeated() {
        let banana = Banana { ripeness: 0 };
        let apple = Apple {};
        let mut component = Component::new("banana");
        add_object!(component, Banana, banana, [Fruit, Ripe], [Display]);
        add_object!(component, Apple, apple, [Ball], [Display]);

        // display method
        // fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error>;

        let displays: Vec<String> = find_repeated_trait!(component, Display)
            .map(|t| format!("{}", &(*t)))
            .collect();
        assert_eq!(displays.len(), 2);
        assert!(
            (displays[0] == "Apple" && displays[1] == "Banana")
                || (displays[1] == "Apple" && displays[0] == "Banana")
        );
    }
}

#[cfg(test)]
mod thread_tests {
    use super::*;
    use std::{
        sync::{Arc, RwLock},
        thread,
    };

    trait Name {
        fn get(&self) -> &str;
        fn get_mut(&mut self) -> &mut String;
    }
    register_type!(Name);

    struct Thing {
        name: String,
    }
    register_type!(Thing);

    impl Name for Thing {
        fn get(&self) -> &str {
            &self.name
        }

        fn get_mut(&mut self) -> &mut String {
            &mut self.name
        }
    }

    #[test]
    fn threading() {
        let thing = Thing {
            name: "hello world".to_owned(),
        };
        let mut component = Component::new("thing");
        add_object!(component, Thing, thing, [Name]);

        let gstate = Arc::new(RwLock::new(component));

        let state = gstate.clone();
        let thread1 = thread::spawn(move || {
            for _ in 0..100 {
                {
                    let component = state.write().unwrap();
                    let mut name = find_trait_mut!(component, Name).unwrap();
                    let name = name.get_mut();
                    if name.len() < 30 {
                        name.insert(6, '_');
                    }
                }
                thread::yield_now();
            }
        });

        let state = gstate.clone();
        let thread2 = thread::spawn(move || {
            for _ in 0..100 {
                {
                    let component = state.write().unwrap();
                    let mut name = find_trait_mut!(component, Name).unwrap();
                    let name = name.get_mut();
                    if name.len() > 11 {
                        name.remove(6);
                    }
                }
                thread::yield_now();
            }
        });

        thread1.join().unwrap();
        thread2.join().unwrap();

        let component = &gstate.read().unwrap();
        let name = find_trait!(component, Name).unwrap();
        let name = name.get();
        assert!(name.starts_with("hello"));
        assert!(name.ends_with("world"));
    }
}

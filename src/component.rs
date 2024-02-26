#[allow(unused_imports)]
use super::*;
use fnv::FnvHashMap;
#[allow(unused_imports)]
use paste::paste;
use std::any::Any;
use std::hash::{Hash, Hasher};
use std::marker::Unsize;
use std::mem::transmute;
use std::ptr::{self, DynMetadata, Pointee};

/// The unit of composition for the gear object model.
/// A component consists  of one or more objects. Each object implements one or more
/// traits. Component clients are only allowed to interact with objects via their traits.
/// Note that publicly released traits should be treated as immutable to foster backward
/// compatibility.
pub struct Component {
    pub id: ComponentId,
    objects: FnvHashMap<TypeId, Box<dyn Any>>, // object id => type erased boxed object
    traits: FnvHashMap<TypeId, TypeErasedPointer>, // trait id => type erased trait pointer
    repeated: FnvHashMap<TypeId, Vec<TypeErasedPointer>>, // trait id => [type erased trait pointer]
    empty: Vec<TypeErasedPointer>,
}

impl Component {
    pub fn new() -> Component {
        Component {
            id: next_component_id(),
            objects: FnvHashMap::default(),
            traits: FnvHashMap::default(),
            repeated: FnvHashMap::default(),
            empty: Vec::new(),
        }
    }

    /// Normally the [`add_object`]` macro would be used instead of calling this directly.
    pub fn add_trait<Trait, Object>(&mut self, trait_id: TypeId, obj_ptr: *mut Object)
    where
        Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
        Object: Unsize<Trait> + 'static,
    {
        let erased = TypeErasedPointer::from_trait::<Object, Trait>(obj_ptr);
        let old = self.traits.insert(trait_id, erased);
        assert!(old.is_none(), "trait was already added to the component");
    }

    /// Normally the [`add_object`]` macro would be used instead of calling this directly.
    pub fn add_repeated_trait<Trait, Object>(&mut self, trait_id: TypeId, obj_ptr: *mut Object)
    where
        Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
        Object: Unsize<Trait> + 'static,
    {
        let erased = TypeErasedPointer::from_trait::<Object, Trait>(obj_ptr);
        let pointers = self.repeated.entry(trait_id).or_insert(vec![]);
        pointers.push(erased);
    }

    /// Normally the [`add_object`]` macro would be used instead of calling this directly.
    pub fn add_object<Object>(&mut self, obj_id: TypeId, obj_ptr: *mut Object)
    where
        Object: 'static,
    {
        let erased: Box<dyn Any> = unsafe { Box::from_raw(obj_ptr) };

        // Note that the same object type can be added multiple times. Not clear how useful
        // this is but it may be when repeated traits are used.
        self.objects.insert(obj_id, erased);
    }

    /// Normally the [`has_trait`]` macro would be used instead of calling this directly.
    pub fn has<Trait>(&self, trait_id: TypeId) -> bool
    where
        Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
    {
        self.traits.get(&trait_id).is_some()
    }

    /// Normally the [`find_trait`]` macro would be used instead of calling this directly.
    pub fn find<Trait>(&self, trait_id: TypeId) -> Option<&Trait>
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
    pub fn find_mut<Trait>(&mut self, trait_id: TypeId) -> Option<&mut Trait>
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

    /// Normally the [`find_repeated_trait`]` macro would be used instead of calling this directly.
    pub fn find_repeated<Trait>(&self, trait_id: TypeId) -> impl Iterator<Item = &Trait>
    where
        Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
    {
        self.repeated
            .get(&trait_id)
            .unwrap_or(&self.empty)
            .iter()
            .map(|e| unsafe { e.to_trait::<Trait>() })
    }

    /// Normally the [`find_repeated_trait_mut`]` macro would be used instead of calling this directly.
    pub fn find_repeated_mut<Trait>(&mut self, trait_id: TypeId) -> impl Iterator<Item = &mut Trait>
    where
        Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
    {
        self.repeated
            .get(&trait_id)
            .unwrap_or(&self.empty)
            .iter()
            .map(|e| unsafe { e.to_trait_mut::<Trait>() })
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
            pub fn [<get_ $type:lower _id>]() -> TypeId {
                unique_type_id!()
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
        add_traits!($component, $obj_type, $obj_ptr, $trait1);
        add_traits!($component, $obj_type, $obj_ptr, $($trait2),+)
    }};
}

/// Use the [`add_object`] macro not this one.
#[macro_export]
macro_rules! add_repeated_traits {
    ($component:expr, $obj_type:ty, $obj_ptr:expr, $trait1:ty) => {{
        paste! {
            $component.add_repeated_trait::<dyn $trait1, $obj_type>(
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
/// use gear::*;
/// use core::sync::atomic::Ordering;
/// use paste::paste;
/// use std::fmt::{self, Display};
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
/// let mut component = Component::new();
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
    use std::fmt::{self, Display};
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
    fn has() {
        let apple = Apple {};
        let mut component = Component::new();
        add_object!(component, Apple, apple, [Fruit, Ball]);

        assert!(has_trait!(component, Fruit));
        assert!(!has_trait!(component, Ripe));
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

    #[test]
    fn repeated() {
        let banana = Banana { ripeness: 0 };
        let apple = Apple {};
        let mut component = Component::new();
        add_object!(component, Banana, banana, [Fruit, Ripe], [Display]);
        add_object!(component, Apple, apple, [Ball], [Display]);

        let displays: Vec<String> = find_repeated_trait!(component, Display)
            .map(|t| format!("{t}"))
            .collect();
        assert_eq!(displays.len(), 2);
        assert!(
            (displays[0] == "Apple" && displays[1] == "Banana")
                || (displays[1] == "Apple" && displays[0] == "Banana")
        );
    }

    // TODO: support removing objects?
    // TODO: add an example project, maybe predator/prey sim? grass grows, rabbits eat grass, wolves eat rabbits
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

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
    objects: HashMap<String, Box<dyn Any>>, // object id => type erased boxed object
    traits: HashMap<String, TypeErasedPointer>, // trait id => type erased trait pointer
}

impl Component {
    pub fn new() -> Component {
        Component {
            objects: HashMap::new(),
            traits: HashMap::new(),
        }
    }

    pub fn add_impl1<Trait, Object>(&mut self, trait_id: String, obj_id: String, obj: Box<Object>)
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

    pub fn add_impl2<Trait1, Trait2, Object>(
        &mut self,
        trait1_id: String,
        trait2_id: String,
        obj_id: String,
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

    // TODO: also need a mutable version
    pub fn find<Trait>(&self, trait_id: &str) -> Option<&Trait>
    where
        Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
    {
        if let Some(erased) = self.traits.get(trait_id) {
            let r = unsafe { erased.to_trait::<Trait>() };
            Some(r)
        } else {
            None
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU8, Ordering};

    trait Fruit {
        fn eat(&self) -> String;
    }

    trait Ball {
        fn throw(&self) -> String;
    }

    struct Apple {}

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

    impl Fruit for Banana {
        fn eat(&self) -> String {
            "mushy".to_owned()
        }
    }

    static DROP_COUNT: AtomicU8 = AtomicU8::new(0);
    struct Football {}

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
        component.add_impl2::<dyn Fruit, dyn Ball, Apple>(
            "Fruit".to_owned(),
            "Ball".to_owned(),
            "Apple".to_owned(),
            Box::new(apple),
        );

        let fruit = component.find::<dyn Fruit>("Fruit");
        assert!(fruit.is_some());
        assert_eq!(fruit.unwrap().eat(), "yum!");

        let ball = component.find::<dyn Ball>("Ball");
        assert!(ball.is_some());
        assert_eq!(ball.unwrap().throw(), "splat");
    }

    #[test]
    fn missing_trait() {
        let banana = Banana {};
        let mut component = Component::new();
        component.add_impl1::<dyn Fruit, Banana>(
            "Fruit".to_owned(),
            "Apple".to_owned(),
            Box::new(banana),
        );

        let fruit = component.find::<dyn Fruit>("Fruit");
        assert!(fruit.is_some());
        assert_eq!(fruit.unwrap().eat(), "mushy");

        let ball = component.find::<dyn Ball>("Ball");
        assert!(ball.is_none());
    }

    #[test]
    fn dropped_object() {
        assert_eq!(DROP_COUNT.load(Ordering::Relaxed), 0);
        {
            let football = Football {};
            let mut component = Component::new();
            component.add_impl1::<dyn Ball, Football>(
                "Ball".to_owned(),
                "Football".to_owned(),
                Box::new(football),
            );

            let ball = component.find::<dyn Ball>("Ball");
            assert!(ball.is_some());
            assert_eq!(ball.unwrap().throw(), "touchdown");
        }
        assert_eq!(DROP_COUNT.load(Ordering::Relaxed), 1);
    }

    // TODO: add a macro that generates a get_TraitId function
    //       https://stackoverflow.com/questions/71463576/how-to-use-a-macro-to-generate-compile-time-unique-integers
    //       https://stackoverflow.com/questions/27415011/can-a-rust-macro-create-new-identifiers
    //       https://users.rust-lang.org/t/idiomatic-rust-way-to-generate-unique-id/33805
    // TODO: would be nice to retain stringified trait and object names
    //       could then have a Debug impl that printed that
    //       does make Components heavier weight, maybe only do this for debug builds?
    //          or can we generate functions to get a string from an id? not sure how we'd call those
    //       can we use Formatter to optionally delegate to objects?
    // TODO: need a mutable find
    // TODO: add macros for add and find
    // TODO: add support for repeated traits?
    // TODO: support removing objects?
    // TODO: add support for more than two traits per object
    //       can probably use a build script (build.rs) to generate these
    //       https://doc.rust-lang.org/cargo/reference/build-script-examples.html
    // TODO: work on readme
}

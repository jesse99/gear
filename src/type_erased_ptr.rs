use super::*;
use std::marker::Unsize;
use std::mem::transmute;
use std::ops::{Deref, DerefMut};
use std::ptr::{self, DynMetadata, Pointee};
use std::sync::atomic::{AtomicU32, Ordering};

// Decomposed trait pointer.
pub struct TypeErasedPointer {
    pub object_id: TypeId,
    pointer: *mut (),
    metadata: Box<*const ()>,
}

impl TypeErasedPointer {
    pub fn from_trait<Object, Trait>(object_id: TypeId, pointer: *mut Object) -> Self
    where
        Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
        Object: Unsize<Trait>,
    {
        let (pointer, metadata) = (pointer as *mut Trait).to_raw_parts();
        let metadata = unsafe { transmute(Box::new(metadata)) };

        TypeErasedPointer {
            object_id,
            pointer,
            metadata,
        }
    }

    pub unsafe fn to_trait<'a, Trait>(&self, refs: &'a ObjectRefs) -> RefTrait<'a, Trait>
    where
        Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
    {
        let old = refs
            .immutable_refs
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        assert!(old < u32::MAX, "immutable_refs wrapped around");
        assert!(
            refs.mutable_refs.load(Ordering::Relaxed) == 0,
            "mutable reference already exists"
        );

        let src = self.metadata.as_ref();
        let metadata = unsafe { *transmute::<_, *const <Trait as Pointee>::Metadata>(src) };
        let typed_ptr = ptr::from_raw_parts_mut::<Trait>(self.pointer, metadata);
        RefTrait {
            trait_ptr: typed_ptr,
            refs,
        }
    }

    pub unsafe fn to_trait_mut<'a, Trait>(&self, refs: &'a ObjectRefs) -> RefMutTrait<'a, Trait>
    where
        Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
    {
        let old = refs
            .mutable_refs
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        assert!(old == 0, "mutable reference already exists");
        assert!(
            refs.immutable_refs.load(Ordering::Relaxed) == 0,
            "immutable_ref already exists"
        );

        let src = self.metadata.as_ref();
        let metadata = unsafe { *transmute::<_, *const <Trait as Pointee>::Metadata>(src) };
        let typed_ptr = ptr::from_raw_parts_mut::<Trait>(self.pointer, metadata);
        RefMutTrait {
            trait_ptr: typed_ptr,
            refs,
        }
    }
}

// Code can only get at these pointers except by going through the Component interface
// which owns the underlying object so it's safe for TypeErasedPointer to be Send+Sync.
// See https://doc.rust-lang.org/nomicon/send-and-sync.html for more.
unsafe impl Send for TypeErasedPointer {}
unsafe impl Sync for TypeErasedPointer {}

pub struct RefTrait<'a, Trait>
where
    Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
{
    trait_ptr: *mut Trait,
    refs: &'a ObjectRefs,
}

impl<'a, Trait> Deref for RefTrait<'a, Trait>
where
    Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
{
    type Target = Trait;

    fn deref(&self) -> &Trait {
        unsafe { &*self.trait_ptr }
    }
}

impl<'a, Trait> Drop for RefTrait<'a, Trait>
where
    Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
{
    fn drop(&mut self) {
        let old = self
            .refs
            .immutable_refs
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        assert!(old < u32::MAX, "immutable_refs wrapped around");
    }
}

pub struct RefMutTrait<'a, Trait>
where
    Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
{
    trait_ptr: *mut Trait,
    refs: &'a ObjectRefs,
}

impl<'a, Trait> Deref for RefMutTrait<'a, Trait>
where
    Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
{
    type Target = Trait;

    fn deref(&self) -> &Trait {
        unsafe { &*self.trait_ptr }
    }
}

impl<'a, Trait> DerefMut for RefMutTrait<'a, Trait>
where
    Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
{
    fn deref_mut(&mut self) -> &mut Trait {
        unsafe { &mut *self.trait_ptr }
    }
}

impl<'a, Trait> Drop for RefMutTrait<'a, Trait>
where
    Trait: ?Sized + Pointee<Metadata = DynMetadata<Trait>> + 'static,
{
    fn drop(&mut self) {
        let old = self
            .refs
            .mutable_refs
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        assert!(old < u32::MAX, "mutable_refs wrapped around");
    }
}
pub struct ObjectRefs {
    immutable_refs: AtomicU32,
    mutable_refs: AtomicU32,
}

impl ObjectRefs {
    pub fn new() -> ObjectRefs {
        ObjectRefs {
            immutable_refs: AtomicU32::new(0),
            mutable_refs: AtomicU32::new(0),
        }
    }
}

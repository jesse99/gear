#![feature(lazy_cell)]
#![feature(ptr_metadata)]
#![feature(unsize)]

mod component;
mod component_id;
mod type_id;

pub use component::*;
pub use component_id::*;
pub use type_id::*;

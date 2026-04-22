#[cfg(feature = "alloc")]
mod hash;
#[cfg(feature = "alloc")]
pub use hash::{HashMap, HashMapExt, HashSet, HashSetExt};

#[cfg(feature = "alloc")]
mod type_map;
#[cfg(feature = "alloc")]
pub use type_map::{TypeMap, UnsafeTypeMap};

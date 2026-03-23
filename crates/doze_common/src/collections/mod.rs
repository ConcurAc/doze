#[cfg(feature = "alloc")]
mod hash;
#[cfg(feature = "alloc")]
pub use hash::{HashMap, HashSet};

#[cfg(feature = "alloc")]
mod type_map;
#[cfg(feature = "alloc")]
pub use type_map::{TypeMap, UnsafeTypeMap};

// #[cfg(feature = "alloc")]
// mod history_buffer;
// #[cfg(feature = "alloc")]
// pub use history_buffer::HistoryBuffer;

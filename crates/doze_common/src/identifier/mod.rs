#[cfg(feature = "alloc")]
mod strong;
#[cfg(feature = "alloc")]
pub use strong::StrongIdentifier;

mod weak;
pub use weak::WeakIdentifier;

mod hash;
pub use hash::IdentifierHash;

// pub trait Identifier: Borrow<CStr> {
//     #[inline]
//     fn eq_str(&self, other: &str) -> bool {
//         let this_bytes = self.borrow().to_bytes();
//         let other_bytes = other.as_bytes();
//         if this_bytes.len() != other_bytes.len() {
//             return false;
//         }
//         this_bytes.iter().zip(other_bytes).all(|(c, s)| c == s)
//     }
// }

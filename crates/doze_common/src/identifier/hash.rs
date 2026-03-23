use core::borrow::Borrow;

use rapidhash::v3::rapidhash_v3 as hash;

use super::weak::WeakIdentifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IdentifierHash<T>(T);

impl<T> From<T> for IdentifierHash<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: Copy> IdentifierHash<T> {
    #[inline]
    pub fn get(&self) -> T {
        self.0
    }
}

impl<T> AsRef<T> for IdentifierHash<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<'i> From<WeakIdentifier<'i>> for IdentifierHash<u32> {
    #[inline]
    fn from(value: WeakIdentifier<'i>) -> Self {
        let hash = hash(value.borrow());
        Self(((hash >> size_of::<u32>()) ^ hash) as u32)
    }
}

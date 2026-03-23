use derive_more::{Deref, DerefMut};

use crate::storage::{LinearStorage, Storage};

#[derive(Debug, Default, Clone, Deref, DerefMut)]
pub struct Vec<T, const N: usize>(heapless::Vec<T, N>);

impl<T, const N: usize> Storage for Vec<T, N> {
    type Item = T;
    type Handle = usize;

    fn capacity(&self) -> usize {
        self.0.capacity()
    }
    fn len(&self) -> usize {
        self.0.len()
    }
    fn get(&self, handle: Self::Handle) -> Option<&Self::Item> {
        self.0.get(handle)
    }
    fn get_mut(&mut self, handle: Self::Handle) -> Option<&mut Self::Item> {
        self.0.get_mut(handle)
    }
}

impl<T, const N: usize> LinearStorage for Vec<T, N>
where
    Self: Storage<Item = T>,
{
    fn push(&mut self, item: T) -> Result<(), T> {
        self.0.push(item)
    }
    fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }
}

#[cfg(feature = "alloc")]
pub mod alloc {
    use derive_more::{Deref, DerefMut};

    use crate::storage::{LinearStorage, Storage};

    #[derive(Debug, Default, Clone, Deref, DerefMut)]
    pub struct Vec<T>(alloc::vec::Vec<T>);

    impl<T> Storage for Vec<T> {
        type Item = T;
        type Handle = usize;

        fn capacity(&self) -> usize {
            self.0.capacity()
        }
        fn len(&self) -> usize {
            self.0.len()
        }
        fn get(&self, handle: Self::Handle) -> Option<&Self::Item> {
            self.0.get(handle)
        }
        fn get_mut(&mut self, handle: Self::Handle) -> Option<&mut Self::Item> {
            self.0.get_mut(handle)
        }
    }

    impl<T> LinearStorage for Vec<T>
    where
        Self: Storage<Item = T>,
    {
        fn push(&mut self, item: T) -> Result<(), T> {
            Ok(self.0.push(item))
        }
        fn pop(&mut self) -> Option<T> {
            self.0.pop()
        }
    }

    impl<T> Vec<T> {
        pub fn with_capacity(capacity: usize) -> Self {
            Self(alloc::vec::Vec::with_capacity(capacity))
        }
    }
}

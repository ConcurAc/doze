use crate::storage::{LinearStorage, Storage, vec::Vec};

#[derive(Default)]
pub struct InnerHistory<S: LinearStorage> {
    storage: S,
    last: usize,
}

impl<S> Storage for InnerHistory<S>
where
    S: LinearStorage,
{
    type Item = S::Item;
    type Handle = S::Handle;

    fn capacity(&self) -> usize {
        self.storage.capacity()
    }
    fn len(&self) -> usize {
        self.storage.len()
    }

    fn get(&self, handle: Self::Handle) -> Option<&Self::Item> {
        self.storage.get(handle)
    }

    fn get_mut(&mut self, handle: Self::Handle) -> Option<&mut Self::Item> {
        self.storage.get_mut(handle)
    }
}

impl<T, S> InnerHistory<S>
where
    S: LinearStorage<Item = T, Handle = usize>,
{
    pub fn len(&self) -> usize {
        self.storage.len()
    }

    pub fn is_empty(&self) -> bool {
        self.storage.len() == 0
    }

    pub fn is_full(&self) -> bool {
        self.storage.len() == self.storage.capacity()
    }

    pub fn write(&mut self, item: T) {
        let cap = self.storage.capacity();
        let Err(item) = self.storage.push(item) else {
            return;
        };

        let Some(slot) = self.storage.get_mut(self.last) else {
            unreachable!()
        };
        *slot = item;
        self.last = (self.last + 1) % cap;
    }

    pub fn oldest(&self, steps: usize) -> Option<&T> {
        if steps >= self.len() {
            return None;
        }
        let cap = self.storage.capacity();
        let start = if self.is_full() { self.last } else { 0 };
        let i = (start + steps) % cap;
        self.storage.get(i)
    }

    pub fn recent(&self, steps: usize) -> Option<&T> {
        if steps >= self.len() {
            return None;
        }
        let cap = self.storage.capacity();
        let i = (self.last + cap - steps - 1) % cap;
        self.storage.get(i)
    }

    pub fn first(&self) -> Option<&T> {
        self.oldest(0)
    }

    pub fn last(&self) -> Option<&T> {
        self.recent(0)
    }

    pub fn clear(&mut self) {
        while self.storage.pop().is_some() {}
        self.last = 0;
    }
}

pub type History<T, const N: usize> = InnerHistory<Vec<T, N>>;

#[cfg(feature = "alloc")]
pub mod alloc {
    use crate::storage::vec::alloc::Vec;

    use super::InnerHistory;

    pub type History<T> = InnerHistory<Vec<T>>;

    impl<T> History<T> {
        pub fn with_capacity(capacity: usize) -> Self {
            Self {
                storage: Vec::with_capacity(capacity),
                last: 0,
            }
        }
    }
}

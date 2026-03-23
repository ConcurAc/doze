mod vec;

mod arena;
pub use arena::{Arena, ArenaHandle};

mod history;
pub use history::History;

pub trait Storage {
    type Item;
    type Handle;

    fn capacity(&self) -> usize;
    fn len(&self) -> usize;
    fn get(&self, handle: Self::Handle) -> Option<&Self::Item>;
    fn get_mut(&mut self, handle: Self::Handle) -> Option<&mut Self::Item>;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }
}

pub trait LinearStorage: Storage {
    fn push(&mut self, item: Self::Item) -> Result<(), Self::Item>;
    fn pop(&mut self) -> Option<Self::Item>;
}

pub trait NonLinearStorage: Storage {
    fn insert(&mut self, item: Self::Item) -> Result<Self::Handle, Self::Item>;
    fn remove(&mut self, handle: Self::Handle) -> Option<Self::Item>;
}

#[cfg(feature = "alloc")]
pub mod alloc {
    pub use super::{arena::alloc::Arena, history::alloc::History};
}

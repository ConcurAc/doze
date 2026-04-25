use crate::storage::{LinearStorage, NonLinearStorage, Storage, vec::Vec};

pub enum Slot<T> {
    Occupied {
        item: T,
        generation: u32,
        next: Option<usize>,
        prev: Option<usize>,
    },
    Free {
        next: Option<usize>,
        generation: u32,
    },
}

impl<T> Slot<T> {
    fn generation(&self) -> u32 {
        match self {
            Slot::Occupied { generation, .. } => *generation,
            Slot::Free { generation, .. } => *generation,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ArenaHandle {
    slot: usize,
    generation: u32,
}

#[derive(Default)]
pub struct InnerArena<T, S>
where
    S: LinearStorage<Item = Slot<T>>, {
    slots: S,
    occupied: Option<usize>,
    free: Option<usize>,
    count: usize,
}

impl<T, S> InnerArena<T, S>
where
    S: LinearStorage<Item = Slot<T>, Handle = usize>,
{
    fn link_at_head(&mut self, index: usize) {
        // Point new slot at current head
        if let Some(Slot::Occupied { next, .. }) = self.slots.get_mut(index) {
            *next = self.occupied;
        }
        // Back-link old head to new slot
        if let Some(head) = self.occupied {
            if let Some(Slot::Occupied { prev, .. }) = self.slots.get_mut(head) {
                *prev = Some(index);
            }
        }
        self.occupied = Some(index);
    }

    fn unlink(&mut self, index: usize) {
        let (next, prev) = match self.slots.get(index) {
            Some(Slot::Occupied { next, prev, .. }) => (*next, *prev),
            _ => return,
        };
        match prev {
            Some(p) => {
                if let Some(Slot::Occupied { next: p_next, .. }) = self.slots.get_mut(p) {
                    *p_next = next;
                }
            }
            None => self.occupied = next,
        }
        if let Some(n) = next {
            if let Some(Slot::Occupied { prev: n_prev, .. }) = self.slots.get_mut(n) {
                *n_prev = prev;
            }
        }
    }

    pub fn iter(&self) -> ArenaIterator<'_, T, S> {
        ArenaIterator {
            arena: self,
            next: self.occupied,
        }
    }
}

impl<T, S> Storage for InnerArena<T, S>
where
    S: LinearStorage<Item = Slot<T>, Handle = usize>,
{
    type Item = T;
    type Handle = ArenaHandle;

    fn capacity(&self) -> usize {
        self.slots.capacity()
    }

    fn len(&self) -> usize {
        self.count
    }

    fn get(&self, handle: Self::Handle) -> Option<&Self::Item> {
        match self.slots.get(handle.slot)? {
            Slot::Occupied {
                item, generation, ..
            } if *generation == handle.generation => Some(item),
            _ => None,
        }
    }

    fn get_mut(&mut self, handle: Self::Handle) -> Option<&mut Self::Item> {
        match self.slots.get_mut(handle.slot)? {
            Slot::Occupied {
                item, generation, ..
            } if *generation == handle.generation => Some(item),
            _ => None,
        }
    }
}

impl<T, S> NonLinearStorage for InnerArena<T, S>
where
    S: LinearStorage<Item = Slot<T>, Handle = usize>,
{
    fn insert(&mut self, item: T) -> Result<Self::Handle, Self::Item> {
        // Try to reuse a free slot
        if let Some(free_index) = self.free {
            if let Some(slot @ Slot::Free { .. }) = self.slots.get_mut(free_index) {
                let generation = slot.generation();
                self.free = match slot {
                    Slot::Free { next, .. } => *next,
                    _ => None,
                };

                *slot = Slot::Occupied {
                    item,
                    generation,
                    next: None,
                    prev: None,
                };

                self.link_at_head(free_index);
                self.count += 1;

                return Ok(ArenaHandle {
                    slot: free_index,
                    generation,
                });
            }
        }

        // Allocate a new slot
        let index = self.slots.len();
        self.slots
            .push(Slot::Occupied {
                item,
                generation: 0,
                next: None,
                prev: None,
            })
            .map_err(|slot| match slot {
                Slot::Occupied { item, .. } => item,
                Slot::Free { .. } => unreachable!(),
            })?;

        self.link_at_head(index);
        self.count += 1;

        Ok(ArenaHandle {
            slot: index,
            generation: 0,
        })
    }

    fn remove(&mut self, handle: Self::Handle) -> Option<Self::Item> {
        let generation = match self.slots.get(handle.slot)? {
            Slot::Occupied { generation, .. } if *generation == handle.generation => *generation,
            _ => return None,
        };

        self.unlink(handle.slot);

        let old = core::mem::replace(
            self.slots.get_mut(handle.slot)?,
            Slot::Free {
                next: self.free,
                generation: generation.saturating_add(1),
            },
        );

        self.free = Some(handle.slot);
        self.count -= 1;

        match old {
            Slot::Occupied { item, .. } => Some(item),
            Slot::Free { .. } => unreachable!(),
        }
    }
}

pub struct ArenaIterator<'a, T, S>
where
    S: LinearStorage<Item = Slot<T>>, {
    arena: &'a InnerArena<T, S>,
    next: Option<usize>,
}

impl<'a, T, S> Iterator for ArenaIterator<'a, T, S>
where
    S: LinearStorage<Item = Slot<T>, Handle = usize>,
{
    type Item = (ArenaHandle, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.next?;
        match self.arena.slots.get(index)? {
            Slot::Occupied {
                item,
                generation,
                next,
                ..
            } => {
                self.next = *next;
                Some((
                    ArenaHandle {
                        slot: index,
                        generation: *generation,
                    },
                    item,
                ))
            }
            _ => None,
        }
    }
}

pub type Arena<T, const N: usize> = InnerArena<T, Vec<Slot<T>, N>>;

#[cfg(feature = "alloc")]
pub mod alloc {
    use super::{InnerArena, Slot};
    use crate::storage::vec::alloc::Vec;

    pub type Arena<T> = InnerArena<T, Vec<Slot<T>>>;

    impl<T> Arena<T> {
        pub fn with_capacity(capacity: usize) -> Self {
            Self {
                slots: Vec::<Slot<T>>::with_capacity(capacity),
                occupied: None,
                free: None,
                count: 0,
            }
        }
    }
}

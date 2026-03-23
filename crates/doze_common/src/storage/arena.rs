use crate::storage::{LinearStorage, NonLinearStorage, Storage, vec::Vec};

pub enum Slot<T> {
    Occupied {
        item: T,
        generation: u32,
    },
    Free {
        next_free: Option<usize>,
        generation: u32,
    },
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
    free: Option<usize>,
    count: usize,
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
        self.slots.get(handle.slot).and_then(|slot| {
            if let Slot::Occupied { item, generation } = slot {
                if *generation == handle.generation {
                    Some(item)
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
    fn get_mut(&mut self, handle: Self::Handle) -> Option<&mut Self::Item> {
        self.slots.get_mut(handle.slot).and_then(|slot| {
            if let Slot::Occupied { item, generation } = slot {
                if *generation == handle.generation {
                    Some(item)
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
}

impl<T: Clone, S> NonLinearStorage for InnerArena<T, S>
where
    S: LinearStorage<Item = Slot<T>, Handle = usize>,
{
    fn insert(&mut self, item: T) -> Result<Self::Handle, Self::Item> {
        if let Some(head) = self.free {
            if let Some(slot) = self.slots.get_mut(head) {
                if let Slot::Free {
                    next_free,
                    generation,
                } = slot
                {
                    let generation = *generation;
                    self.free = *next_free;

                    *slot = Slot::Occupied { item, generation };

                    let handle = ArenaHandle {
                        slot: head,
                        generation,
                    };

                    self.count += 1;
                    return Ok(handle);
                }
            }
        }

        let Err(slot) = self.slots.push(Slot::Occupied {
            item,
            generation: 0,
        }) else {
            self.count += 1;
            return Ok(ArenaHandle {
                slot: self.slots.len() - 1,
                generation: 0,
            });
        };

        let Slot::Occupied { item, .. } = slot else {
            unreachable!()
        };

        return Err(item);
    }
    fn remove(&mut self, handle: Self::Handle) -> Option<Self::Item> {
        let slot = self.slots.get_mut(handle.slot)?;

        let generation = match slot {
            Slot::Occupied { generation, .. } => *generation + 1,
            Slot::Free { .. } => return None,
        };

        if generation != handle.generation {
            return None;
        }

        let old = core::mem::replace(
            slot,
            Slot::Free {
                next_free: self.free,
                generation,
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

pub type Arena<T, const N: usize> = InnerArena<T, Vec<Slot<T>, N>>;

#[cfg(feature = "alloc")]
pub mod alloc {
    use crate::storage::vec::alloc::Vec;

    use super::{InnerArena, Slot};

    pub type Arena<T> = InnerArena<T, Vec<Slot<T>>>;

    impl<T> Arena<T> {
        pub fn with_capacity(capacity: usize) -> Self {
            Self {
                slots: Vec::<Slot<T>>::with_capacity(capacity),
                free: None,
                count: 0,
            }
        }
    }
}

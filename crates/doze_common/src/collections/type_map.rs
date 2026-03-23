use core::any::{Any, TypeId};

use crate::collections::HashMap;

use derive_more::Deref;

#[derive(Deref)]
pub struct TypeMap<A>(HashMap<TypeId, A>);

impl<A: Any> TypeMap<A> {
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.0
            .get(&TypeId::of::<T>())
            .map(|a| (a as &dyn Any).downcast_ref())
            .flatten()
    }
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.0
            .get_mut(&TypeId::of::<T>())
            .map(|a| (a as &mut dyn Any).downcast_mut())
            .flatten()
    }
}

impl<A: Any> FromIterator<A> for TypeMap<A> {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        Self(iter.into_iter().map(|a| (TypeId::of::<A>(), a)).collect())
    }
}

impl<A: Any> FromIterator<(TypeId, A)> for TypeMap<A> {
    fn from_iter<T: IntoIterator<Item = (TypeId, A)>>(iter: T) -> Self {
        Self(HashMap::from_iter(iter))
    }
}

#[derive(Deref)]
pub struct UnsafeTypeMap(TypeMap<*mut ()>);

impl UnsafeTypeMap {
    pub unsafe fn get<T: 'static>(&self) -> Option<&T> {
        self.0
            .get::<*mut ()>()
            .map(|&p| unsafe { (p as *const T).as_ref() })
            .flatten()
    }
    pub unsafe fn get_mut<T: 'static>(&self) -> Option<&mut T> {
        self.0
            .get::<*mut ()>()
            .map(|&p| unsafe { (p as *mut T).as_mut() })
            .flatten()
    }
}

impl FromIterator<(TypeId, *mut ())> for UnsafeTypeMap {
    fn from_iter<T: IntoIterator<Item = (TypeId, *mut ())>>(iter: T) -> Self {
        Self(TypeMap(HashMap::from_iter(iter)))
    }
}

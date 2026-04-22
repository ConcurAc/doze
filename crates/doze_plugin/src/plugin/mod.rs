use std::{
    any::{Any, TypeId},
    hash::Hash,
    marker::PhantomData,
};

use doze_common::{collections::HashMap, identifier::StrongIdentifier};

use crate::{
    PluginApi,
    extensions::{Bridge, Extension},
    process::{Process, Status},
};

pub mod descriptor;
pub use descriptor::PluginDescriptor;

pub mod feature;
pub use feature::PluginFeature;

pub trait Plugin: Any + Send + Sync + 'static {
    fn init(&mut self);
    fn reset(&mut self);
    fn activate(&mut self, sample_rate: f64, min_frames_count: u32, max_frames_count: u32) -> bool;
    fn deactivate(&mut self);
    fn start_processing(&mut self) -> bool;
    fn stop_processing(&mut self);
    fn process(&mut self, state: Process) -> Status;
    fn on_main_thread(&mut self);
}

pub struct ExtensionRegistry<A: PluginApi> {
    map: HashMap<TypeId, A::Extension>,
    _api: PhantomData<A>,
}

impl<A: PluginApi> FromIterator<A::Extension> for ExtensionRegistry<A> {
    fn from_iter<T: IntoIterator<Item = A::Extension>>(iter: T) -> Self {
        Self {
            map: iter
                .into_iter()
                .map(|b| (b.extension().type_id(), b))
                .collect(),
            _api: PhantomData,
        }
    }
}

impl<A: PluginApi> ExtensionRegistry<A> {
    pub fn get<E: Extension>(&self) -> Option<&E> {
        self.map.get(&TypeId::of::<E>()).and_then(|b| b.get::<E>())
    }
}

pub struct EntityRegistry<Id: Eq + Hash>(HashMap<Id, Entity>);

impl<Id: Eq + Hash> From<HashMap<Id, Entity>> for EntityRegistry<Id> {
    fn from(map: HashMap<Id, Entity>) -> Self {
        EntityRegistry(map)
    }
}

impl<Id: Eq + Hash> FromIterator<(Id, Entity)> for EntityRegistry<Id> {
    fn from_iter<T: IntoIterator<Item = (Id, Entity)>>(iter: T) -> Self {
        EntityRegistry(HashMap::from_iter(iter))
    }
}

impl<Id: Eq + Hash> EntityRegistry<Id> {
    pub fn get(&self, id: &Id) -> Option<&Entity> {
        self.0.get(id)
    }
}

pub struct Entity {
    pub index: usize,
    pub identifier: StrongIdentifier,
}

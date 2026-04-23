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

/// Core plugin interface defining lifecycle and audio processing.
///
/// Implement this trait to create an audio plugin. The plugin receives lifecycle
/// callbacks and processes audio frames sent from the host.
pub trait Plugin: Any + Send + Sync + 'static {
    /// Initialize plugin state (called once after instantiation).
    fn init(&mut self) {}

    /// Reset plugin state to initial condition.
    fn reset(&mut self) {}

    /// Activate the plugin with runtime configuration.
    ///
    /// Called before `start_processing()`. The plugin learns the sample rate and buffer size constraints.
    fn activate(&mut self, sample_rate: f64, min_frames_count: u32, max_frames_count: u32) -> bool;

    /// Deactivate the plugin and release resources.
    fn deactivate(&mut self) {}

    /// Begin audio processing (called before first `process()` call).
    fn start_processing(&mut self) -> bool;

    /// Stop audio processing (called after last `process()` call).
    fn stop_processing(&mut self) {}

    /// Main audio processing callback (real-time thread, must be real-time safe).
    fn process(&mut self, state: Process) -> Status;

    /// Main thread callback for non-real-time work (UI updates, state persistence, etc.).
    fn on_main_thread(&mut self) {}
}

/// Registry of extensions by their type ID.
///
/// Stores implementations of `Extension` traits, allowing the host to query
/// the plugin for capabilities (e.g., "Does this plugin have a Params extension?").
pub struct ExtensionRegistry<A: PluginApi> {
    map: HashMap<TypeId, A::Extension>,
    _api: PhantomData<A>,
}

impl<A: PluginApi> FromIterator<A::Extension> for ExtensionRegistry<A> {
    /// Constructs a registry from a collection of extensions.
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
    /// Retrieve a specific extension by its trait type.
    pub fn get<E: Extension>(&self) -> Option<&E> {
        self.map.get(&TypeId::of::<E>()).and_then(|b| b.get::<E>())
    }
}

/// Registry mapping identifiers to entities by index.
///
/// Associates a stable symbolic identifier with an indexed entity (e.g., Port ID "in", Param ID "gain").
pub struct EntityRegistry<Id: Eq + Hash>(HashMap<Id, Entity>);

impl<Id: Eq + Hash> From<HashMap<Id, Entity>> for EntityRegistry<Id> {
    fn from(map: HashMap<Id, Entity>) -> Self {
        EntityRegistry(map)
    }
}

impl<Id: Eq + Hash> FromIterator<(Id, Entity)> for EntityRegistry<Id> {
    /// Constructs a registry from an iterator of (ID, Entity) pairs.
    fn from_iter<T: IntoIterator<Item = (Id, Entity)>>(iter: T) -> Self {
        EntityRegistry(HashMap::from_iter(iter))
    }
}

impl<Id: Eq + Hash> EntityRegistry<Id> {
    /// Look up an entity by its ID.
    pub fn get(&self, id: &Id) -> Option<&Entity> {
        self.0.get(id)
    }
}

/// Represents a single entity within a registry (e.g., a parameter or port).
#[derive(Debug, Clone)]
pub struct Entity {
    /// Index in the registry (0-based).
    ///
    /// Must remain stable across the entire plugin session.
    /// The host caches this index for performance, so changing it invalidates host state.
    pub index: usize,
    /// Stable symbolic identifier (e.g., "gain", "output_main").
    ///
    /// Used by the host to remember this entity across sessions.
    pub identifier: StrongIdentifier,
}

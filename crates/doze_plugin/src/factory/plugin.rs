use crate::{
    PluginApi,
    extensions::{Bridge, Extension},
    plugin::{Plugin, descriptor::PluginDescriptor},
};

use doze_common::identifier::WeakIdentifier;

use std::{borrow::Borrow, marker::PhantomData};

pub trait PluginFactory {
    fn descriptor_count(&self) -> usize;
    fn get_descriptor(&self, index: usize) -> Option<impl Borrow<PluginDescriptor>>;
    fn create_plugin(&self, identifier: WeakIdentifier) -> Option<Box<dyn Plugin>>;
}

#[derive(Clone)]
pub struct PluginFactoryBuilder<A: PluginApi> {
    definitions: Vec<PluginDefinition<A>>,
}

impl<A: PluginApi> IntoIterator for PluginFactoryBuilder<A> {
    type Item = PluginDefinition<A>;
    type IntoIter = std::vec::IntoIter<PluginDefinition<A>>;

    fn into_iter(self) -> Self::IntoIter {
        self.definitions.into_iter()
    }
}

impl<A: PluginApi> PluginFactoryBuilder<A> {
    pub fn new() -> Self {
        Self {
            definitions: Vec::new(),
        }
    }
    pub fn get_definitions(&self) -> &[PluginDefinition<A>] {
        &self.definitions
    }
    pub fn add_plugin(mut self, definition: PluginDefinition<A>) -> Self {
        self.definitions.push(definition);
        self
    }
}

#[derive(Clone)]
pub struct PluginDefinition<A: PluginApi> {
    pub creator: fn() -> Box<dyn Plugin>,
    pub descriptor: PluginDescriptor,
    pub extensions: Vec<A::Extension>,
    pub event_capacity: usize,
}

pub struct PluginBuilder<A: PluginApi, P: Plugin> {
    creator: Option<fn() -> Box<dyn Plugin>>,
    descriptor: Option<PluginDescriptor>,
    event_capacity: Option<usize>,
    extensions: Vec<A::Extension>,

    _api: PhantomData<A>,
    _plugin: PhantomData<P>,
}

impl<A: PluginApi, P: Plugin> Default for PluginBuilder<A, P> {
    fn default() -> Self {
        Self {
            creator: None,
            descriptor: None,
            event_capacity: None,
            extensions: Vec::new(),
            _api: PhantomData,
            _plugin: PhantomData,
        }
    }
}

impl<A: PluginApi, P: Plugin> PluginBuilder<A, P> {
    pub fn set_creator(mut self, creator: fn() -> Box<dyn Plugin>) -> Self {
        self.creator = Some(creator);
        self
    }
    pub fn set_descriptor(mut self, descriptor: PluginDescriptor) -> Self {
        self.descriptor = Some(descriptor);
        self
    }
    pub fn set_event_capacity(mut self, capacity: usize) -> Self {
        self.event_capacity = Some(capacity);
        self
    }
    pub fn add_extension<E: Extension>(mut self, extension: E) -> Self {
        let bridge = <A::Extension as Bridge>::wrap(extension);
        self.extensions.push(bridge);
        self
    }
}

const DEFAULT_EVENT_CAPACITY: usize = 32;

impl<A: PluginApi, P: Plugin> Into<PluginDefinition<A>> for PluginBuilder<A, P> {
    fn into(self) -> PluginDefinition<A> {
        PluginDefinition {
            creator: self.creator.expect("plugin creator not specified"),
            descriptor: self
                .descriptor
                .clone()
                .expect("plugin descriptor not provided"),
            event_capacity: self.event_capacity.unwrap_or(DEFAULT_EVENT_CAPACITY),
            extensions: self.extensions,
        }
    }
}

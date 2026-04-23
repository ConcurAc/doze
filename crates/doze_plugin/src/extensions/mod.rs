use std::any::Any;

use doze_common::identifier::StrongIdentifier;

use crate::plugin::Plugin;

pub mod audio_ports;
pub mod params;

pub trait Extension: Any + Send + Sync {
    fn as_registry_source(&self) -> Option<&dyn RegistrySource> {
        None
    }
}

pub trait PluginAccess<P: Plugin> {
    fn get(plugin: &dyn Plugin) -> &P {
        (plugin as &dyn Any)
            .downcast_ref()
            .expect("extension does not match plugin")
    }
    fn get_mut(plugin: &mut dyn Plugin) -> &mut P {
        (plugin as &mut dyn Any)
            .downcast_mut()
            .expect("extension does not match plugin")
    }
}

pub trait RegistrySource: Extension {
    fn identifiers(&self, plugin: &dyn Plugin) -> Vec<StrongIdentifier>;
}

pub trait Bridge {
    fn wrap<E: Extension>(extension: E) -> Self;
    fn extension(&self) -> &dyn Extension;
    fn get<E: Extension>(&self) -> Option<&E> {
        (self.extension() as &dyn Any).downcast_ref()
    }
}

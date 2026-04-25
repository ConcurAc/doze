use std::{ffi::c_void, sync::Arc};

use doze_common::identifier::WeakIdentifier;
use doze_plugin::extensions::{Bridge, Extension};

pub mod audio_ports;
pub mod params;

pub type ClapExtensions = &'static [ClapExtensionDescriptor];

#[derive(Clone)]
pub struct ClapBridge {
    pub payload: ClapExtensions,
    pub extension: Arc<dyn Extension>,
}

impl Bridge for ClapBridge {
    fn wrap<E: Extension>(extension: E) -> Self {
        Self {
            payload: <ClapExtern as ClapPayload<E>>::get(),
            extension: Arc::new(extension),
        }
    }
    fn extension(&self) -> &dyn Extension {
        self.extension.as_ref()
    }
}

pub struct ClapExtensionDescriptor {
    pub id: WeakIdentifier<'static>,
    pub vtable: *const c_void,
}

// *const c_void vtable points to a vtable and is read only
unsafe impl Send for ClapExtensionDescriptor {}
unsafe impl Sync for ClapExtensionDescriptor {}

pub trait ClapPayload<E: Extension> {
    fn get() -> ClapExtensions;
}

pub struct ClapExtern;

impl<E: Extension> ClapPayload<E> for ClapExtern {
    default fn get() -> ClapExtensions {
        &[]
    }
}

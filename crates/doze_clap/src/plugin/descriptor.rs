use std::{ffi::c_char, ptr::null};

use clap_sys::{plugin::clap_plugin_descriptor, version::CLAP_VERSION};

use doze_common::identifier::{StrongIdentifier, WeakIdentifier};
use doze_plugin::plugin::descriptor::PluginDescriptor;

use super::feature::feature_as_clap;

struct InnerClapPluginDescriptor {
    id: StrongIdentifier,
    name: StrongIdentifier,
    vendor: StrongIdentifier,
    version: StrongIdentifier,
    url: Option<StrongIdentifier>,
    manual_url: Option<StrongIdentifier>,
    support_url: Option<StrongIdentifier>,
    description: Option<StrongIdentifier>,
    features: Vec<StrongIdentifier>,

    /// The contains pointers to the strings in `clap_features`.
    feature_ptrs: Vec<*const c_char>,
}

impl AsRef<Self> for InnerClapPluginDescriptor {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl Into<clap_plugin_descriptor> for &InnerClapPluginDescriptor {
    fn into(self) -> clap_plugin_descriptor {
        clap_plugin_descriptor {
            clap_version: CLAP_VERSION,
            id: self.id.downgrade().as_cstr().as_ptr(),
            name: self.name.downgrade().as_cstr().as_ptr(),
            vendor: self.vendor.downgrade().as_cstr().as_ptr(),
            version: self.version.downgrade().as_cstr().as_ptr(),
            url: self
                .url
                .as_ref()
                .map_or_else(null, |url| url.downgrade().as_cstr().as_ptr()),
            manual_url: self
                .manual_url
                .as_ref()
                .map_or_else(null, |url| url.downgrade().as_cstr().as_ptr()),
            support_url: self
                .support_url
                .as_ref()
                .map_or_else(null, |url| url.downgrade().as_cstr().as_ptr()),
            description: self.description.as_ref().map_or_else(null, |description| {
                description.downgrade().as_cstr().as_ptr()
            }),
            features: self.feature_ptrs.as_ptr(),
        }
    }
}

// SAFETY
// *const c_char points to data owned by the struct,
// so pointers are valid so long as the struct lives.
unsafe impl Send for InnerClapPluginDescriptor {}

// SAFETY
// Struct is read only with *const c_char as well as other fields
// entirely private. Raw memory addresses for this struct are shared
// over FFI, external modificaton is possible, we assume this data
// will be treated as read only across FFI.
unsafe impl Sync for InnerClapPluginDescriptor {}

pub struct ClapPluginDescriptor {
    descriptor: clap_plugin_descriptor,
    _inner: InnerClapPluginDescriptor,
}

impl ClapPluginDescriptor {
    pub fn get(&self) -> &clap_plugin_descriptor {
        &self.descriptor
    }
}

impl<'d> From<PluginDescriptor> for ClapPluginDescriptor {
    /// Construct the plugin descriptor for a specific CLAP plugin.
    fn from(descriptor: PluginDescriptor) -> Self {
        let mut inner = InnerClapPluginDescriptor {
            id: descriptor.id,
            name: descriptor.name,
            vendor: descriptor.vendor,
            url: descriptor.url,
            version: descriptor.version,
            manual_url: descriptor.manual_url,
            support_url: descriptor.support_url,
            description: descriptor.description,
            features: descriptor
                .features
                .iter()
                .filter_map(|feat| feature_as_clap(feat))
                .map(|s| WeakIdentifier::from(s).into())
                .collect(),

            // Contain pointers to the fields in this descriptor
            feature_ptrs: Vec::new(),
        };

        // List of char pointers terminated by a null pointer.
        inner.feature_ptrs = inner
            .features
            .iter()
            .map(|feature| feature.downgrade().as_cstr().as_ptr())
            .collect();
        inner.feature_ptrs.push(std::ptr::null());

        Self {
            descriptor: inner.as_ref().into(),
            _inner: inner,
        }
    }
}

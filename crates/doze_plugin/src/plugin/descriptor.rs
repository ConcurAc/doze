use doze_common::identifier::StrongIdentifier;

use super::feature::PluginFeature;

#[derive(Clone)]
pub struct PluginDescriptor {
    pub id: StrongIdentifier,
    pub name: StrongIdentifier,
    pub vendor: StrongIdentifier,
    pub version: StrongIdentifier,
    pub url: Option<StrongIdentifier>,
    pub manual_url: Option<StrongIdentifier>,
    pub support_url: Option<StrongIdentifier>,
    pub description: Option<StrongIdentifier>,
    pub features: Vec<PluginFeature>,
}

impl AsRef<Self> for PluginDescriptor {
    fn as_ref(&self) -> &Self {
        self
    }
}

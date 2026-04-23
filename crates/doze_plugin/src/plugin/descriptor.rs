use doze_common::identifier::StrongIdentifier;

use super::feature::PluginFeature;

/// Complete metadata describing a plugin to the host.
///
/// This struct provides all information the host needs to identify, categorize,
/// and display information about the plugin.
///
/// # Fields
/// - `id`: Unique plugin identifier (e.g., "com.company.plugin")
/// - `name`: Display name
/// - `vendor`: Developer/company name
/// - `version`: Plugin version
/// - `url`: Plugin website
/// - `manual_url`: Link to plugin documentation
/// - `support_url`: Link to issue tracker or support
/// - `description`: Long-form plugin description
/// - `features`: Plugin type and capabilities
#[derive(Debug, Clone)]
pub struct PluginDescriptor {
    /// Unique plugin identifier (domain-style).
    ///
    /// Must be globally unique and stable across plugin versions.
    /// Should follow reverse domain naming convention.
    ///
    /// This ID should not change between plugin versions, as the host
    /// uses it to identify which plugin instance to load.
    pub id: StrongIdentifier,

    /// Display name for the plugin.
    ///
    /// Shown in the host's plugin list and UI. Can include spaces and special characters.
    pub name: StrongIdentifier,

    /// Developer or company name.
    ///
    /// Shown alongside the plugin name to identify the creator.
    pub vendor: StrongIdentifier,

    /// Plugin version string.
    ///
    /// Used by the host to check for updates and maintain compatibility.
    /// Should follow semantic versioning (e.g., "1.0.0", "2.1.3").
    pub version: StrongIdentifier,

    /// Plugin website URL (optional).
    ///
    /// Link to the plugin's main website or product page.
    /// The host may provide this link to the user.
    pub url: Option<StrongIdentifier>,

    /// Plugin manual/documentation URL (optional).
    ///
    /// Link to comprehensive documentation or manual.
    /// Users can access this from the host's help menu.
    pub manual_url: Option<StrongIdentifier>,

    /// Plugin support/issue tracking URL (optional).
    ///
    /// Link to bug reports, feature requests, or support forum.
    /// Where users can get help or report issues.
    pub support_url: Option<StrongIdentifier>,

    /// Long-form plugin description (optional).
    ///
    /// Detailed description of what the plugin does and its features.
    pub description: Option<StrongIdentifier>,

    /// Feature tags for categorization and filtering.
    ///
    /// Multiple features can be specified to describe the plugin completely.
    /// Expected to contain either `PluginFeature::Instrument` or `PluginFeature::AudioEffect`
    /// Examples: `vec![AudioEffect, Reverb, Stereo]` or `vec![Instrument, Synthesizer, Polyphonic]`
    pub features: Vec<PluginFeature>,
}

impl AsRef<Self> for PluginDescriptor {
    fn as_ref(&self) -> &Self {
        self
    }
}

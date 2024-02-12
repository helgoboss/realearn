use crate::util::deserialize_null_default;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Meta data that is common to both main and controller presets.
///
/// Preset meta data is everything that is loaded right at startup in order to be able to
/// display a list of preset, do certain validations etc. It doesn't include the preset
/// content which is necessary to actually use the preset (e.g. it doesn't include the mappings).
#[derive(Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct CommonPresetMetaData {
    /// Display name of the preset.
    pub name: String,
    /// The ReaLearn version for which this preset was built.
    ///
    /// This can effect the way the preset is loaded, e.g. it can lead to different interpretation
    /// or migration of properties. So care should be taken to set this correctly!
    ///
    /// If `None`, it's assumed that it was built for a very old version (< 1.12.0-pre18) that
    /// didn't have the versioning concept yet.
    #[serde(
        default,
        deserialize_with = "deserialize_null_default",
        skip_serializing_if = "Option::is_none"
    )]
    #[serde(alias = "version")]
    pub realearn_version: Option<Version>,
    /// Author of the preset.
    #[serde(
        default,
        deserialize_with = "deserialize_null_default",
        skip_serializing_if = "Option::is_none"
    )]
    pub author: Option<String>,
    /// Preset description (prose).
    #[serde(
        default,
        deserialize_with = "deserialize_null_default",
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
    /// Preset setup instructions (prose).
    #[serde(
        default,
        deserialize_with = "deserialize_null_default",
        skip_serializing_if = "Option::is_none"
    )]
    pub setup_instructions: Option<String>,
}

/// Meta data that is specific to controller presets.
#[derive(Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct ControllerPresetMetaData {
    /// Device manufacturer.
    #[serde(
        default,
        deserialize_with = "deserialize_null_default",
        skip_serializing_if = "Option::is_none"
    )]
    pub device_manufacturer: Option<String>,
    /// Original name of the device.
    #[serde(
        default,
        deserialize_with = "deserialize_null_default",
        skip_serializing_if = "Option::is_none"
    )]
    pub device_name: Option<String>,
    /// MIDI identity compatibility pattern.
    ///
    /// Will be used for auto-adding controllers and for finding the correct controller preset when calculating auto
    /// units.
    #[serde(
        default,
        deserialize_with = "deserialize_null_default",
        skip_serializing_if = "Option::is_none"
    )]
    pub midi_identity_pattern: Option<String>,
    /// Possible MIDI identity compatibility patterns.
    ///
    /// Will be used for auto-adding controllers and for finding the correct controller preset when calculating auto
    /// units.
    ///
    /// It should only be provided if the device in question doesn't reply to device queries or if it exposes
    /// multiple ports which all respond with the same device identity and only one of the ports is the correct one.
    /// Example: APC Key 25 mk2, which exposes a "Control" and a "Keys" port.
    ///
    /// It's a list because names often differ between operating systems. ReaLearn will match any in the list.
    #[serde(default)]
    pub midi_output_port_patterns: Vec<String>,
    /// Provided virtual control schemes.
    ///
    /// Will be used for finding the correct controller preset when calculating auto units.
    #[serde(default)]
    pub provided_schemes: HashSet<VirtualControlSchemeId>,
}

/// Meta data that is specific to main presets.
#[derive(Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct MainPresetMetaData {
    /// Used virtual control schemes.
    ///
    /// Will be used for finding the correct controller preset when calculating auto units.
    #[serde(default)]
    pub used_schemes: HashSet<VirtualControlSchemeId>,
    /// A set of features that a Helgobox instance needs to provide for the preset to make sense.
    ///
    /// See [instance_features].
    ///
    /// Will be used for determining whether an auto unit should be created for a specific instance
    /// or not. Example: If the required feature is "playtime" and a controller is configured with
    /// this main preset but the instance doesn't contain a Playtime Clip Matrix, this instance will
    /// not load the main preset.
    #[serde(default)]
    pub required_features: HashSet<String>,
}

impl MainPresetMetaData {
    pub fn requires_playtime(&self) -> bool {
        self.required_features.contains(instance_features::PLAYTIME)
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct VirtualControlSchemeId(String);

impl VirtualControlSchemeId {
    pub fn get(&self) -> &str {
        &self.0
    }
}

/// Known instance features.
pub mod instance_features {
    /// Instance owns a Playtime Clip Matrix.
    pub const PLAYTIME: &str = "playtime";
}

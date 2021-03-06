use crate::application::{GroupModel, MappingModel, ParameterSetting, SharedGroup, SharedMapping};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;

pub trait Preset: Clone + Debug {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn default_group(&self) -> &GroupModel;
    fn groups(&self) -> &Vec<GroupModel>;
    fn mappings(&self) -> &Vec<MappingModel>;
    fn parameters(&self) -> &HashMap<u32, ParameterSetting>;
}

pub trait PresetManager: fmt::Debug {
    type PresetType;

    fn find_by_id(&self, id: &str) -> Option<Self::PresetType>;

    fn mappings_are_dirty(&self, id: &str, mappings: &[SharedMapping]) -> bool;

    fn parameter_settings_are_dirty(
        &self,
        id: &str,
        parameter_settings: &HashMap<u32, ParameterSetting>,
    ) -> bool;

    fn groups_are_dirty(
        &self,
        id: &str,
        default_group: &SharedGroup,
        groups: &[SharedGroup],
    ) -> bool;
}

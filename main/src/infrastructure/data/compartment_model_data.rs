use crate::application::{CompartmentModel, GroupModel};
use crate::base::default_util::is_default;
use crate::domain::{
    Compartment, CompartmentParamIndex, GroupId, GroupKey, MappingId, MappingKey, ParamSetting,
};
use crate::infrastructure::data::{
    GroupModelData, MappingModelData, MigrationDescriptor, ModelToDataConversionContext,
    SimpleDataToModelConversionContext,
};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::hash::Hash;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompartmentModelData {
    #[serde(default, skip_serializing_if = "is_default")]
    pub default_group: Option<GroupModelData>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub groups: Vec<GroupModelData>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub mappings: Vec<MappingModelData>,
    // String key workaround because otherwise deserialization doesn't work with flattening.
    // (https://github.com/serde-rs/serde/issues/1183)
    #[serde(default, skip_serializing_if = "is_default")]
    pub parameters: HashMap<String, ParamSetting>,
    /// At the moment, custom data is only used in the controller compartment.
    #[serde(default, skip_serializing_if = "is_default")]
    pub custom_data: HashMap<String, serde_json::Value>,
}

impl ModelToDataConversionContext for CompartmentModel {
    fn non_default_group_key_by_id(&self, group_id: GroupId) -> Option<GroupKey> {
        let group = self.groups.iter().find(|g| g.id() == group_id)?;
        Some(group.key().clone())
    }

    fn mapping_key_by_id(&self, mapping_id: MappingId) -> Option<MappingKey> {
        let mapping = self.mappings.iter().find(|m| m.id() == mapping_id)?;
        Some(mapping.key().clone())
    }
}

impl CompartmentModelData {
    pub fn from_model(model: &CompartmentModel) -> Self {
        Self {
            default_group: Some(GroupModelData::from_model(&model.default_group, model)),
            groups: model
                .groups
                .iter()
                .map(|g| GroupModelData::from_model(g, model))
                .collect(),
            mappings: model
                .mappings
                .iter()
                .map(|m| MappingModelData::from_model(m, model))
                .collect(),
            parameters: model
                .parameters
                .iter()
                .map(|(key, value)| (key.to_string(), value.clone()))
                .collect(),
            custom_data: model.custom_data.clone(),
        }
    }

    pub fn to_model(
        &self,
        version: Option<&Version>,
        compartment: Compartment,
    ) -> Result<CompartmentModel, String> {
        ensure_no_duplicate_compartment_data(
            &self.mappings,
            &self.groups,
            self.parameters.values(),
        )?;
        let migration_descriptor = MigrationDescriptor::new(version);
        let conversion_context =
            SimpleDataToModelConversionContext::new(&self.groups, &self.mappings);
        let final_default_group = self
            .default_group
            .as_ref()
            .map(|g| g.to_model(compartment, true, &conversion_context))
            .unwrap_or_else(|| GroupModel::default_for_compartment(compartment));
        let groups = self
            .groups
            .iter()
            .map(|g| g.to_model(compartment, false, &conversion_context))
            .collect();
        let model = CompartmentModel {
            default_group: final_default_group,
            mappings: self
                .mappings
                .iter()
                .map(|m| {
                    m.to_model_for_preset(
                        compartment,
                        &migration_descriptor,
                        version,
                        &conversion_context,
                    )
                })
                .collect(),
            parameters: self
                .parameters
                .iter()
                .filter_map(|(key, value)| {
                    let index: CompartmentParamIndex = key.parse::<u32>().ok()?.try_into().ok()?;
                    Some((index, value.clone()))
                })
                .collect(),
            groups,
            custom_data: self.custom_data.clone(),
        };
        Ok(model)
    }
}

pub fn ensure_no_duplicate_compartment_data<'a>(
    mappings: &[MappingModelData],
    groups: &[GroupModelData],
    parameters: impl Iterator<Item = &'a ParamSetting>,
) -> Result<(), String> {
    ensure_no_duplicate("mapping IDs", mappings.iter().filter_map(|m| m.id.as_ref()))?;
    ensure_no_duplicate(
        "group IDs",
        groups
            .iter()
            .filter_map(|g| if g.id.is_empty() { None } else { Some(&g.id) }),
    )?;
    ensure_no_duplicate("parameter IDs", parameters.filter_map(|p| p.key.as_ref()))?;
    Ok(())
}

#[allow(clippy::unnecessary_filter_map)]
pub fn ensure_no_duplicate<T>(list_label: &str, iter: T) -> Result<(), String>
where
    T: IntoIterator,
    T::Item: Eq + Hash + Display,
{
    use std::fmt::Write;
    let mut uniq = HashSet::new();
    let duplicates: HashSet<_> = iter
        .into_iter()
        .filter_map(|d| {
            if uniq.contains(&d) {
                Some(d)
            } else {
                uniq.insert(d);
                None
            }
        })
        .collect();
    if duplicates.is_empty() {
        Ok(())
    } else {
        let mut s = format!("Found the following duplicate {}: ", list_label);
        for (i, d) in duplicates.into_iter().enumerate() {
            if i > 0 {
                s.push_str(", ");
            }
            let _ = write!(&mut s, "{}", d);
        }
        Err(s)
    }
}

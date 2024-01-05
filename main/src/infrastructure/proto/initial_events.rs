use crate::infrastructure::plugin::{BackboneShell, InstanceShell};
use crate::infrastructure::proto::{
    occasional_global_update, occasional_instance_update, occasional_matrix_update,
    occasional_track_update, qualified_occasional_clip_update, qualified_occasional_slot_update,
    ClipAddress, OccasionalGlobalUpdate, OccasionalInstanceUpdate, OccasionalMatrixUpdate,
    OccasionalTrackUpdate, QualifiedOccasionalClipUpdate, QualifiedOccasionalSlotUpdate,
    QualifiedOccasionalTrackUpdate, SlotAddress,
};
use playtime_clip_engine::base::{Matrix, PlaytimeTrackInputProps};
use reaper_high::{Guid, OrCurrentProject, Track};
use std::collections::HashMap;
use std::iter;

pub fn create_initial_global_updates() -> Vec<OccasionalGlobalUpdate> {
    use occasional_global_update::Update;
    fn create(updates: impl Iterator<Item = Update>) -> Vec<OccasionalGlobalUpdate> {
        updates
            .into_iter()
            .map(|u| OccasionalGlobalUpdate { update: Some(u) })
            .collect()
    }
    let global_updates = [
        Update::midi_input_devices(),
        Update::midi_output_devices(),
        // TODO-high-ms3 Update when changed
        Update::audio_input_channels(),
        Update::controller_presets(&BackboneShell::get().controller_preset_manager().borrow()),
        Update::main_presets(&BackboneShell::get().main_preset_manager().borrow()),
        Update::controller_config(&BackboneShell::get().controller_manager().borrow()),
    ];
    create(global_updates.into_iter())
}

pub fn create_initial_instance_updates(
    instance_shell: &InstanceShell,
) -> Vec<OccasionalInstanceUpdate> {
    use occasional_instance_update::Update;
    fn create(updates: impl Iterator<Item = Update>) -> Vec<OccasionalInstanceUpdate> {
        updates
            .into_iter()
            .map(|u| OccasionalInstanceUpdate { update: Some(u) })
            .collect()
    }
    let instance_updates = [Update::settings(instance_shell)];
    create(instance_updates.into_iter())
}

pub fn create_initial_matrix_updates(matrix: Option<&Matrix>) -> Vec<OccasionalMatrixUpdate> {
    use occasional_matrix_update::Update;
    fn create(updates: impl Iterator<Item = Update>) -> Vec<OccasionalMatrixUpdate> {
        updates
            .into_iter()
            .map(|u| OccasionalMatrixUpdate { update: Some(u) })
            .collect()
    }
    let Some(matrix) = matrix else {
        return create(iter::once(Update::MatrixExists(false)));
    };
    let project = matrix.permanent_project().or_current_project();
    let master_track = project.master_track().expect("project gone");
    let updates = [
        Update::MatrixExists(true),
        Update::volume(master_track.volume().db()),
        Update::pan(master_track.pan().reaper_value()),
        Update::mute(master_track.is_muted()),
        Update::tempo(matrix.tempo()),
        Update::arrangement_play_state(project.play_state()),
        Update::sequencer_play_state(matrix.sequencer().status()),
        Update::complete_persistent_data(matrix),
        Update::history_state(matrix),
        Update::click_enabled(matrix),
        Update::silence_mode(matrix),
        Update::time_signature(project),
        Update::track_list(project),
        Update::simple_mappings(matrix),
        Update::learn_state(matrix),
        Update::active_slot(matrix),
    ];
    create(updates.into_iter())
}

pub fn create_initial_track_updates(
    matrix: Option<&Matrix>,
) -> Vec<QualifiedOccasionalTrackUpdate> {
    let Some(matrix) = matrix else {
        return vec![];
    };
    let track_by_guid: HashMap<Guid, Track> = matrix
        .columns()
        .flat_map(|column| {
            column
                .playback_track()
                .into_iter()
                .cloned()
                .chain(column.effective_recording_track())
        })
        .map(|track| (*track.guid(), track))
        .collect();
    track_by_guid
        .into_iter()
        .map(|(guid, track)| {
            let input_props = PlaytimeTrackInputProps::from_reaper_track(&track);
            use occasional_track_update::Update;
            QualifiedOccasionalTrackUpdate {
                track_id: guid.to_string_without_braces(),
                track_updates: [
                    Update::name(&track),
                    Update::color(&track),
                    Update::input(track.recording_input()),
                    Update::armed(input_props.armed),
                    Update::input_monitoring(input_props.input_monitoring),
                    Update::mute(track.is_muted()),
                    Update::solo(track.is_solo()),
                    Update::selected(track.is_selected()),
                    Update::volume(track.volume().db()),
                    Update::pan(track.pan().reaper_value()),
                ]
                .into_iter()
                .map(|update| OccasionalTrackUpdate {
                    update: Some(update),
                })
                .collect(),
            }
        })
        .collect()
}

pub fn create_initial_slot_updates(matrix: Option<&Matrix>) -> Vec<QualifiedOccasionalSlotUpdate> {
    let Some(matrix) = matrix else {
        return vec![];
    };
    matrix
        .slots()
        .map(|slot| {
            let play_state = slot.value().play_state();
            let address = SlotAddress {
                column_index: slot.column_index() as u32,
                row_index: slot.value().index() as u32,
            };
            QualifiedOccasionalSlotUpdate {
                slot_address: Some(address),
                update: Some(qualified_occasional_slot_update::Update::play_state(
                    play_state,
                )),
            }
        })
        .collect()
}

pub fn create_initial_clip_updates(matrix: Option<&Matrix>) -> Vec<QualifiedOccasionalClipUpdate> {
    let Some(matrix) = matrix else {
        return vec![];
    };
    matrix
        .clips()
        .map(|item| QualifiedOccasionalClipUpdate {
            clip_address: Some(ClipAddress::from_engine(item.address)),
            update: Some(qualified_occasional_clip_update::Update::content_info(
                item.clip,
            )),
        })
        .collect()
}

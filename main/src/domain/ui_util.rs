use crate::domain::{
    format_as_pretty_hex, FeedbackReason, MatchOutcome, OwnedIncomingMidiMessage, Tag, UnitId,
};
use derive_more::Display;
use helgoboss_learn::{
    format_percentage_without_unit, parse_percentage_without_unit, MidiSourceValue, UnitValue,
};
use helgoboss_midi::{RawShortMessage, ShortMessage};
use itertools::Itertools;
use reaper_high::{Reaper, SliderVolume};
use reaper_medium::Db;
use rosc::{OscMessage, OscPacket};
use std::convert::TryInto;
use std::fmt::Display;

pub fn format_as_percentage_without_unit(value: UnitValue) -> String {
    format_percentage_without_unit(value.get())
}

pub fn format_as_symmetric_percentage_without_unit(value: UnitValue) -> String {
    let symmetric_unit_value = value.get() * 2.0 - 1.0;
    format_percentage_without_unit(symmetric_unit_value)
}

pub fn format_as_double_percentage_without_unit(value: UnitValue) -> String {
    let double_unit_value = value.get() * 2.0;
    format_percentage_without_unit(double_unit_value)
}

pub fn parse_unit_value_from_percentage(text: &str) -> Result<UnitValue, &'static str> {
    parse_percentage_without_unit(text)?.try_into()
}

pub fn parse_from_symmetric_percentage(text: &str) -> Result<UnitValue, &'static str> {
    let percentage: f64 = text.parse().map_err(|_| "not a valid decimal value")?;
    let symmetric_unit_value = percentage / 100.0;
    ((symmetric_unit_value + 1.0) / 2.0).try_into()
}

pub fn parse_from_double_percentage(text: &str) -> Result<UnitValue, &'static str> {
    let percentage: f64 = text.parse().map_err(|_| "not a valid decimal value")?;
    let doble_unit_value = percentage / 100.0;
    (doble_unit_value / 2.0).try_into()
}

/// Parses the given string as a dB value up to 12 dB.
pub fn parse_value_from_db(text: &str) -> Result<UnitValue, &'static str> {
    let decimal: f64 = text.parse().map_err(|_| "not a decimal value")?;
    let db: Db = decimal.try_into().map_err(|_| "not in dB range")?;
    SliderVolume::from_db(db)
        .normalized_slider_value()
        .try_into()
}

pub fn format_value_as_db_without_unit(value: UnitValue) -> String {
    let volume =
        SliderVolume::try_from_normalized_slider_value(value.get()).unwrap_or(SliderVolume::MIN);
    format_volume_as_db_without_unit(volume)
}

pub fn format_volume_as_db_without_unit(volume: SliderVolume) -> String {
    let db = volume.db();
    if db == Db::MINUS_INF {
        "-inf".to_string()
    } else {
        format!("{:.4}", db.get())
    }
}

#[allow(unused)]
pub fn db_unit_value(volume: Db) -> UnitValue {
    volume_unit_value(SliderVolume::from_db(volume))
}

/// Returns the given volume as unit value, clamping to 1.0 if the volume is higher than 12 dB.
pub fn volume_unit_value(volume: SliderVolume) -> UnitValue {
    // The soft-normalized value can be > 1.0, e.g. when we have a volume of 12 dB and then
    // lower the volume fader limit to a lower value. In that case we just report the
    // highest possible value ... not much else we can do.
    UnitValue::new_clamped(volume.normalized_slider_value())
}

pub fn convert_bool_to_unit_value(on: bool) -> UnitValue {
    if on {
        UnitValue::MAX
    } else {
        UnitValue::MIN
    }
}

pub fn format_value_as_db(value: UnitValue) -> String {
    SliderVolume::try_from_normalized_slider_value(value.get())
        .unwrap_or(SliderVolume::MIN)
        .to_string()
}

pub fn format_control_input_with_match_result(
    msg: impl Display,
    match_result: MatchOutcome,
) -> String {
    format!("{msg} ({match_result})")
}

pub fn log_virtual_control_input(unit_id: UnitId, msg: impl Display) {
    log(unit_id, "Virtual control", msg);
}

pub fn log_real_control_input(unit_id: UnitId, msg: impl Display) {
    log(unit_id, "Real control", msg);
}

pub fn log_real_learn_input(unit_id: UnitId, msg: impl Display) {
    log(unit_id, "Real learn", msg);
}

pub fn log_target_control(unit_id: UnitId, msg: impl Display) {
    log(unit_id, "Target control", msg);
}

pub fn log_virtual_feedback_output(unit_id: UnitId, msg: impl Display) {
    log_output(unit_id, OutputReason::VirtualFeedback, msg);
}

pub fn log_real_feedback_output(
    unit_id: UnitId,
    feedback_reason: FeedbackReason,
    msg: impl Display,
) {
    log_output(
        unit_id,
        OutputReason::RealFeedback,
        format!("{msg} ({feedback_reason:?})"),
    );
}

pub fn log_lifecycle_output(unit_id: UnitId, msg: impl Display) {
    log_output(unit_id, OutputReason::Lifecycle, msg);
}

pub fn log_target_output(unit_id: UnitId, msg: impl Display) {
    log_output(unit_id, OutputReason::TargetOutput, msg);
}

pub fn log_output(unit_id: UnitId, reason: OutputReason, msg: impl Display) {
    log(unit_id, reason, msg);
}

#[derive(Copy, Clone, Debug, Display)]
pub enum OutputReason {
    #[display(fmt = "Real feedback")]
    RealFeedback,
    #[display(fmt = "Virtual feedback")]
    VirtualFeedback,
    #[display(fmt = "Lifecycle output")]
    Lifecycle,
    #[display(fmt = "Target output")]
    TargetOutput,
}

/// Used for logging at the moment.
pub fn format_midi_source_value(value: &MidiSourceValue<RawShortMessage>) -> String {
    use MidiSourceValue::*;
    match value {
        Plain(m) => format_short_midi_message(*m),
        ParameterNumber(m) => serde_json::to_string(&m).unwrap(),
        ControlChange14Bit(m) => serde_json::to_string(&m).unwrap(),
        Tempo(bpm) => format!("{bpm:?}"),
        Raw { events, .. } => {
            let event_strings: Vec<_> = events
                .iter()
                .map(|event| format_as_pretty_hex(event.bytes()))
                .collect();
            event_strings.join(", ")
        }
        BorrowedSysEx(bytes) => format_as_pretty_hex(bytes),
    }
}

pub fn format_osc_packet(packet: &OscPacket) -> String {
    format!("{packet:?}")
}

pub fn format_osc_message(msg: &OscMessage) -> String {
    format!("{msg:?}")
}

fn format_short_midi_message(msg: RawShortMessage) -> String {
    let bytes = msg.to_bytes();
    let decimal = format!("[{}, {}, {}]", bytes.0, bytes.1, bytes.2);
    let structured = format!("{:?}", msg.to_structured());
    let hex = format!(
        "[{:02X}, {:02X}, {:02X}]",
        bytes.0,
        bytes.1.get(),
        bytes.2.get()
    );
    format!("{hex} = {decimal} = {structured}")
}

pub fn format_incoming_midi_message(msg: OwnedIncomingMidiMessage) -> String {
    use OwnedIncomingMidiMessage::*;
    match msg {
        Short(m) => format_short_midi_message(m),
        SysEx(m) => format_as_pretty_hex(&m),
    }
}

fn log(unit_id: UnitId, label: impl Display, msg: impl Display) {
    let reaper = Reaper::get();
    reaper.show_console_msg(format!(
        "{:.3} | ReaLearn {} | {:<16} | {}\n",
        reaper.medium_reaper().low().time_precise(),
        unit_id,
        label,
        msg
    ));
}

pub fn format_tags_as_csv<'a>(tags: impl IntoIterator<Item = &'a Tag>) -> String {
    format_as_csv(tags)
}

fn format_as_csv(iter: impl IntoIterator<Item = impl Display>) -> String {
    iter.into_iter().join(", ")
}

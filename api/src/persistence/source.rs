use crate::persistence::{OscArgument, VirtualControlElementCharacter, VirtualControlElementId};
use derive_more::Display;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(PartialEq, Default, Serialize, Deserialize)]
#[serde(tag = "kind")]
#[allow(clippy::enum_variant_names)]
pub enum Source {
    // None
    #[default]
    None,
    // REAPER
    MidiDeviceChanges,
    RealearnInstanceStart,
    Timer(TimerSource),
    RealearnParameter(RealearnParameterSource),
    Speech,
    // MIDI
    MidiNoteVelocity(MidiNoteVelocitySource),
    MidiNoteKeyNumber(MidiNoteKeyNumberSource),
    MidiPolyphonicKeyPressureAmount(MidiPolyphonicKeyPressureAmountSource),
    MidiControlChangeValue(MidiControlChangeValueSource),
    MidiProgramChangeNumber(MidiProgramChangeNumberSource),
    MidiSpecificProgramChange(MidiSpecificProgramChangeSource),
    MidiChannelPressureAmount(MidiChannelPressureAmountSource),
    MidiPitchBendChangeValue(MidiPitchBendChangeValueSource),
    MidiParameterNumberValue(MidiParameterNumberValueSource),
    MidiClockTempo,
    MidiClockTransport(MidiClockTransportSource),
    MidiRaw(MidiRawSource),
    MidiScript(MidiScriptSource),
    MackieLcd(MackieLcdSource),
    XTouchMackieLcd(XTouchMackieLcdSource),
    MackieSevenSegmentDisplay(MackieSevenSegmentDisplaySource),
    SlKeyboardDisplay(SlKeyboardDisplaySource),
    SiniConE24Display(SiniConE24DisplaySource),
    LaunchpadProScrollingTextDisplay,
    // OSC
    Osc(OscSource),
    // Keyboard
    Key(KeySource),
    // Virtual
    Virtual(VirtualSource),
}

// Only makes sense for sources that support both control *and* feedback.
#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum FeedbackBehavior {
    Normal,
    SendFeedbackAfterControl,
    PreventEchoFeedback,
}

impl Default for FeedbackBehavior {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct MidiNoteVelocitySource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_behavior: Option<FeedbackBehavior>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_number: Option<u8>,
}

#[derive(Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct MidiNoteKeyNumberSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_behavior: Option<FeedbackBehavior>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<u8>,
}

#[derive(Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct MidiPolyphonicKeyPressureAmountSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_behavior: Option<FeedbackBehavior>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_number: Option<u8>,
}

#[derive(Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct MidiControlChangeValueSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_behavior: Option<FeedbackBehavior>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub controller_number: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character: Option<SourceCharacter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fourteen_bit: Option<bool>,
}

#[derive(Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct MidiProgramChangeNumberSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_behavior: Option<FeedbackBehavior>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<u8>,
}

#[derive(Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct MidiSpecificProgramChangeSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_behavior: Option<FeedbackBehavior>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub program_number: Option<u8>,
}

#[derive(Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct MidiChannelPressureAmountSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_behavior: Option<FeedbackBehavior>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<u8>,
}

#[derive(Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct MidiPitchBendChangeValueSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_behavior: Option<FeedbackBehavior>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<u8>,
}

#[derive(Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct MidiParameterNumberValueSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_behavior: Option<FeedbackBehavior>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fourteen_bit: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registered: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character: Option<SourceCharacter>,
}

#[derive(Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct MidiClockTransportSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<MidiClockTransportMessage>,
}

#[derive(Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct MidiRawSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_behavior: Option<FeedbackBehavior>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character: Option<SourceCharacter>,
}

#[derive(Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct MidiScriptSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "kind")]
    pub script_kind: Option<MidiScriptKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
}

/// Kind of a MIDI script
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Default,
    Serialize,
    Deserialize,
    EnumIter,
    TryFromPrimitive,
    IntoPrimitive,
    Display,
)]
#[repr(usize)]
pub enum MidiScriptKind {
    #[default]
    #[serde(alias = "eel")]
    #[display(fmt = "EEL")]
    Eel,
    #[serde(alias = "lua")]
    #[display(fmt = "Lua")]
    Lua,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum SourceCharacter {
    #[default]
    Range,
    Button,
    // 127 = decrement;  0 = none;  1 = increment
    Relative1,
    //  63 = decrement; 64 = none; 65 = increment
    Relative2,
    //  65 = decrement;  0 = none;  1 = increment
    Relative3,
    StatefulButton,
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
pub enum MidiClockTransportMessage {
    #[default]
    Start,
    Continue,
    Stop,
}

#[derive(Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct MackieLcdSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extender_index: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u8>,
}

#[derive(Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct XTouchMackieLcdSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extender_index: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u8>,
}

#[derive(Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct SlKeyboardDisplaySource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u8>,
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct MackieSevenSegmentDisplaySource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<MackieSevenSegmentDisplayScope>,
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
pub enum MackieSevenSegmentDisplayScope {
    All,
    #[default]
    Assignment,
    Tc,
    TcHoursBars,
    TcMinutesBeats,
    TcSecondsSub,
    TcFramesTicks,
}

#[derive(Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct SiniConE24DisplaySource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell_index: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_index: Option<u8>,
}

#[derive(Default, PartialEq, Serialize, Deserialize)]
pub struct OscSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_behavior: Option<FeedbackBehavior>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub argument: Option<OscArgument>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relative: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_arguments: Option<Vec<String>>,
}

#[derive(Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct RealearnParameterSource {
    pub parameter_index: u32,
}

#[derive(Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct TimerSource {
    pub duration: u64,
}

#[derive(Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct KeySource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keystroke: Option<Keystroke>,
}

#[derive(Copy, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Keystroke {
    pub modifiers: u8,
    pub key: u16,
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct VirtualSource {
    pub id: VirtualControlElementId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character: Option<VirtualControlElementCharacter>,
}

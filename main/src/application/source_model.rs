use crate::application::{
    Affected, Change, GetProcessingRelevance, MappingProp, ProcessingRelevance,
};
use crate::base::CloneAsDefault;
use crate::domain::{
    Backbone, CompartmentKind, CompartmentParamIndex, CompoundMappingSource, EelMidiSourceScript,
    ExtendedSourceCharacter, FlexibleMidiSourceScript, KeySource, Keystroke, LuaMidiSourceScript,
    MidiSource, RealearnParameterSource, ReaperSource, SpeechSource, TimerSource,
    VirtualControlElement, VirtualControlElementId, VirtualSource,
};
use derive_more::Display;
use helgoboss_learn::{
    ControlValue, DetailedSourceCharacter, DisplaySpec, DisplayType, Interval, MackieLcdScope,
    MackieSevenSegmentDisplayScope, MidiClockTransportMessage, OscArgDescriptor, OscSource,
    OscTypeTag, SiniConE24Scope, SlKeyboardDisplayScope, SourceCharacter, UnitValue,
    DEFAULT_OSC_ARG_VALUE_RANGE,
};
use helgoboss_midi::{Channel, U14, U7};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use realearn_api::persistence::{MidiScriptKind, VirtualControlElementCharacter};
use serde::{Deserialize, Serialize};
use serde_repr::*;
use std::borrow::Cow;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::Display;
use std::time::Duration;
use strum::EnumIter;

#[allow(clippy::enum_variant_names)]
pub enum SourceCommand {
    SetCategory(SourceCategory),
    SetMidiSourceType(MidiSourceType),
    SetChannel(Option<Channel>),
    SetMidiMessageNumber(Option<U7>),
    SetParameterNumberMessageNumber(Option<U14>),
    SetCustomCharacter(SourceCharacter),
    SetMidiClockTransportMessage(MidiClockTransportMessage),
    SetIsRegistered(Option<bool>),
    SetIs14Bit(Option<bool>),
    SetRawMidiPattern(String),
    SetMidiScriptKind(MidiScriptKind),
    SetMidiScript(String),
    SetDisplayType(DisplayType),
    SetDisplayId(Option<u8>),
    SetLine(Option<u8>),
    SetOscAddressPattern(String),
    SetOscArgIndex(Option<u32>),
    SetOscArgTypeTag(OscTypeTag),
    SetOscArgIsRelative(bool),
    SetOscArgValueRange(Interval<f64>),
    SetOscFeedbackArgs(Vec<String>),
    SetReaperSourceType(ReaperSourceType),
    SetTimerMillis(u64),
    SetParameterIndex(CompartmentParamIndex),
    SetKeystroke(Option<Keystroke>),
    SetControlElementCharacter(VirtualControlElementCharacter),
    SetControlElementId(VirtualControlElementId),
}

#[derive(Eq, PartialEq)]
pub enum SourceProp {
    Category,
    MidiSourceType,
    Channel,
    MidiMessageNumber,
    ParameterNumberMessageNumber,
    CustomCharacter,
    MidiClockTransportMessage,
    IsRegistered,
    Is14Bit,
    RawMidiPattern,
    MidiScriptKind,
    MidiScript,
    DisplayType,
    DisplayId,
    Line,
    OscAddressPattern,
    OscArgIndex,
    OscArgTypeTag,
    OscArgIsRelative,
    OscArgValueRange,
    OscFeedbackArgs,
    ReaperSourceType,
    ControlElementType,
    ControlElementId,
    TimerMillis,
    ParameterIndex,
    Keystroke,
}

impl GetProcessingRelevance for SourceProp {
    fn processing_relevance(&self) -> Option<ProcessingRelevance> {
        // At the moment, all source aspects are relevant for processing.
        Some(ProcessingRelevance::ProcessingRelevant)
    }
}

impl<'a> Change<'a> for SourceModel {
    type Command = SourceCommand;
    type Prop = SourceProp;

    fn change(&mut self, cmd: Self::Command) -> Option<Affected<SourceProp>> {
        use Affected::*;
        use SourceCommand as C;
        use SourceProp as P;
        let affected = match cmd {
            C::SetCategory(v) => {
                self.category = v;
                One(P::Category)
            }
            C::SetMidiSourceType(v) => {
                self.midi_source_type = v;
                One(P::MidiSourceType)
            }
            C::SetChannel(v) => {
                self.channel = v;
                One(P::Channel)
            }
            C::SetMidiMessageNumber(v) => {
                self.midi_message_number = v;
                One(P::MidiMessageNumber)
            }
            C::SetParameterNumberMessageNumber(v) => {
                self.parameter_number_message_number = v;
                One(P::ParameterNumberMessageNumber)
            }
            C::SetCustomCharacter(v) => {
                self.custom_character = v;
                One(P::CustomCharacter)
            }
            C::SetMidiClockTransportMessage(v) => {
                self.midi_clock_transport_message = v;
                One(P::MidiClockTransportMessage)
            }
            C::SetIsRegistered(v) => {
                self.is_registered = v;
                One(P::IsRegistered)
            }
            C::SetIs14Bit(v) => {
                self.is_14_bit = v;
                One(P::Is14Bit)
            }
            C::SetRawMidiPattern(v) => {
                self.raw_midi_pattern = v;
                One(P::RawMidiPattern)
            }
            C::SetMidiScriptKind(v) => {
                self.midi_script_kind = v;
                One(P::MidiScriptKind)
            }
            C::SetMidiScript(v) => {
                self.midi_script = v;
                One(P::MidiScript)
            }
            C::SetDisplayType(v) => {
                self.display_type = v;
                One(P::DisplayType)
            }
            C::SetDisplayId(v) => {
                self.display_id = v;
                One(P::DisplayId)
            }
            C::SetLine(v) => {
                self.line = v;
                One(P::Line)
            }
            C::SetOscAddressPattern(v) => {
                self.osc_address_pattern = v;
                One(P::OscAddressPattern)
            }
            C::SetOscArgIndex(v) => {
                self.osc_arg_index = v;
                One(P::OscArgIndex)
            }
            C::SetOscArgTypeTag(v) => {
                self.osc_arg_type_tag = v;
                One(P::OscArgTypeTag)
            }
            C::SetOscArgIsRelative(v) => {
                self.osc_arg_is_relative = v;
                One(P::OscArgIsRelative)
            }
            C::SetOscArgValueRange(v) => {
                self.osc_arg_value_range = v;
                One(P::OscArgValueRange)
            }
            C::SetOscFeedbackArgs(v) => {
                self.osc_feedback_args = v;
                One(P::OscFeedbackArgs)
            }
            C::SetReaperSourceType(v) => {
                self.reaper_source_type = v;
                One(P::ReaperSourceType)
            }
            C::SetControlElementCharacter(v) => {
                self.control_element_character = v;
                One(P::ControlElementType)
            }
            C::SetControlElementId(v) => {
                self.control_element_id = v;
                One(P::ControlElementId)
            }
            C::SetTimerMillis(v) => {
                self.timer_millis = v;
                One(P::TimerMillis)
            }
            C::SetParameterIndex(v) => {
                self.parameter_index = v;
                One(P::ParameterIndex)
            }
            C::SetKeystroke(v) => {
                self.keystroke = v;
                One(P::Keystroke)
            }
        };
        Some(affected)
    }
}

/// A model for creating sources
#[derive(Clone, Debug)]
pub struct SourceModel {
    category: SourceCategory,
    custom_character: SourceCharacter,
    // MIDI
    midi_source_type: MidiSourceType,
    channel: Option<Channel>,
    midi_message_number: Option<U7>,
    parameter_number_message_number: Option<U14>,
    midi_clock_transport_message: MidiClockTransportMessage,
    is_registered: Option<bool>,
    is_14_bit: Option<bool>,
    raw_midi_pattern: String,
    midi_script_kind: MidiScriptKind,
    midi_script: String,
    display_type: DisplayType,
    display_id: Option<u8>,
    line: Option<u8>,
    // OSC
    osc_address_pattern: String,
    osc_arg_index: Option<u32>,
    osc_arg_type_tag: OscTypeTag,
    osc_arg_is_relative: bool,
    osc_arg_value_range: Interval<f64>,
    osc_feedback_args: Vec<String>,
    // REAPER
    reaper_source_type: ReaperSourceType,
    timer_millis: u64,
    parameter_index: CompartmentParamIndex,
    // Key
    keystroke: Option<Keystroke>,
    // Virtual
    control_element_character: VirtualControlElementCharacter,
    control_element_id: VirtualControlElementId,
}

impl Default for SourceModel {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceModel {
    pub fn new() -> Self {
        Self {
            category: SourceCategory::Never,
            midi_source_type: Default::default(),
            control_element_character: Default::default(),
            control_element_id: Default::default(),
            channel: None,
            midi_message_number: None,
            parameter_number_message_number: None,
            custom_character: Default::default(),
            midi_clock_transport_message: Default::default(),
            is_registered: Some(false),
            is_14_bit: Some(false),
            raw_midi_pattern: "".to_owned(),
            midi_script_kind: Default::default(),
            midi_script: "".to_owned(),
            display_type: Default::default(),
            display_id: Default::default(),
            line: None,
            osc_address_pattern: "".to_owned(),
            osc_arg_index: Some(0),
            osc_arg_type_tag: Default::default(),
            osc_arg_is_relative: false,
            osc_arg_value_range: DEFAULT_OSC_ARG_VALUE_RANGE,
            osc_feedback_args: vec![],
            reaper_source_type: Default::default(),
            timer_millis: Default::default(),
            parameter_index: Default::default(),
            keystroke: None,
        }
    }

    pub fn category(&self) -> SourceCategory {
        self.category
    }

    pub fn midi_source_type(&self) -> MidiSourceType {
        self.midi_source_type
    }

    pub fn channel(&self) -> Option<Channel> {
        self.channel
    }

    pub fn midi_message_number(&self) -> Option<U7> {
        self.midi_message_number
    }

    pub fn parameter_number_message_number(&self) -> Option<U14> {
        self.parameter_number_message_number
    }

    pub fn custom_character(&self) -> SourceCharacter {
        self.custom_character
    }

    pub fn midi_clock_transport_message(&self) -> MidiClockTransportMessage {
        self.midi_clock_transport_message
    }

    pub fn is_registered(&self) -> Option<bool> {
        self.is_registered
    }

    pub fn is_14_bit(&self) -> Option<bool> {
        self.is_14_bit
    }

    pub fn raw_midi_pattern(&self) -> &str {
        &self.raw_midi_pattern
    }

    pub fn midi_script_kind(&self) -> MidiScriptKind {
        self.midi_script_kind
    }

    pub fn midi_script(&self) -> &str {
        &self.midi_script
    }

    pub fn display_type(&self) -> DisplayType {
        self.display_type
    }

    pub fn display_id(&self) -> Option<u8> {
        self.display_id
    }

    pub fn line(&self) -> Option<u8> {
        self.line
    }

    pub fn osc_address_pattern(&self) -> &str {
        &self.osc_address_pattern
    }

    pub fn osc_arg_index(&self) -> Option<u32> {
        self.osc_arg_index
    }

    pub fn osc_arg_type_tag(&self) -> OscTypeTag {
        self.osc_arg_type_tag
    }

    pub fn osc_arg_is_relative(&self) -> bool {
        self.osc_arg_is_relative
    }

    pub fn osc_arg_value_range(&self) -> Interval<f64> {
        self.osc_arg_value_range
    }

    pub fn osc_feedback_args(&self) -> &[String] {
        &self.osc_feedback_args
    }

    pub fn keystroke(&self) -> Option<Keystroke> {
        self.keystroke
    }

    pub fn reaper_source_type(&self) -> ReaperSourceType {
        self.reaper_source_type
    }

    pub fn parameter_index(&self) -> CompartmentParamIndex {
        self.parameter_index
    }

    pub fn timer_millis(&self) -> u64 {
        self.timer_millis
    }

    pub fn control_element_character(&self) -> VirtualControlElementCharacter {
        self.control_element_character
    }

    pub fn control_element_id(&self) -> VirtualControlElementId {
        self.control_element_id
    }

    pub fn supports_control(&self) -> bool {
        use SourceCategory::*;
        match self.category {
            Midi => self.midi_source_type.supports_control(),
            Osc => self.osc_arg_type_tag.supports_control(),
            Reaper => self.reaper_source_type.supports_control(),
            Virtual | Keyboard => true,
            // Main use case: Group interaction (follow-only).
            Never => true,
        }
    }

    pub fn supports_feedback(&self) -> bool {
        use SourceCategory::*;
        match self.category {
            Midi => self.midi_source_type.supports_feedback(),
            Osc => self.osc_arg_type_tag.supports_feedback(),
            Reaper => self.reaper_source_type.supports_feedback(),
            Virtual => true,
            Keyboard | Never => false,
        }
    }

    #[must_use]
    pub fn apply_from_source(
        &mut self,
        source: &CompoundMappingSource,
    ) -> Option<Affected<MappingProp>> {
        use CompoundMappingSource::*;
        match source {
            Midi(s) => {
                self.category = SourceCategory::Midi;
                self.midi_source_type = MidiSourceType::from_source(s);
                self.channel = s.channel();
                use helgoboss_learn::MidiSource::*;
                match s {
                    NoteVelocity { key_number, .. }
                    | PolyphonicKeyPressureAmount { key_number, .. } => {
                        self.midi_message_number = key_number.map(Into::into);
                    }
                    SpecificProgramChange { program_number, .. } => {
                        self.midi_message_number = *program_number;
                    }
                    ControlChangeValue {
                        controller_number,
                        custom_character,
                        ..
                    } => {
                        self.is_14_bit = Some(false);
                        self.midi_message_number = controller_number.map(Into::into);
                        self.custom_character = *custom_character;
                    }
                    ControlChange14BitValue {
                        msb_controller_number,
                        ..
                    } => {
                        self.is_14_bit = Some(true);
                        self.midi_message_number = msb_controller_number.map(Into::into);
                    }
                    ParameterNumberValue {
                        number,
                        is_14_bit,
                        is_registered,
                        custom_character,
                        ..
                    } => {
                        self.parameter_number_message_number = *number;
                        self.is_14_bit = *is_14_bit;
                        self.is_registered = *is_registered;
                        self.custom_character = *custom_character;
                    }
                    ClockTransport { message } => {
                        self.midi_clock_transport_message = *message;
                    }
                    Raw {
                        pattern,
                        custom_character,
                    } => {
                        self.custom_character = *custom_character;
                        self.raw_midi_pattern = pattern.to_string();
                    }
                    _ => {}
                }
            }
            Virtual(s) => {
                self.category = SourceCategory::Virtual;
                self.control_element_character = s.control_element().character();
                self.control_element_id = s.control_element().id();
            }
            Osc(s) => {
                self.category = SourceCategory::Osc;
                self.osc_address_pattern = s.address_pattern().to_owned();
                self.osc_arg_index = s.arg_descriptor().map(|d| d.index());
                self.osc_arg_type_tag =
                    s.arg_descriptor().map(|d| d.type_tag()).unwrap_or_default();
                self.osc_arg_is_relative = s
                    .arg_descriptor()
                    .map(|d| d.is_relative())
                    .unwrap_or_default();
            }
            Reaper(s) => {
                self.category = SourceCategory::Reaper;
                self.reaper_source_type = ReaperSourceType::from_source(s);
                use ReaperSource::*;
                match s {
                    RealearnParameter(p) => {
                        self.parameter_index = p.parameter_index;
                    }
                    MidiDeviceChanges | RealearnInstanceStart | Timer(_) | Speech(_) => {}
                }
            }
            Never => {
                self.category = SourceCategory::Never;
            }
            Key(s) => {
                self.category = SourceCategory::Keyboard;
                self.keystroke = Some(s.stroke());
            }
        };
        Some(Affected::Multiple)
    }

    pub fn format_control_value(&self, value: ControlValue) -> Result<String, &'static str> {
        self.create_source().format_control_value(value)
    }

    pub fn parse_control_value(&self, text: &str) -> Result<UnitValue, &'static str> {
        self.create_source().parse_control_value(text)
    }

    pub fn character(&self) -> ExtendedSourceCharacter {
        self.create_source().character()
    }

    pub fn possible_detailed_characters(&self) -> Vec<DetailedSourceCharacter> {
        match self.create_source() {
            CompoundMappingSource::Midi(s) => s.possible_detailed_characters(),
            CompoundMappingSource::Osc(s) => s.possible_detailed_characters(),
            CompoundMappingSource::Virtual(s) => match s.control_element().character() {
                VirtualControlElementCharacter::Multi => vec![
                    DetailedSourceCharacter::MomentaryVelocitySensitiveButton,
                    DetailedSourceCharacter::MomentaryOnOffButton,
                    DetailedSourceCharacter::Trigger,
                    DetailedSourceCharacter::RangeControl,
                    DetailedSourceCharacter::Relative,
                ],
                VirtualControlElementCharacter::Button => vec![
                    DetailedSourceCharacter::MomentaryOnOffButton,
                    DetailedSourceCharacter::Trigger,
                ],
            },
            CompoundMappingSource::Reaper(s) => s.possible_detailed_characters(),
            // Can be anything, depending on the mapping that uses the group interaction.
            CompoundMappingSource::Never => vec![
                DetailedSourceCharacter::MomentaryVelocitySensitiveButton,
                DetailedSourceCharacter::MomentaryOnOffButton,
                DetailedSourceCharacter::Trigger,
                DetailedSourceCharacter::RangeControl,
                DetailedSourceCharacter::Relative,
            ],
            CompoundMappingSource::Key(_) => vec![DetailedSourceCharacter::MomentaryOnOffButton],
        }
    }

    /// Creates a source reflecting this model's current values
    pub fn create_source(&self) -> CompoundMappingSource {
        self.create_source_internal()
            .unwrap_or(CompoundMappingSource::Never)
    }

    fn create_source_internal(&self) -> Option<CompoundMappingSource> {
        use SourceCategory::*;
        let source = match self.category {
            Midi => {
                use MidiSourceType::*;
                let channel = self.channel;
                let key_number = self.midi_message_number.map(|n| n.into());
                let midi_source = match self.midi_source_type {
                    NoteVelocity => MidiSource::NoteVelocity {
                        channel,
                        key_number,
                    },
                    NoteKeyNumber => MidiSource::NoteKeyNumber { channel },
                    PolyphonicKeyPressureAmount => MidiSource::PolyphonicKeyPressureAmount {
                        channel,
                        key_number,
                    },
                    ControlChangeValue => {
                        if self.is_14_bit == Some(true) {
                            MidiSource::ControlChange14BitValue {
                                channel,
                                msb_controller_number: self.midi_message_number.map(|n| {
                                    // We accept even non-MSB numbers and convert them into them.
                                    // https://github.com/helgoboss/realearn/issues/30
                                    let msb_controller_number = U7::new(n.get() % 32);
                                    msb_controller_number.into()
                                }),
                                custom_character: self.custom_character,
                            }
                        } else {
                            MidiSource::ControlChangeValue {
                                channel,
                                controller_number: self.midi_message_number.map(|n| n.into()),
                                custom_character: self.custom_character,
                            }
                        }
                    }
                    ProgramChangeNumber => MidiSource::ProgramChangeNumber { channel },
                    SpecificProgramChange => MidiSource::SpecificProgramChange {
                        channel,
                        program_number: self.midi_message_number,
                    },
                    ChannelPressureAmount => MidiSource::ChannelPressureAmount { channel },
                    PitchBendChangeValue => MidiSource::PitchBendChangeValue { channel },
                    ParameterNumberValue => MidiSource::ParameterNumberValue {
                        channel,
                        number: self.parameter_number_message_number,
                        is_14_bit: self.is_14_bit,
                        is_registered: self.is_registered,
                        custom_character: self.custom_character,
                    },
                    ClockTempo => MidiSource::ClockTempo,
                    ClockTransport => MidiSource::ClockTransport {
                        message: self.midi_clock_transport_message,
                    },
                    Raw => MidiSource::Raw {
                        pattern: self.raw_midi_pattern.parse().unwrap_or_default(),
                        custom_character: self.custom_character,
                    },
                    Script => MidiSource::Script {
                        script: {
                            let script = match self.midi_script_kind {
                                MidiScriptKind::Eel => FlexibleMidiSourceScript::Eel(
                                    EelMidiSourceScript::compile(&self.midi_script).ok()?,
                                ),
                                MidiScriptKind::Lua => {
                                    let lua = unsafe { Backbone::main_thread_lua() };
                                    FlexibleMidiSourceScript::Lua(
                                        LuaMidiSourceScript::compile(lua, &self.midi_script)
                                            .ok()?,
                                    )
                                }
                            };
                            CloneAsDefault::new(Some(script))
                        },
                    },
                    Display => MidiSource::Display {
                        spec: self.display_spec(),
                    },
                };
                CompoundMappingSource::Midi(midi_source)
            }
            Virtual => {
                let virtual_source = VirtualSource::new(self.create_control_element());
                CompoundMappingSource::Virtual(virtual_source)
            }
            Osc => {
                let osc_source = OscSource::new(
                    self.osc_address_pattern.clone(),
                    self.osc_arg_descriptor(),
                    self.osc_feedback_args
                        .iter()
                        .map(|prop_string| prop_string.parse().unwrap_or_default())
                        .collect(),
                );
                CompoundMappingSource::Osc(osc_source)
            }
            Reaper => {
                use ReaperSourceType::*;
                let reaper_source = match self.reaper_source_type {
                    MidiDeviceChanges => ReaperSource::MidiDeviceChanges,
                    RealearnUnitStart => ReaperSource::RealearnInstanceStart,
                    Timer => ReaperSource::Timer(self.create_timer_source()),
                    RealearnParameter => {
                        ReaperSource::RealearnParameter(self.create_realearn_parameter_source())
                    }
                    Speech => ReaperSource::Speech(SpeechSource::new()),
                };
                CompoundMappingSource::Reaper(reaper_source)
            }
            Never => CompoundMappingSource::Never,
            Keyboard => CompoundMappingSource::Key(self.create_key_source()?),
        };
        Some(source)
    }

    pub fn create_key_source(&self) -> Option<KeySource> {
        Some(KeySource::new(self.keystroke?))
    }

    fn create_timer_source(&self) -> TimerSource {
        TimerSource::new(Duration::from_millis(self.timer_millis))
    }

    fn create_realearn_parameter_source(&self) -> RealearnParameterSource {
        RealearnParameterSource {
            parameter_index: self.parameter_index,
        }
    }

    fn display_spec(&self) -> DisplaySpec {
        use DisplayType::*;
        match self.display_type {
            MackieLcd => DisplaySpec::MackieLcd {
                scope: self.mackie_lcd_scope(),
                extender_index: 0,
            },
            MackieXtLcd => DisplaySpec::MackieLcd {
                scope: self.mackie_lcd_scope(),
                extender_index: 1,
            },
            XTouchMackieLcd => DisplaySpec::XTouchMackieLcd {
                scope: self.mackie_lcd_scope(),
                extender_index: 0,
            },
            XTouchMackieXtLcd => DisplaySpec::XTouchMackieLcd {
                scope: self.mackie_lcd_scope(),
                extender_index: 1,
            },
            MackieSevenSegmentDisplay => DisplaySpec::MackieSevenSegmentDisplay {
                scope: self.mackie_7_segment_display_scope(),
            },
            SlKeyboardDisplay => DisplaySpec::SlKeyboard {
                scope: self.sl_keyboard_display_scope(),
            },
            SiniConE24 => DisplaySpec::SiniConE24 {
                scope: self.sinicon_e24_scope(),
                // TODO-low Not so nice to have runtime state in this descriptor.
                last_sent_background_color: Default::default(),
            },
            LaunchpadProScrollingText => DisplaySpec::LaunchpadProScrollingText,
        }
    }

    pub fn mackie_lcd_scope(&self) -> MackieLcdScope {
        MackieLcdScope::new(self.display_id, self.line)
    }

    pub fn sinicon_e24_scope(&self) -> SiniConE24Scope {
        SiniConE24Scope::new(self.display_id, self.line)
    }

    pub fn sl_keyboard_display_scope(&self) -> SlKeyboardDisplayScope {
        SlKeyboardDisplayScope::new(self.display_id, self.line)
    }

    pub fn mackie_7_segment_display_scope(&self) -> MackieSevenSegmentDisplayScope {
        self.display_id
            .and_then(|id| MackieSevenSegmentDisplayScope::try_from(id as usize).ok())
            .unwrap_or_default()
    }

    pub fn simple_source(&self) -> Option<playtime_api::runtime::SimpleSource> {
        use playtime_api::runtime::*;
        use MidiSourceType::*;
        if self.category == SourceCategory::Never {
            return None;
        }
        let s = match self.midi_source_type {
            NoteVelocity => {
                if let (Some(channel), Some(number)) = (self.channel, self.midi_message_number) {
                    let s = NoteSource {
                        channel: channel.get(),
                        number: number.get(),
                    };
                    SimpleSource::Note(s)
                } else {
                    SimpleSource::MoreComplicated
                }
            }
            _ => SimpleSource::MoreComplicated,
        };
        Some(s)
    }

    fn osc_arg_descriptor(&self) -> Option<OscArgDescriptor> {
        let arg_index = self.osc_arg_index?;
        let arg_desc = OscArgDescriptor::new(
            arg_index,
            self.osc_arg_type_tag,
            self.osc_arg_is_relative,
            self.osc_arg_value_range,
        );
        Some(arg_desc)
    }

    pub fn supports_type(&self) -> bool {
        use SourceCategory::*;
        matches!(self.category, Midi | Virtual | Reaper)
    }

    pub fn supports_channel(&self) -> bool {
        if !self.is_midi() {
            return false;
        }
        self.midi_source_type.supports_channel()
    }

    pub fn supports_osc_arg_value_range(&self) -> bool {
        self.category == SourceCategory::Osc
            && self.osc_arg_index.is_some()
            && self.osc_arg_type_tag.supports_value_range()
    }

    pub fn display_count(&self) -> u8 {
        self.display_type.display_count()
    }

    fn is_midi(&self) -> bool {
        self.category == SourceCategory::Midi
    }

    fn channel_label(&self) -> Cow<str> {
        if self.supports_channel() {
            match self.channel {
                None => "Any channel".into(),
                Some(ch) => format!("Channel {}", ch.get() + 1).into(),
            }
        } else {
            "".into()
        }
    }

    fn note_label(&self) -> Cow<str> {
        match self.midi_message_number {
            None => "Any note".into(),
            Some(n) => format!("Note number {}", n.get()).into(),
        }
    }

    fn program_label(&self) -> Cow<str> {
        match self.midi_message_number {
            None => "Any program".into(),
            Some(n) => format!("Program {}", n.get()).into(),
        }
    }

    pub fn create_control_element(&self) -> VirtualControlElement {
        VirtualControlElement::new(self.control_element_id, self.control_element_character)
    }
}

impl Display for SourceModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use SourceCategory::*;
        let lines: Vec<Cow<str>> = match self.category {
            Midi => match self.midi_source_type {
                t @ MidiSourceType::NoteVelocity => {
                    vec![
                        t.to_string().into(),
                        self.channel_label(),
                        self.note_label(),
                    ]
                }
                MidiSourceType::ParameterNumberValue => {
                    let line_1 = match self.is_registered {
                        None => MidiSourceType::ParameterNumberValue.to_string().into(),
                        Some(is_registered) => {
                            if is_registered {
                                "RPN".into()
                            } else {
                                "NRPN".into()
                            }
                        }
                    };
                    let line_3 = match self.parameter_number_message_number {
                        None => "Any number".into(),
                        Some(n) => format!("Number {}", n.get()).into(),
                    };
                    vec![line_1, self.channel_label(), line_3]
                }
                MidiSourceType::PolyphonicKeyPressureAmount => {
                    vec![
                        "Poly after touch".into(),
                        self.channel_label(),
                        self.note_label(),
                    ]
                }
                t @ MidiSourceType::SpecificProgramChange => {
                    vec![
                        t.to_string().into(),
                        self.channel_label(),
                        self.program_label(),
                    ]
                }
                MidiSourceType::ClockTempo => vec!["MIDI clock".into(), "Tempo".into()],
                MidiSourceType::ClockTransport => {
                    vec![
                        "MIDI clock".into(),
                        self.midi_clock_transport_message.to_string().into(),
                    ]
                }
                t @ MidiSourceType::ControlChangeValue => {
                    let line_3 = match self.midi_message_number {
                        None => "Any CC".into(),
                        Some(n) => format!("CC number {}", n.get()).into(),
                    };
                    use MidiSourceType::*;
                    let line_4 = match self.midi_source_type {
                        ControlChangeValue if self.is_14_bit == Some(false) => {
                            use SourceCharacter::*;
                            let label = match self.custom_character {
                                RangeElement => "Range element",
                                MomentaryButton => "Momentary button",
                                Encoder1 => "Encoder 1",
                                Encoder2 => "Encoder 2",
                                Encoder3 => "Encoder 3",
                                ToggleButton => "Toggle-only button",
                            };
                            label.into()
                        }
                        _ => "".into(),
                    };
                    vec![t.to_string().into(), self.channel_label(), line_3, line_4]
                }
                t @ MidiSourceType::Display => vec![t.to_string().into()],
                t => vec![t.to_string().into(), self.channel_label()],
            },
            Virtual => vec![
                "Virtual".into(),
                self.create_control_element().to_string().into(),
            ],
            Osc => vec!["OSC".into(), (&self.osc_address_pattern).into()],
            Reaper => {
                let type_label = self.reaper_source_type.to_string().into();
                match self.reaper_source_type {
                    ReaperSourceType::Timer => {
                        vec![type_label, format!("{} ms", self.timer_millis).into()]
                    }
                    ReaperSourceType::RealearnParameter => {
                        vec![
                            type_label,
                            format!("Parameter #{}", self.parameter_index.get() + 1).into(),
                        ]
                    }
                    _ => {
                        vec![type_label]
                    }
                }
            }
            Never => vec!["None".into()],
            Keyboard => {
                let text = self
                    .create_key_source()
                    .map(|s| Cow::Owned(s.to_string()))
                    .unwrap_or_else(|| Cow::Borrowed(KEY_UNDEFINED_LABEL));
                vec![text]
            }
        };
        let non_empty_lines: Vec<_> = lines.into_iter().filter(|l| !l.is_empty()).collect();
        write!(f, "{}", non_empty_lines.join("\n"))
    }
}

pub const KEY_UNDEFINED_LABEL: &str = "<Key undefined>";

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
pub enum SourceCategory {
    #[serde(rename = "never")]
    #[display(fmt = "None")]
    Never,
    #[default]
    #[serde(rename = "midi")]
    #[display(fmt = "MIDI")]
    Midi,
    #[serde(rename = "osc")]
    #[display(fmt = "OSC")]
    Osc,
    #[serde(rename = "keyboard")]
    #[display(fmt = "Keyboard")]
    Keyboard,
    #[serde(rename = "reaper")]
    #[display(fmt = "REAPER")]
    Reaper,
    #[serde(rename = "virtual")]
    #[display(fmt = "Virtual")]
    Virtual,
}

impl SourceCategory {
    pub fn default_for(compartment: CompartmentKind) -> Self {
        use SourceCategory::*;
        match compartment {
            CompartmentKind::Controller => Midi,
            CompartmentKind::Main => Midi,
        }
    }

    pub fn is_allowed_in(self, compartment: CompartmentKind) -> bool {
        use SourceCategory::*;
        match compartment {
            CompartmentKind::Controller => match self {
                Never => true,
                Midi => true,
                Osc => true,
                Reaper => true,
                Keyboard => true,
                Virtual => false,
            },
            CompartmentKind::Main => true,
        }
    }
}

/// Type of a MIDI source
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Default,
    Serialize_repr,
    Deserialize_repr,
    EnumIter,
    TryFromPrimitive,
    IntoPrimitive,
    Display,
)]
#[repr(usize)]
pub enum MidiSourceType {
    #[default]
    #[display(fmt = "CC value")]
    ControlChangeValue = 0,
    #[display(fmt = "Note velocity")]
    NoteVelocity = 1,
    #[display(fmt = "Note number")]
    NoteKeyNumber = 2,
    #[display(fmt = "Pitch wheel")]
    PitchBendChangeValue = 3,
    #[display(fmt = "Channel after touch")]
    ChannelPressureAmount = 4,
    #[display(fmt = "Program change number")]
    ProgramChangeNumber = 5,
    #[display(fmt = "(N)RPN value")]
    ParameterNumberValue = 6,
    #[display(fmt = "Polyphonic after touch")]
    PolyphonicKeyPressureAmount = 7,
    #[display(fmt = "MIDI clock tempo (experimental)")]
    ClockTempo = 8,
    #[display(fmt = "MIDI clock transport")]
    ClockTransport = 9,
    #[display(fmt = "Raw MIDI / SysEx")]
    Raw = 10,
    #[display(fmt = "MIDI script (feedback only)")]
    Script = 11,
    #[display(fmt = "Display (feedback only)")]
    Display = 12,
    #[display(fmt = "Specific program change")]
    SpecificProgramChange = 13,
}

impl MidiSourceType {
    pub fn from_source(source: &MidiSource) -> MidiSourceType {
        use helgoboss_learn::MidiSource::*;
        match source {
            NoteVelocity { .. } => MidiSourceType::NoteVelocity,
            NoteKeyNumber { .. } => MidiSourceType::NoteKeyNumber,
            PolyphonicKeyPressureAmount { .. } => MidiSourceType::PolyphonicKeyPressureAmount,
            ControlChangeValue { .. } => MidiSourceType::ControlChangeValue,
            ProgramChangeNumber { .. } => MidiSourceType::ProgramChangeNumber,
            SpecificProgramChange { .. } => MidiSourceType::SpecificProgramChange,
            ChannelPressureAmount { .. } => MidiSourceType::ChannelPressureAmount,
            PitchBendChangeValue { .. } => MidiSourceType::PitchBendChangeValue,
            ControlChange14BitValue { .. } => MidiSourceType::ControlChangeValue,
            ParameterNumberValue { .. } => MidiSourceType::ParameterNumberValue,
            ClockTempo => MidiSourceType::ClockTempo,
            ClockTransport { .. } => MidiSourceType::ClockTransport,
            Raw { .. } => MidiSourceType::Raw,
            Script { .. } => MidiSourceType::Script,
            Display { .. } => MidiSourceType::Display,
        }
    }

    pub fn number_label(self) -> &'static str {
        use MidiSourceType::*;
        match self {
            ControlChangeValue => "CC number",
            NoteVelocity | PolyphonicKeyPressureAmount => "Note number",
            ParameterNumberValue => "Number",
            SpecificProgramChange => "Program",
            _ => "",
        }
    }

    pub fn supports_channel(self) -> bool {
        use MidiSourceType::*;
        matches!(
            self,
            ChannelPressureAmount
                | ControlChangeValue
                | NoteVelocity
                | PolyphonicKeyPressureAmount
                | NoteKeyNumber
                | ParameterNumberValue
                | PitchBendChangeValue
                | ProgramChangeNumber
                | SpecificProgramChange
        )
    }

    pub fn supports_midi_message_number(self) -> bool {
        use MidiSourceType::*;
        matches!(
            self,
            ControlChangeValue | NoteVelocity | PolyphonicKeyPressureAmount | SpecificProgramChange
        )
    }

    pub fn supports_parameter_number_message_number(self) -> bool {
        self.supports_parameter_number_message_props()
    }

    pub fn supports_14_bit(self) -> bool {
        use MidiSourceType::*;
        matches!(self, ControlChangeValue | ParameterNumberValue)
    }

    pub fn supports_is_registered(self) -> bool {
        self.supports_parameter_number_message_props()
    }

    pub fn supports_custom_character(self) -> bool {
        use MidiSourceType::*;
        matches!(self, ControlChangeValue | ParameterNumberValue | Raw)
    }

    fn supports_parameter_number_message_props(self) -> bool {
        self == MidiSourceType::ParameterNumberValue
    }

    pub fn supports_control(self) -> bool {
        use MidiSourceType::*;
        !matches!(self, Script | Display)
    }

    pub fn supports_feedback(self) -> bool {
        use MidiSourceType::*;
        !matches!(self, ClockTempo | ClockTransport)
    }
}

/// Type of a REAPER source
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
pub enum ReaperSourceType {
    #[default]
    #[serde(rename = "midi-device-changes")]
    #[display(fmt = "MIDI device changes")]
    MidiDeviceChanges,
    #[serde(rename = "realearn-instance-start")]
    #[display(fmt = "ReaLearn unit start")]
    RealearnUnitStart,
    #[serde(rename = "timer")]
    #[display(fmt = "Timer")]
    Timer,
    #[serde(rename = "realearn-parameter")]
    #[display(fmt = "ReaLearn parameter")]
    RealearnParameter,
    #[serde(rename = "speech")]
    #[display(fmt = "Speech (feedback only, no Linux)")]
    Speech,
}

impl ReaperSourceType {
    pub fn from_source(source: &ReaperSource) -> Self {
        use ReaperSource::*;
        match source {
            MidiDeviceChanges => Self::MidiDeviceChanges,
            RealearnInstanceStart => Self::RealearnUnitStart,
            Timer(_) => Self::Timer,
            RealearnParameter(_) => Self::RealearnParameter,
            Speech(_) => Self::Speech,
        }
    }

    pub fn supports_control(self) -> bool {
        use ReaperSourceType::*;
        match self {
            MidiDeviceChanges | RealearnUnitStart | Timer | RealearnParameter => true,
            Speech => false,
        }
    }

    pub fn supports_feedback(self) -> bool {
        use ReaperSourceType::*;
        match self {
            MidiDeviceChanges | RealearnUnitStart | Timer | RealearnParameter => false,
            Speech => true,
        }
    }
}

pub fn parse_osc_feedback_args(text: &str) -> Vec<String> {
    text.split_whitespace().map(|s| s.to_owned()).collect()
}

pub fn format_osc_feedback_args(args: &[String]) -> String {
    itertools::join(args.iter(), " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_source() {
        // Given
        let m = SourceModel::new();
        // When
        let s = m.create_source();
        // Then
        assert_eq!(s, CompoundMappingSource::Never);
    }
}

use derive_more::Display;
use enum_iterator::IntoEnumIterator;
use helgoboss_learn::{
    AbsoluteMode, Interval, MidiClockTransportMessage, MidiSource, RelativeMode, SourceCharacter,
    ToggleMode, Transformation, UnitValue,
};
use helgoboss_midi::{Channel, U14, U7};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use rx_util::{create_local_prop as p, LocalProp, LocalStaticProp, UnitEvent};
use rxrust::prelude::*;
use serde_repr::*;

/// A model for creating modes
#[derive(Clone, Debug)]
pub struct ModeModel {
    // For all modes
    pub r#type: LocalStaticProp<ModeType>,
    pub min_target_value: LocalStaticProp<UnitValue>,
    pub max_target_value: LocalStaticProp<UnitValue>,
    // For absolute and relative mode
    pub source_value_interval: LocalStaticProp<Interval<UnitValue>>,
    pub reverse: LocalStaticProp<bool>,
    // For absolute mode
    pub min_jump: LocalStaticProp<UnitValue>,
    pub max_jump: LocalStaticProp<UnitValue>,
    pub ignore_out_of_range_source_values: LocalStaticProp<bool>,
    pub round_target_value: LocalStaticProp<bool>,
    pub approach_target_value: LocalStaticProp<bool>,
    pub eel_control_transformation: LocalStaticProp<String>,
    pub eel_feedback_transformation: LocalStaticProp<String>,
    // For relative mode
    pub min_step_size: LocalStaticProp<UnitValue>,
    pub max_step_size: LocalStaticProp<UnitValue>,
    pub rotate: LocalStaticProp<bool>,
}

/// Represents a value transformation done via EEL scripting language.
pub struct EelTransformation {}

impl EelTransformation {
    // Compiles the given script and creates an appropriate transformation.
    fn compile(eel_script: &str) -> Option<EelTransformation> {
        todo!()
    }
}

impl Transformation for EelTransformation {
    fn transform(&self, input_value: UnitValue) -> Result<UnitValue, ()> {
        todo!()
    }
}

// Represents a learn mode
pub enum Mode {
    Absolute(AbsoluteMode<EelTransformation>),
    Relative(RelativeMode),
    Toggle(ToggleMode),
}

/// Type of a mode
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Serialize_repr,
    Deserialize_repr,
    IntoEnumIterator,
    TryFromPrimitive,
    IntoPrimitive,
    Display,
)]
#[repr(usize)]
pub enum ModeType {
    Absolute = 0,
    Relative = 1,
    Toggle = 2,
}

impl Default for ModeModel {
    fn default() -> Self {
        Self {
            r#type: p(ModeType::Absolute),
            min_target_value: p(UnitValue::MIN),
            max_target_value: p(UnitValue::MAX),
            source_value_interval: p(Interval::new(UnitValue::MIN, UnitValue::MAX)),
            reverse: p(false),
            min_jump: p(UnitValue::MIN),
            max_jump: p(UnitValue::MAX),
            ignore_out_of_range_source_values: p(false),
            round_target_value: p(false),
            approach_target_value: p(false),
            eel_control_transformation: p(String::new()),
            eel_feedback_transformation: p(String::new()),
            min_step_size: p(UnitValue::new(0.01)),
            max_step_size: p(UnitValue::new(0.01)),
            rotate: p(false),
        }
    }
}

impl ModeModel {
    /// Fires whenever one of the properties of this model has changed
    pub fn changed(&self) -> impl UnitEvent {
        self.r#type
            .changed()
            .merge(self.min_target_value.changed())
            .merge(self.max_target_value.changed())
            .merge(self.source_value_interval.changed())
            .merge(self.reverse.changed())
            .merge(self.min_jump.changed())
            .merge(self.max_jump.changed())
            .merge(self.ignore_out_of_range_source_values.changed())
            .merge(self.round_target_value.changed())
            .merge(self.approach_target_value.changed())
            .merge(self.eel_control_transformation.changed())
            .merge(self.eel_feedback_transformation.changed())
            .merge(self.min_step_size.changed())
            .merge(self.max_step_size.changed())
            .merge(self.rotate.changed())
    }

    /// Creates a mode reflecting this model's current values
    pub fn create_mode(&self) -> Mode {
        use ModeType::*;
        match self.r#type.get() {
            Absolute => Mode::Absolute(AbsoluteMode {
                source_value_interval: self.source_value_interval.get(),
                target_value_interval: Interval::new(
                    self.min_target_value.get(),
                    self.max_target_value.get(),
                ),
                jump_interval: Interval::new(self.min_jump.get(), self.max_jump.get()),
                approach_target_value: self.approach_target_value.get(),
                reverse_target_value: self.reverse.get(),
                round_target_value: self.round_target_value.get(),
                ignore_out_of_range_source_values: self.ignore_out_of_range_source_values.get(),
                control_transformation: EelTransformation::compile(
                    self.eel_control_transformation.get_ref(),
                ),
                feedback_transformation: EelTransformation::compile(
                    self.eel_feedback_transformation.get_ref(),
                ),
            }),
            Relative => Mode::Relative(RelativeMode {
                source_value_interval: self.source_value_interval.get(),
                step_count_interval: todo!("needs to transform step size "),
                step_size_interval: Interval::new(
                    self.min_step_size.get(),
                    self.max_step_size.get(),
                ),
                target_value_interval: Interval::new(
                    self.min_target_value.get(),
                    self.max_target_value.get(),
                ),
                reverse: self.reverse.get(),
                rotate: self.rotate.get(),
            }),
            Toggle => Mode::Toggle(ToggleMode {
                target_value_interval: Interval::new(
                    self.min_target_value.get(),
                    self.max_target_value.get(),
                ),
            }),
        }
    }

    pub fn supports_reverse(&self) -> bool {
        use ModeType::*;
        matches!(self.r#type.get(), Absolute | Relative)
    }

    pub fn supports_ignore_out_of_range_source_values(&self) -> bool {
        self.r#type.get() == ModeType::Absolute
    }

    pub fn supports_jump(&self) -> bool {
        self.r#type.get() == ModeType::Absolute
    }

    pub fn supports_eel_control_transformation(&self) -> bool {
        self.r#type.get() == ModeType::Absolute
    }

    pub fn supports_eel_feedback_transformation(&self) -> bool {
        self.r#type.get() == ModeType::Absolute
    }

    pub fn supports_round_target_value(&self) -> bool {
        self.r#type.get() == ModeType::Absolute
    }

    pub fn supports_approach_target_value(&self) -> bool {
        self.r#type.get() == ModeType::Absolute
    }

    pub fn supports_step_size(&self) -> bool {
        self.r#type.get() == ModeType::Relative
    }

    pub fn supports_rotate_is_enabled(&self) -> bool {
        self.r#type.get() == ModeType::Relative
    }
}

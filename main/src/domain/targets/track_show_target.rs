use crate::domain::ui_util::convert_bool_to_unit_value;
use crate::domain::{
    format_value_as_on_off, get_control_type_and_character_for_track_exclusivity,
    handle_track_exclusivity, ControlContext, RealearnTarget, TargetCharacter, TrackExclusivity,
};
use helgoboss_learn::{ControlType, ControlValue, Target, UnitValue};
use reaper_high::{Project, Track};
use reaper_medium::TrackArea;

#[derive(Clone, Debug, PartialEq)]
pub struct TrackShowTarget {
    pub track: Track,
    pub exclusivity: TrackExclusivity,
    pub area: TrackArea,
    pub poll_for_feedback: bool,
}

impl RealearnTarget for TrackShowTarget {
    fn control_type_and_character(&self) -> (ControlType, TargetCharacter) {
        get_control_type_and_character_for_track_exclusivity(self.exclusivity)
    }

    fn format_value(&self, value: UnitValue) -> String {
        format_value_as_on_off(value).to_string()
    }

    fn control(&self, value: ControlValue, _: ControlContext) -> Result<(), &'static str> {
        if value.as_absolute()?.is_zero() {
            handle_track_exclusivity(&self.track, self.exclusivity, |t| {
                t.set_shown(self.area, true)
            });
            self.track.set_shown(self.area, false);
        } else {
            handle_track_exclusivity(&self.track, self.exclusivity, |t| {
                t.set_shown(self.area, false)
            });
            self.track.set_shown(self.area, true);
        }
        Ok(())
    }

    fn is_available(&self) -> bool {
        self.track.is_available()
    }

    fn project(&self) -> Option<Project> {
        Some(self.track.project())
    }

    fn track(&self) -> Option<&Track> {
        Some(&self.track)
    }

    fn track_exclusivity(&self) -> Option<TrackExclusivity> {
        Some(self.exclusivity)
    }

    fn supports_automatic_feedback(&self) -> bool {
        self.poll_for_feedback
    }
}

impl<'a> Target<'a> for TrackShowTarget {
    type Context = ();

    fn current_value(&self, _: ()) -> Option<UnitValue> {
        let is_shown = self.track.is_shown(self.area);
        Some(convert_bool_to_unit_value(is_shown))
    }

    fn control_type(&self) -> ControlType {
        self.control_type_and_character().0
    }
}

use crate::domain::{
    format_value_as_pan, pan_unit_value, parse_value_from_pan, ControlContext, RealearnTarget,
    TargetCharacter,
};
use helgoboss_learn::{ControlType, ControlValue, Target, UnitValue};
use reaper_high::{ChangeEvent, Pan, Project, Track, TrackRoute};

#[derive(Clone, Debug, PartialEq)]
pub struct RoutePanTarget {
    pub route: TrackRoute,
}

impl RealearnTarget for RoutePanTarget {
    fn control_type_and_character(&self) -> (ControlType, TargetCharacter) {
        (ControlType::AbsoluteContinuous, TargetCharacter::Continuous)
    }

    fn parse_as_value(&self, text: &str) -> Result<UnitValue, &'static str> {
        parse_value_from_pan(text)
    }

    fn format_value_without_unit(&self, value: UnitValue) -> String {
        format_value_as_pan(value)
    }

    fn hide_formatted_value(&self) -> bool {
        true
    }

    fn hide_formatted_step_size(&self) -> bool {
        true
    }

    fn value_unit(&self) -> &'static str {
        ""
    }

    fn step_size_unit(&self) -> &'static str {
        ""
    }

    fn format_value(&self, value: UnitValue) -> String {
        format_value_as_pan(value)
    }

    fn control(&self, value: ControlValue, _: ControlContext) -> Result<(), &'static str> {
        let pan = Pan::from_normalized_value(value.as_absolute()?.get());
        self.route
            .set_pan(pan)
            .map_err(|_| "couldn't set route pan")?;
        Ok(())
    }

    fn is_available(&self) -> bool {
        self.route.is_available()
    }

    fn project(&self) -> Option<Project> {
        Some(self.route.track().project())
    }

    fn track(&self) -> Option<&Track> {
        Some(&self.route.track())
    }

    fn route(&self) -> Option<&TrackRoute> {
        Some(&self.route)
    }

    fn process_change_event(
        &self,
        evt: &ChangeEvent,
        _: ControlContext,
    ) -> (bool, Option<UnitValue>) {
        match evt {
            ChangeEvent::TrackRoutePanChanged(e) if e.route == self.route => (
                true,
                Some(pan_unit_value(Pan::from_reaper_value(e.new_value))),
            ),
            _ => (false, None),
        }
    }
}

impl<'a> Target<'a> for RoutePanTarget {
    type Context = ();

    fn current_value(&self, _: ()) -> Option<UnitValue> {
        Some(pan_unit_value(self.route.pan()))
    }

    fn control_type(&self) -> ControlType {
        self.control_type_and_character().0
    }
}

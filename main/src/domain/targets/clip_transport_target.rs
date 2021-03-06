use crate::domain::{
    clip_play_state_unit_value, format_value_as_on_off, transport_is_enabled_unit_value,
    ClipChangedEvent, ControlContext, InstanceFeedbackEvent, RealearnTarget, SlotPlayOptions,
    TargetCharacter, TransportAction,
};
use helgoboss_learn::{ControlType, ControlValue, Target, UnitValue};
use reaper_high::{ChangeEvent, Project, Track};

#[derive(Clone, Debug, PartialEq)]
pub struct ClipTransportTarget {
    pub track: Option<Track>,
    pub slot_index: usize,
    pub action: TransportAction,
    pub play_options: SlotPlayOptions,
}

impl RealearnTarget for ClipTransportTarget {
    fn control_type_and_character(&self) -> (ControlType, TargetCharacter) {
        (
            ControlType::AbsoluteContinuousRetriggerable,
            TargetCharacter::Switch,
        )
    }

    fn format_value(&self, value: UnitValue) -> String {
        format_value_as_on_off(value).to_string()
    }

    fn control(&self, value: ControlValue, context: ControlContext) -> Result<(), &'static str> {
        use TransportAction::*;
        let on = !value.as_absolute()?.is_zero();
        let mut instance_state = context.instance_state.borrow_mut();
        match self.action {
            PlayStop => {
                if on {
                    instance_state.play(self.slot_index, self.track.clone(), self.play_options)?;
                } else {
                    instance_state.stop(self.slot_index, !self.play_options.next_bar)?;
                }
            }
            PlayPause => {
                if on {
                    instance_state.play(self.slot_index, self.track.clone(), self.play_options)?;
                } else {
                    instance_state.pause(self.slot_index)?;
                }
            }
            Stop => {
                if on {
                    instance_state.stop(self.slot_index, !self.play_options.next_bar)?;
                }
            }
            Pause => {
                if on {
                    instance_state.pause(self.slot_index)?;
                }
            }
            Record => {
                return Err("not supported at the moment");
            }
            Repeat => {
                instance_state.toggle_repeat(self.slot_index)?;
            }
        };
        Ok(())
    }

    fn is_available(&self) -> bool {
        // TODO-medium With clip targets we should check the control context (instance state) if
        //  slot filled.
        if let Some(t) = &self.track {
            if !t.is_available() {
                return false;
            }
        }
        true
    }

    fn project(&self) -> Option<Project> {
        self.track.as_ref().map(|t| t.project())
    }

    fn track(&self) -> Option<&Track> {
        self.track.as_ref()
    }

    fn process_change_event(
        &self,
        evt: &ChangeEvent,
        context: ControlContext,
    ) -> (bool, Option<UnitValue>) {
        // Feedback handled from instance-scoped feedback events.
        if let ChangeEvent::PlayStateChanged(e) = evt {
            let mut instance_state = context.instance_state.borrow_mut();
            instance_state.process_transport_change(e.new_value);
        };
        (false, None)
    }

    fn value_changed_from_instance_feedback_event(
        &self,
        evt: &InstanceFeedbackEvent,
    ) -> (bool, Option<UnitValue>) {
        match evt {
            InstanceFeedbackEvent::ClipChanged {
                slot_index: si,
                event,
            } if *si == self.slot_index => {
                use TransportAction::*;
                match self.action {
                    PlayStop | PlayPause | Stop | Pause => match event {
                        ClipChangedEvent::PlayStateChanged(new_state) => (
                            true,
                            Some(clip_play_state_unit_value(self.action, *new_state)),
                        ),
                        _ => (false, None),
                    },
                    // Not supported at the moment.
                    Record => (false, None),
                    Repeat => match event {
                        ClipChangedEvent::ClipRepeatChanged(new_state) => {
                            (true, Some(transport_is_enabled_unit_value(*new_state)))
                        }
                        _ => (false, None),
                    },
                }
            }
            _ => (false, None),
        }
    }
}

impl<'a> Target<'a> for ClipTransportTarget {
    type Context = ControlContext<'a>;

    fn current_value(&self, context: ControlContext<'a>) -> Option<UnitValue> {
        let instance_state = context.instance_state.borrow();
        use TransportAction::*;
        let val = match self.action {
            PlayStop | PlayPause | Stop | Pause => {
                let play_state = instance_state.get_slot(self.slot_index).ok()?.play_state();
                clip_play_state_unit_value(self.action, play_state)
            }
            Repeat => {
                let is_looped = instance_state
                    .get_slot(self.slot_index)
                    .ok()?
                    .repeat_is_enabled();
                transport_is_enabled_unit_value(is_looped)
            }
            Record => return None,
        };
        Some(val)
    }

    fn control_type(&self) -> ControlType {
        self.control_type_and_character().0
    }
}

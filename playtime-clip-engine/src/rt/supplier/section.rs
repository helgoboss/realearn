use crate::conversion_util::{
    convert_duration_in_frames_to_other_frame_rate, convert_duration_in_frames_to_seconds,
};
use crate::rt::buffer::AudioBufMut;
use crate::rt::supplier::fade_util::{apply_fade_in, apply_fade_out};
use crate::rt::supplier::midi_util::SilenceMidiBlockMode;
use crate::rt::supplier::{
    midi_util, AudioSupplier, ExactDuration, ExactFrameCount, MidiSupplier, PreBufferFillRequest,
    PreBufferSourceSkill, SupplyAudioRequest, SupplyMidiRequest, SupplyRequest, SupplyRequestInfo,
    SupplyResponse, SupplyResponseStatus, WithFrameRate,
};
use playtime_api::{MidiResetMessageRange, MidiResetMessages};
use reaper_medium::{BorrowedMidiEventList, DurationInSeconds, Hz};

#[derive(Debug)]
pub struct Section<S> {
    supplier: S,
    boundary: Boundary,
    midi_reset_msg_range: MidiResetMessageRange,
}

#[derive(PartialEq, Debug, Default)]
struct Boundary {
    start_frame: usize,
    length: Option<usize>,
}

impl Boundary {
    fn is_default(&self) -> bool {
        self == &Default::default()
    }
}

impl<S: WithFrameRate + ExactFrameCount> Section<S> {
    pub fn new(supplier: S) -> Self {
        Self {
            supplier,
            boundary: Default::default(),
            midi_reset_msg_range: Default::default(),
        }
    }

    pub fn start_frame(&self) -> usize {
        self.boundary.start_frame
    }

    pub fn length(&self) -> Option<usize> {
        self.boundary.length
    }

    pub fn set_midi_reset_msg_range(&mut self, range: MidiResetMessageRange) {
        self.midi_reset_msg_range = range;
    }

    pub fn set_bounds(&mut self, start_frame: usize, length: Option<usize>) {
        self.boundary.start_frame = start_frame;
        self.boundary.length = length;
    }

    pub fn reset(&mut self) {
        self.boundary = Default::default();
    }

    pub fn supplier(&self) -> &S {
        &self.supplier
    }

    pub fn supplier_mut(&mut self) -> &mut S {
        &mut self.supplier
    }

    fn get_instruction(
        &mut self,
        request: &impl SupplyRequest,
        dest_frame_count: usize,
        dest_frame_rate: Hz,
        is_midi: bool,
    ) -> Instruction {
        if self.boundary.is_default() {
            return Instruction::Bypass;
        }
        // Section is set (start and/or length).
        let source_frame_rate = self
            .supplier
            .frame_rate()
            .expect("supplier doesn't have frame rate yet");
        if !is_midi {
            debug_assert_eq!(source_frame_rate, dest_frame_rate);
        }
        let ideal_num_frames_to_be_consumed = if is_midi {
            convert_duration_in_frames_to_other_frame_rate(
                dest_frame_count,
                dest_frame_rate,
                source_frame_rate,
            )
        } else {
            // For audio, the source and destination frame rates are always equal in our chain setup.
            dest_frame_count
        };
        let ideal_end_frame_in_section =
            request.start_frame() + ideal_num_frames_to_be_consumed as isize;
        if ideal_end_frame_in_section <= 0 {
            // Pure count-in phase. Pass through for now.
            return Instruction::Bypass;
        }
        // TODO-high-downbeat If the start frame is < 0 and the end frame is > 0, we currently play
        //  some material which is shortly before the section start. I think one effect of this is
        //  that the MIDI piano clip sometimes plays the F note when using this boundary:
        //             boundary: Boundary {
        //                 start_frame: 1_024_000,
        //                 length: Some(1_024_000),
        //             }
        //  Let's deal with that as soon as we add support for customizable downbeats.
        // Determine source range
        let start_frame_in_source = self.boundary.start_frame as isize + request.start_frame();
        let (phase_two, num_frames_to_be_written) = match self.boundary.length {
            None => {
                // Section doesn't have right bound (easy).
                (PhaseTwo::Unbounded, dest_frame_count)
            }
            Some(length) => {
                // Section has right bound.
                if request.start_frame() > length as isize {
                    // We exceeded the section boundary. Return silence.
                    return Instruction::Return(SupplyResponse::exceeded_end());
                }
                // We are still within the section.
                let right_bound_in_source = self.boundary.start_frame + length;
                let ideal_end_frame_in_source =
                    start_frame_in_source + ideal_num_frames_to_be_consumed as isize;
                let (reached_bound, effective_end_frame_in_source) =
                    if ideal_end_frame_in_source < right_bound_in_source as isize {
                        (false, ideal_end_frame_in_source)
                    } else {
                        (true, right_bound_in_source as isize)
                    };
                let bounded_num_frames_to_be_consumed =
                    (effective_end_frame_in_source - start_frame_in_source) as usize;
                let bounded_num_frames_to_be_written = if is_midi {
                    convert_duration_in_frames_to_other_frame_rate(
                        bounded_num_frames_to_be_consumed,
                        source_frame_rate,
                        dest_frame_rate,
                    )
                } else {
                    // For audio, the source and destination frame rate are always equal in our chain setup.
                    bounded_num_frames_to_be_consumed
                };
                let phase_two = PhaseTwo::Bounded {
                    reached_bound,
                    bounded_num_frames_to_be_consumed,
                    bounded_num_frames_to_be_written,
                    ideal_num_frames_to_be_consumed,
                };
                (phase_two, bounded_num_frames_to_be_written)
            }
        };
        let data = SectionRequestData {
            phase_one: PhaseOne {
                start_frame: start_frame_in_source,
                info: SupplyRequestInfo {
                    audio_block_frame_offset: request.info().audio_block_frame_offset,
                    requester: "section-request",
                    note: "",
                    is_realtime: request.info().is_realtime,
                },
                num_frames_to_be_written,
            },
            phase_two,
        };
        Instruction::ApplySection(data)
    }

    fn generate_outer_response(
        &self,
        inner_response: SupplyResponse,
        phase_two: PhaseTwo,
    ) -> SupplyResponse {
        use PhaseTwo::*;
        match phase_two {
            Unbounded => {
                // Section has open end. In that case the inner response is valid.
                inner_response
            }
            Bounded {
                reached_bound,
                bounded_num_frames_to_be_consumed,
                bounded_num_frames_to_be_written,
                ideal_num_frames_to_be_consumed,
            } => {
                // Section has right bound.
                if reached_bound {
                    // Bound reached.
                    SupplyResponse::reached_end(
                        bounded_num_frames_to_be_consumed,
                        bounded_num_frames_to_be_written,
                    )
                } else {
                    // Bound not yet reached.
                    use SupplyResponseStatus::*;
                    match inner_response.status {
                        PleaseContinue => {
                            // Source has more material.
                            SupplyResponse::please_continue(bounded_num_frames_to_be_consumed)
                        }
                        ReachedEnd { .. } => {
                            // Source has reached its end (but the boundary is not reached yet).
                            SupplyResponse::please_continue(ideal_num_frames_to_be_consumed)
                        }
                    }
                }
            }
        }
    }
}

impl<S: AudioSupplier + WithFrameRate + ExactFrameCount> AudioSupplier for Section<S> {
    fn supply_audio(
        &mut self,
        request: &SupplyAudioRequest,
        dest_buffer: &mut AudioBufMut,
    ) -> SupplyResponse {
        let data = match self.get_instruction(
            request,
            dest_buffer.frame_count(),
            request.dest_sample_rate,
            false,
        ) {
            Instruction::Bypass => {
                return self.supplier.supply_audio(request, dest_buffer);
            }
            Instruction::Return(r) => return r,
            Instruction::ApplySection(d) => d,
        };
        let inner_request = SupplyAudioRequest {
            start_frame: data.phase_one.start_frame,
            dest_sample_rate: request.dest_sample_rate,
            info: data.phase_one.info,
            parent_request: Some(request),
            general_info: request.general_info,
        };
        let mut inner_dest_buffer =
            dest_buffer.slice_mut(0..data.phase_one.num_frames_to_be_written);
        let inner_response = self
            .supplier
            .supply_audio(&inner_request, &mut inner_dest_buffer);
        if self.boundary.start_frame > 0 {
            apply_fade_in(dest_buffer, request.start_frame);
        }
        if let Some(length) = self.boundary.length {
            apply_fade_out(dest_buffer, request.start_frame, length);
        }
        self.generate_outer_response(inner_response, data.phase_two)
    }

    fn channel_count(&self) -> usize {
        self.supplier.channel_count()
    }
}

impl<S: MidiSupplier + WithFrameRate + ExactFrameCount> MidiSupplier for Section<S> {
    fn supply_midi(
        &mut self,
        request: &SupplyMidiRequest,
        event_list: &mut BorrowedMidiEventList,
    ) -> SupplyResponse {
        let data = match self.get_instruction(
            request,
            request.dest_frame_count,
            request.dest_sample_rate,
            true,
        ) {
            Instruction::Bypass => {
                return self.supplier.supply_midi(request, event_list);
            }
            Instruction::Return(r) => return r,
            Instruction::ApplySection(d) => d,
        };
        let inner_request = SupplyMidiRequest {
            start_frame: data.phase_one.start_frame,
            dest_frame_count: data.phase_one.num_frames_to_be_written,
            info: data.phase_one.info.clone(),
            dest_sample_rate: request.dest_sample_rate,
            parent_request: request.parent_request,
            general_info: request.general_info,
        };
        let inner_response = self.supplier.supply_midi(&inner_request, event_list);
        // Reset MIDI at start if necessary
        if request.start_frame <= 0 {
            debug!("Silence MIDI at section start");
            midi_util::silence_midi(
                event_list,
                self.midi_reset_msg_range.left,
                SilenceMidiBlockMode::Prepend,
            );
        }
        // Reset MIDI at end if necessary
        if let PhaseTwo::Bounded {
            reached_bound: true,
            ..
        } = &data.phase_two
        {
            debug!("Silence MIDI at section end");
            midi_util::silence_midi(
                event_list,
                self.midi_reset_msg_range.right,
                SilenceMidiBlockMode::Append,
            );
        }
        self.generate_outer_response(inner_response, data.phase_two)
    }
}

impl<S: PreBufferSourceSkill> PreBufferSourceSkill for Section<S> {
    fn pre_buffer(&mut self, request: PreBufferFillRequest) {
        if self.boundary.is_default() {
            self.supplier.pre_buffer(request);
            return;
        }
        let inner_request = PreBufferFillRequest {
            start_frame: request.start_frame + self.boundary.start_frame as isize,
            ..request
        };
        self.supplier.pre_buffer(inner_request);
    }
}

impl<S: WithFrameRate> WithFrameRate for Section<S> {
    fn frame_rate(&self) -> Option<Hz> {
        self.supplier.frame_rate()
    }
}

impl<S: ExactFrameCount> ExactFrameCount for Section<S> {
    fn frame_count(&self) -> usize {
        if self.boundary.is_default() {
            return self.supplier.frame_count();
        }
        if let Some(length) = self.boundary.length {
            length
        } else {
            let source_frame_count = self.supplier.frame_count();
            source_frame_count.saturating_sub(self.boundary.start_frame)
        }
    }
}

impl<S: ExactDuration + WithFrameRate + ExactFrameCount> ExactDuration for Section<S> {
    fn duration(&self) -> DurationInSeconds {
        if self.boundary == Default::default() {
            return self.supplier.duration();
        };
        let frame_rate = match self.frame_rate() {
            None => return DurationInSeconds::MIN,
            Some(r) => r,
        };
        convert_duration_in_frames_to_seconds(self.frame_count(), frame_rate)
    }
}

enum Instruction {
    Bypass,
    ApplySection(SectionRequestData),
    Return(SupplyResponse),
}

struct SectionRequestData {
    phase_one: PhaseOne,
    phase_two: PhaseTwo,
}

struct PhaseOne {
    start_frame: isize,
    info: SupplyRequestInfo,
    num_frames_to_be_written: usize,
}

enum PhaseTwo {
    Unbounded,
    Bounded {
        reached_bound: bool,
        bounded_num_frames_to_be_consumed: usize,
        bounded_num_frames_to_be_written: usize,
        ideal_num_frames_to_be_consumed: usize,
    },
}
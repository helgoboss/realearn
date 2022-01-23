use crate::domain::clip_engine::buffer::{AudioBufMut, OwnedAudioBuffer};
use crate::domain::clip_engine::supplier::stretcher::resampling::Resampler;
use crate::domain::clip_engine::supplier::stretcher::time_stretching::SeriousTimeStretcher;
use crate::domain::clip_engine::supplier::{
    convert_duration_in_frames_to_seconds, convert_duration_in_seconds_to_frames, AudioSupplier,
    ExactFrameCount, MidiSupplier, SupplyAudioRequest, SupplyMidiRequest, SupplyResponse,
};
use core::cmp;
use reaper_medium::{
    BorrowedMidiEventList, BorrowedPcmSource, DurationInSeconds, Hz, PcmSourceTransfer,
};

mod resampling;
pub use resampling::*;
pub mod time_stretching;
pub use time_stretching::*;

pub struct Stretcher<S> {
    enabled: bool,
    supplier: S,
    tempo_factor: f64,
    audio_mode: StretchAudioMode,
}

#[derive(Debug)]
pub enum StretchAudioMode {
    /// Changes time but also pitch.
    Resampling(Resampler),
    /// Uses serious time stretching, without influencing pitch.
    Serious(SeriousTimeStretcher),
}

impl<S> Stretcher<S> {
    pub fn new(supplier: S) -> Self {
        Self {
            enabled: false,
            supplier,
            tempo_factor: 1.0,
            audio_mode: StretchAudioMode::Resampling(Resampler),
        }
    }

    pub fn reset(&mut self) {
        use StretchAudioMode::*;
        match &mut self.audio_mode {
            Resampling(s) => s.reset(),
            Serious(s) => s.reset(),
        }
    }

    pub fn supplier(&self) -> &S {
        &self.supplier
    }

    pub fn supplier_mut(&mut self) -> &mut S {
        &mut self.supplier
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_tempo_factor(&mut self, tempo_factor: f64) {
        self.tempo_factor = tempo_factor;
    }

    pub fn set_mode(&mut self, mode: StretchAudioMode) {
        self.audio_mode = mode;
    }

    fn ctx<'a, T>(&'a self, mode: &'a T) -> Ctx<'a, T, S>
    where
        S: AudioSupplier,
    {
        Ctx {
            supplier: &self.supplier,
            mode,
            tempo_factor: self.tempo_factor,
        }
    }
}

impl<S: AudioSupplier> AudioSupplier for Stretcher<S> {
    fn supply_audio(
        &self,
        request: &SupplyAudioRequest,
        dest_buffer: &mut AudioBufMut,
    ) -> SupplyResponse {
        if !self.enabled {
            return self.supplier.supply_audio(&request, dest_buffer);
        }
        use StretchAudioMode::*;
        match &self.audio_mode {
            Resampling(m) => self.ctx(m).supply_audio(request, dest_buffer),
            Serious(m) => self.ctx(m).supply_audio(request, dest_buffer),
        }
    }

    fn channel_count(&self) -> usize {
        self.supplier.channel_count()
    }

    fn sample_rate(&self) -> Hz {
        if !self.enabled {
            return self.supplier.sample_rate();
        }
        use StretchAudioMode::*;
        match &self.audio_mode {
            Resampling(m) => self.ctx(m).sample_rate(),
            Serious(m) => self.ctx(m).sample_rate(),
        }
    }
}

impl<S: MidiSupplier> MidiSupplier for Stretcher<S> {
    fn supply_midi(
        &self,
        request: &SupplyMidiRequest,
        event_list: &BorrowedMidiEventList,
    ) -> SupplyResponse {
        if !self.enabled {
            return self.supplier.supply_midi(&request, event_list);
        }
        let request = SupplyMidiRequest {
            dest_sample_rate: Hz::new(request.dest_sample_rate.get() / self.tempo_factor),
            ..*request
        };
        self.supplier.supply_midi(&request, event_list)
    }
}

impl<S: ExactFrameCount> ExactFrameCount for Stretcher<S> {
    fn frame_count(&self) -> usize {
        (self.supplier.frame_count() as f64 / self.tempo_factor).round() as usize
    }
}

pub struct Ctx<'a, M, S: AudioSupplier> {
    supplier: &'a S,
    mode: &'a M,
    tempo_factor: f64,
}

pub const DEFAULT_WAVEFORM_SIZE: usize = 64;
pub const DEFAULT_SPECTRUM_SIZE: usize = 64;

#[derive(Clone, Copy, Debug)]
pub struct DesktopAudioProfile {
    pub bpm: f64,
    pub seed: f64,
    pub noise: f64,
    pub bass_gain: f64,
    pub mid_gain: f64,
    pub treble_gain: f64,
}

impl Default for DesktopAudioProfile {
    fn default() -> Self {
        Self {
            bpm: 128.0,
            seed: 0.35,
            noise: 0.08,
            bass_gain: 0.95,
            mid_gain: 0.74,
            treble_gain: 0.62,
        }
    }
}

pub fn create_waveform(
    time_seconds: f64,
    points: usize,
    profile: &DesktopAudioProfile,
) -> Vec<f64> {
    let mut output = Vec::with_capacity(points.max(1));
    let count = points.max(1) as f64;
    let base = profile.seed + time_seconds;
    for index in 0..points.max(1) {
        let phase = index as f64 / count * core::f64::consts::TAU;
        let sample = (base + phase * 1.7).sin() * 0.45
            + (base * 2.0 + phase).cos() * 0.25
            + ((base * 3.0).sin() + (phase * 3.0).cos()) * 0.12;
        output.push((sample * profile.bass_gain).clamp(-1.0, 1.0));
    }
    output
}

pub fn create_spectrum(
    time_seconds: f64,
    points: usize,
    profile: &DesktopAudioProfile,
) -> Vec<f64> {
    let mut output = Vec::with_capacity(points.max(1));
    let count = points.max(1) as f64;
    let band_phase = time_seconds * profile.bpm / 120.0;

    for index in 0..points.max(1) {
        let normalized = index as f64 / count;
        let envelope = (1.0 - normalized).max(0.0).powf(2.2);
        let low = (band_phase + normalized * 2.8).sin().abs() * profile.bass_gain;
        let mid = (band_phase * 0.7 + normalized * 4.1).cos().abs() * profile.mid_gain;
        let hi = (band_phase * 0.9 + normalized * 8.3).sin().abs() * profile.treble_gain;
        let jitter = profile.noise * ((band_phase * normalized * 12.7).sin() * 0.5 + 0.5);
        output.push((envelope * (low * 0.44 + mid * 0.36 + hi * 0.20 + jitter)).clamp(0.0, 1.0));
    }

    output
}

/// Build a deterministic profile-based audio tuple for one frame.
pub fn build_audio_profile(
    frame: usize,
    fps: f64,
    profile: &DesktopAudioProfile,
) -> (f64, f64, f64, Vec<f64>, Vec<f64>) {
    let time_seconds = frame as f64 / fps.max(1.0);
    let beat = ((time_seconds * profile.bpm / 60.0).fract() * core::f64::consts::TAU).sin();
    let bass = ((beat + 1.0) * 0.5 * profile.bass_gain).clamp(0.0, 1.0);
    let mid = ((beat.cos() + 1.0) * 0.5 * profile.mid_gain).clamp(0.0, 1.0);
    let treble = ((time_seconds * 0.77).sin().abs() * profile.treble_gain).clamp(0.0, 1.0);
    let waveform = create_waveform(time_seconds, DEFAULT_WAVEFORM_SIZE, profile);
    let spectrum = create_spectrum(time_seconds, DEFAULT_SPECTRUM_SIZE, profile);
    (bass, mid, treble, waveform, spectrum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_waveform_returns_correct_length_and_clamps_values() {
        let profile = DesktopAudioProfile::default();

        let waveform = create_waveform(0.0, 0, &profile);
        // Should produce at least 1 sample even when points is 0
        assert!(!waveform.is_empty());

        let waveform = create_waveform(0.0, 16, &profile);
        assert_eq!(waveform.len(), 16);
        for sample in &waveform {
            assert!(*sample >= -1.0 && *sample <= 1.0);
        }

        let waveform = create_waveform(1.0, 64, &profile);
        assert_eq!(waveform.len(), 64);
    }

    #[test]
    fn create_waveform_is_deterministic_for_same_inputs() {
        let profile = DesktopAudioProfile::default();
        let a = create_waveform(2.5, 32, &profile);
        let b = create_waveform(2.5, 32, &profile);
        assert_eq!(a, b);
    }

    #[test]
    fn create_waveform_produces_different_output_for_different_times() {
        let profile = DesktopAudioProfile::default();
        let a = create_waveform(0.0, 32, &profile);
        let b = create_waveform(0.5, 32, &profile);
        assert_ne!(a, b);
    }

    #[test]
    fn create_spectrum_returns_correct_length_and_clamps_values() {
        let profile = DesktopAudioProfile::default();

        let spectrum = create_spectrum(0.0, 0, &profile);
        assert!(!spectrum.is_empty());

        let spectrum = create_spectrum(0.0, 32, &profile);
        assert_eq!(spectrum.len(), 32);
        for sample in &spectrum {
            assert!(*sample >= 0.0 && *sample <= 1.0);
        }

        let spectrum = create_spectrum(1.0, 64, &profile);
        assert_eq!(spectrum.len(), 64);
    }

    #[test]
    fn create_spectrum_is_deterministic_for_same_inputs() {
        let profile = DesktopAudioProfile::default();
        let a = create_spectrum(2.5, 32, &profile);
        let b = create_spectrum(2.5, 32, &profile);
        assert_eq!(a, b);
    }

    #[test]
    fn build_audio_profile_produces_valid_components() {
        let profile = DesktopAudioProfile::default();
        let (bass, mid, treble, waveform, spectrum) =
            build_audio_profile(0, 60.0, &profile);

        assert!(bass >= 0.0 && bass <= 1.0);
        assert!(mid >= 0.0 && mid <= 1.0);
        assert!(treble >= 0.0 && treble <= 1.0);
        assert_eq!(waveform.len(), DEFAULT_WAVEFORM_SIZE);
        assert_eq!(spectrum.len(), DEFAULT_SPECTRUM_SIZE);
    }

    #[test]
    fn build_audio_profile_handles_low_fps_gracefully() {
        let profile = DesktopAudioProfile::default();
        // fps = 0 should not panic; max(1.0) handles division
        let (_, _, _, waveform, spectrum) =
            build_audio_profile(0, 0.0, &profile);
        assert_eq!(waveform.len(), DEFAULT_WAVEFORM_SIZE);
        assert_eq!(spectrum.len(), DEFAULT_SPECTRUM_SIZE);
    }

    #[test]
    fn build_audio_profile_produces_different_outputs_per_frame() {
        let profile = DesktopAudioProfile::default();
        let (_, _, _, wave0, spec0) =
            build_audio_profile(0, 60.0, &profile);
        let (_, _, _, wave1, spec1) =
            build_audio_profile(1, 60.0, &profile);
        assert_ne!(wave0, wave1);
        assert_ne!(spec0, spec1);
    }

    #[test]
    fn desktop_audio_profile_defaults_are_reasonable() {
        let p = DesktopAudioProfile::default();
        assert!(p.bpm > 0.0);
        assert!(p.seed >= 0.0);
        assert!(p.noise >= 0.0);
        assert!(p.bass_gain > 0.0);
        assert!(p.mid_gain > 0.0);
        assert!(p.treble_gain > 0.0);
    }

    #[test]
    fn create_waveform_respects_bass_gain() {
        let flat_profile = DesktopAudioProfile {
            bass_gain: 1.0,
            ..DesktopAudioProfile::default()
        };
        let loud_profile = DesktopAudioProfile {
            bass_gain: 0.0,
            ..DesktopAudioProfile::default()
        };
        let wave_loud = create_waveform(0.0, 8, &flat_profile);
        let wave_quiet = create_waveform(0.0, 8, &loud_profile);
        // With bass_gain=0, all samples are clamped to 0.0
        for sample in &wave_quiet {
            assert_eq!(*sample, 0.0);
        }
        // With bass_gain=1.0, at least some samples should be non-zero
        assert!(wave_loud.iter().any(|s| *s != 0.0));
    }

    #[test]
    fn create_spectrum_envelope_decreases_with_frequency_index() {
        let profile = DesktopAudioProfile::default();
        let spectrum = create_spectrum(0.0, 8, &profile);
        // The first bin (index 0, normalized 0.0) gets the highest envelope
        // because envelope = (1.0 - normalized)^2.2
        let first = spectrum.first().copied().unwrap_or(0.0);
        let last = spectrum.last().copied().unwrap_or(0.0);
        assert!(first >= last);
    }
}

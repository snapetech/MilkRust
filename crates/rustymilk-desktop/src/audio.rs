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

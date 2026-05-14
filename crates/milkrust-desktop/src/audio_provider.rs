use std::collections::VecDeque;
use std::fmt;
use std::sync::{Arc, Mutex};

#[cfg(feature = "audio")]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

#[cfg(feature = "audio")]
use crate::{DesktopAudioContext, DesktopAudioFrame, DesktopAudioProvider};

#[cfg(feature = "audio")]
#[derive(Clone, Debug)]
pub struct CpalDesktopAudioProviderConfig {
    pub preferred_device_name: Option<String>,
    pub history_frames: usize,
}

#[cfg(feature = "audio")]
impl Default for CpalDesktopAudioProviderConfig {
    fn default() -> Self {
        Self {
            preferred_device_name: None,
            history_frames: 8192,
        }
    }
}

#[cfg(feature = "audio")]
#[derive(Debug)]
pub enum CpalDesktopAudioProviderError {
    NoInputDevices,
    DeviceUnavailable(String),
    UnsupportedInputStream(String),
    StreamCreationFailed(String),
    StreamPlayFailed(String),
}

#[cfg(feature = "audio")]
impl fmt::Display for CpalDesktopAudioProviderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoInputDevices => formatter.write_str("no input devices available"),
            Self::DeviceUnavailable(device) => {
                write!(formatter, "audio input device '{device}' not found")
            }
            Self::UnsupportedInputStream(details) => {
                write!(formatter, "unsupported audio input stream: {details}")
            }
            Self::StreamCreationFailed(details) => {
                write!(formatter, "failed to create input stream: {details}")
            }
            Self::StreamPlayFailed(details) => {
                write!(formatter, "failed to start input stream: {details}")
            }
        }
    }
}

#[cfg(feature = "audio")]
impl std::error::Error for CpalDesktopAudioProviderError {}

#[cfg(feature = "audio")]
struct CpalCaptureState {
    samples: VecDeque<f32>,
    max_samples: usize,
}

#[cfg(feature = "audio")]
impl CpalCaptureState {
    fn new(max_samples: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(max_samples.max(1)),
            max_samples: max_samples.max(1),
        }
    }

    fn push_mono_samples(&mut self, samples: impl IntoIterator<Item = f32>) {
        for sample in samples {
            self.samples.push_back(sample.clamp(-1.0, 1.0));
            while self.samples.len() > self.max_samples {
                let _ = self.samples.pop_front();
            }
        }
    }

    fn snapshot(&self) -> Vec<f32> {
        self.samples.iter().copied().collect()
    }
}

#[cfg(feature = "audio")]
pub struct CpalDesktopAudioProvider {
    device_name: String,
    sample_rate: f64,
    channels: usize,
    history_frames: usize,
    _stream: Option<cpal::Stream>,
    state: Arc<Mutex<CpalCaptureState>>,
}

#[cfg(feature = "audio")]
impl CpalDesktopAudioProvider {
    pub fn available_device_names() -> Vec<String> {
        let host = cpal::default_host();
        host.input_devices()
            .map(|devices| {
                devices
                    .filter_map(|device| device.name().ok())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }

    pub fn new() -> Result<Self, CpalDesktopAudioProviderError> {
        Self::new_with_config(CpalDesktopAudioProviderConfig::default())
    }

    pub fn new_with_device_name(device_name: &str) -> Result<Self, CpalDesktopAudioProviderError> {
        Self::new_with_config(CpalDesktopAudioProviderConfig {
            preferred_device_name: Some(device_name.to_string()),
            ..Default::default()
        })
    }

    pub fn new_with_config(
        config: CpalDesktopAudioProviderConfig,
    ) -> Result<Self, CpalDesktopAudioProviderError> {
        let host = cpal::default_host();
        let devices = host
            .input_devices()
            .map_err(|_| CpalDesktopAudioProviderError::NoInputDevices)?;
        let mut selected: Option<cpal::Device> = None;
        let mut selected_name = String::new();

        if let Some(name) = &config.preferred_device_name {
            let target = name.to_ascii_lowercase();
            for device in devices {
                let current_name = device.name().unwrap_or_default();
                let current_name_lc = current_name.to_ascii_lowercase();
                if current_name_lc == target {
                    selected = Some(device);
                    selected_name = current_name;
                    break;
                }
                if selected.is_none() && current_name_lc.contains(&target) {
                    selected = Some(device);
                    selected_name = current_name;
                }
            }
        }

        let device = match selected {
            Some(device) => device,
            None => {
                let default_device = host
                    .default_input_device()
                    .ok_or(CpalDesktopAudioProviderError::NoInputDevices)?;
                if let Some(preferred) = &config.preferred_device_name {
                    return Err(CpalDesktopAudioProviderError::DeviceUnavailable(
                        preferred.clone(),
                    ));
                }
                selected_name = default_device
                    .name()
                    .unwrap_or_else(|_| "default-input-device".to_string());
                default_device
            }
        };

        let supported_config = device.default_input_config().map_err(|error| {
            CpalDesktopAudioProviderError::UnsupportedInputStream(error.to_string())
        })?;
        let sample_format = supported_config.sample_format();
        let stream_config: cpal::StreamConfig = supported_config.into();
        let sample_rate = stream_config.sample_rate.0 as f64;
        let channels = stream_config.channels as usize;
        let history_frames = config.history_frames.max(1024);
        let state = Arc::new(Mutex::new(CpalCaptureState::new(history_frames)));
        let capture_state = Arc::clone(&state);

        let stream: cpal::Stream = match sample_format {
            cpal::SampleFormat::F32 => {
                build_input_stream(&device, &stream_config, channels, capture_state, |sample| {
                    sample
                })?
            }
            cpal::SampleFormat::F64 => build_input_stream(
                &device,
                &stream_config,
                channels,
                capture_state,
                |sample: f64| sample as f32,
            )?,
            cpal::SampleFormat::I16 => build_input_stream(
                &device,
                &stream_config,
                channels,
                capture_state,
                |sample: i16| (sample as f64 / i16::MAX as f64) as f32,
            )?,
            cpal::SampleFormat::U16 => build_input_stream(
                &device,
                &stream_config,
                channels,
                capture_state,
                |sample: u16| sample as f32 / u16::MAX as f32 * 2.0 - 1.0,
            )?,
            _ => {
                return Err(CpalDesktopAudioProviderError::UnsupportedInputStream(
                    "unsupported sample format".to_string(),
                ))
            }
        };

        stream
            .play()
            .map_err(|error| CpalDesktopAudioProviderError::StreamPlayFailed(error.to_string()))?;

        Ok(Self {
            device_name: selected_name,
            sample_rate,
            channels,
            history_frames,
            _stream: Some(stream),
            state,
        })
    }

    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
    }

    pub fn channels(&self) -> usize {
        self.channels
    }

    pub fn history_frames(&self) -> usize {
        self.history_frames
    }
}

#[cfg(feature = "audio")]
fn build_input_stream<T, F>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    channels: usize,
    state: Arc<Mutex<CpalCaptureState>>,
    convert: F,
) -> Result<cpal::Stream, CpalDesktopAudioProviderError>
where
    T: cpal::SizedSample + cpal::Sample + Send + 'static,
    F: Fn(T) -> f32 + Send + 'static + Copy,
{
    let channel_count = channels.max(1);
    device
        .build_input_stream(
            config,
            move |input: &[T], _| {
                let mut frames = input.chunks_exact(channel_count);
                let mono_samples = frames.by_ref().filter_map(|frame| {
                    if frame.is_empty() {
                        None
                    } else {
                        let sum = frame.iter().map(|value| convert(*value)).sum::<f32>();
                        Some(sum / frame.len() as f32)
                    }
                });
                if let Ok(mut guard) = state.lock() {
                    guard.push_mono_samples(mono_samples);
                }
            },
            |_error| {},
            None,
        )
        .map_err(|error| CpalDesktopAudioProviderError::StreamCreationFailed(error.to_string()))
}

#[cfg(feature = "audio")]
impl DesktopAudioProvider for CpalDesktopAudioProvider {
    fn provide_audio_frame(&self, context: &DesktopAudioContext) -> DesktopAudioFrame {
        let waveform_size = context.waveform_size.max(1);
        let spectrum_size = context.spectrum_size.max(1);
        let mut spectrum_bins = vec![0.0f64; spectrum_size];
        let mut spectrum_counts = vec![0f64; spectrum_size];

        let samples = self
            .state
            .lock()
            .map(|state| state.snapshot())
            .unwrap_or_default();

        let waveform = if samples.is_empty() {
            vec![0.0; waveform_size]
        } else {
            let start = samples.len().saturating_sub(waveform_size);
            let slice = &samples[start..];
            let mut output = Vec::with_capacity(waveform_size);
            output.extend(
                slice
                    .iter()
                    .map(|value| *value as f64)
                    .chain(std::iter::repeat_n(
                        0.0,
                        waveform_size.saturating_sub(slice.len()),
                    )),
            );
            output
        };

        if !samples.is_empty() {
            for (index, sample) in samples.iter().enumerate() {
                let band = (index * spectrum_size) / samples.len().max(1);
                let band = band.min(spectrum_size - 1);
                let value = sample.abs() as f64;
                spectrum_bins[band] += value;
                spectrum_counts[band] += 1.0;
            }
        }

        let mut spectrum = vec![0.0; spectrum_size];
        for index in 0..spectrum_size {
            let average = if spectrum_counts[index] > 0.0 {
                spectrum_bins[index] / spectrum_counts[index]
            } else {
                0.0
            };
            spectrum[index] = (average * 2.5).min(1.0);
        }

        let bass = if spectrum.is_empty() {
            0.0
        } else {
            let split = (spectrum.len() / 3).max(1);
            let bass_sum: f64 = spectrum.iter().take(split).sum();
            (bass_sum / split as f64).clamp(0.0, 1.0)
        };
        let mid = if spectrum.len() <= 2 {
            0.0
        } else {
            let split = (spectrum.len() / 3).max(1);
            let end = (split * 2).min(spectrum.len());
            let mid_sum: f64 = spectrum.iter().skip(split).take(end - split).sum();
            (mid_sum / split as f64).clamp(0.0, 1.0)
        };
        let treble = if spectrum.len() <= 3 {
            0.0
        } else {
            let split = (spectrum.len() / 3).max(1);
            let start = ((split * 2).min(spectrum.len())) as usize;
            let treble_sum: f64 = spectrum.iter().skip(start).sum();
            (treble_sum / (spectrum.len() - start).max(1) as f64).clamp(0.0, 1.0)
        };

        DesktopAudioFrame {
            bass,
            mid,
            treble,
            waveform,
            spectrum,
        }
    }
}

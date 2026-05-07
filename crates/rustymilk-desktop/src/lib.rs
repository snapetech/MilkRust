use rustymilk_core::{
    rustymilk_frame_set_from_source_with_audio, RustyMilkFrameSet, RustyMilkFrameSetRuntime,
};
use rustymilk_renderer_core::RustyMilkBatchRenderer;
use rustymilk_renderer_headless::{create_headless_batches, RustyMilkHeadlessRenderer};

pub mod audio;
#[cfg(feature = "audio")]
pub mod audio_provider;
mod cli_support;
mod player_api;
mod runner_player;
#[cfg(feature = "ui")]
mod runner_player_ui;
mod runner_probe;
mod runner_studio;
pub mod session;

pub use audio::{build_audio_profile, create_spectrum, create_waveform, DesktopAudioProfile};
#[cfg(feature = "audio")]
pub use audio_provider::{
    CpalDesktopAudioProvider, CpalDesktopAudioProviderConfig, CpalDesktopAudioProviderError,
};
pub use cli_support::{
    collect_pack_plugins, collect_preset_inputs, parse_non_negative_f64, parse_positive_f64,
    parse_positive_usize, write_pack_plugin_report, PackPluginInput, PresetInput,
};
pub use player_api::{
    DesktopAudioContext, DesktopAudioFrame, DesktopAudioProvider, DesktopDataPlugin,
    DesktopPlayerEngine, DesktopPlayerEngineConfig, DesktopPlayerError, DesktopPlayerFrame,
    DesktopPlayerPreset, DesktopPlayerState, DesktopPlugin, DesktopPluginDataDescriptor,
    DesktopPluginFrameContext, DesktopPluginHeartbeatContext, DesktopPluginPresetChangeContext,
    DesktopPluginPresetContext, DesktopPluginRenderContext, SyntheticDesktopAudioProvider,
};
pub use runner_player::run_desktop_player;
#[cfg(feature = "ui")]
pub use runner_player_ui::run_desktop_player_ui;
pub use runner_probe::run_desktop_probe;
pub use runner_studio::run_desktop_studio;
pub use session::{
    collect_headless_frames, summarize_headless_frames, DesktopSessionConfig, DesktopSessionFrame,
    DesktopSessionSummary, DesktopSessionTiming,
};

#[derive(Clone, Debug)]
pub struct RustyMilkFrameSetReport {
    pub source_title: String,
    pub transition_mode: String,
    pub transition_seconds: f64,
    pub source_preset_count: usize,
    pub frame_count: usize,
    pub line_vertices: usize,
    pub point_vertices: usize,
    pub textured_vertices: usize,
    pub triangle_vertices: usize,
}

/// Creates a lightweight frame-set render report for a preset source.
///
/// This is intentionally renderer-agnostic and is used by both prototype desktop hosts
/// and tooling that needs deterministic runtime coverage without opening a browser.
pub fn collect_frame_set_report(
    source: &str,
    time_seconds: f64,
    bass: f64,
    mid: f64,
    treble: f64,
    waveform: &[f64],
    spectrum: &[f64],
) -> (RustyMilkFrameSet, RustyMilkFrameSetReport) {
    let frame_set = rustymilk_frame_set_from_source_with_audio(
        source,
        time_seconds,
        bass,
        mid,
        treble,
        waveform,
        spectrum,
    );
    let report = report_frame_set(&frame_set);
    (frame_set, report)
}

pub fn report_frame_set(frame_set: &RustyMilkFrameSet) -> RustyMilkFrameSetReport {
    let mut renderer = RustyMilkHeadlessRenderer::new();
    let batches = create_headless_batches(frame_set);
    let stats = renderer
        .render_batches(&batches)
        .expect("headless renderer is infallible");

    RustyMilkFrameSetReport {
        source_title: frame_set.title.clone(),
        transition_mode: frame_set.transition_mode.clone(),
        transition_seconds: frame_set.transition_seconds,
        source_preset_count: frame_set.preset_count,
        frame_count: frame_set.entries.len(),
        line_vertices: stats.line_vertices,
        point_vertices: stats.point_vertices,
        textured_vertices: stats.textured_vertices,
        triangle_vertices: stats.triangle_vertices,
    }
}

/// Provides a deterministic baseline runtime for future desktop host loops.
#[derive(Default)]
pub struct RustyMilkFrameSetRuntimeHost {
    runtime: RustyMilkFrameSetRuntime,
    source: String,
}

impl RustyMilkFrameSetRuntimeHost {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render_frame_set(
        &mut self,
        source: &str,
        time_seconds: f64,
        bass: f64,
        mid: f64,
        treble: f64,
        waveform: &[f64],
        spectrum: &[f64],
    ) -> RustyMilkFrameSet {
        self.source = source.to_string();
        self.runtime.render_source_with_audio(
            source,
            time_seconds,
            bass,
            mid,
            treble,
            waveform,
            spectrum,
        )
    }
}

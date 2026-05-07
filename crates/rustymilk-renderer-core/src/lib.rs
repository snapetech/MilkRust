use rustymilk_core::{RustyMilkFrameSet, RustyMilkWebGpuFrameSetBatches};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RustyMilkRendererBackend {
    Canvas2d,
    Headless,
    WebGl2,
    WebGpu,
    Wgpu,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RustyMilkRendererCapabilities {
    pub backend: RustyMilkRendererBackend,
    pub display_name: String,
    pub feedback_textures: bool,
    pub max_texture_size: Option<u32>,
    pub named_textures: bool,
    pub shader_translation: bool,
    pub supports_capture: bool,
    pub supports_gpu_timing: bool,
    pub supports_headless: bool,
    pub webgpu_ready: bool,
}

impl RustyMilkRendererCapabilities {
    pub fn headless() -> Self {
        Self {
            backend: RustyMilkRendererBackend::Headless,
            display_name: "RustyMilk headless renderer".to_string(),
            feedback_textures: false,
            max_texture_size: None,
            named_textures: false,
            shader_translation: false,
            supports_capture: true,
            supports_gpu_timing: false,
            supports_headless: true,
            webgpu_ready: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RustyMilkRenderStats {
    pub frame_entries: usize,
    pub line_vertices: usize,
    pub point_vertices: usize,
    pub textured_vertices: usize,
    pub triangle_vertices: usize,
}

pub trait RustyMilkRenderer {
    type Error;

    fn capabilities(&self) -> RustyMilkRendererCapabilities;

    fn render_frame_set(
        &mut self,
        frame_set: &RustyMilkFrameSet,
    ) -> Result<RustyMilkRenderStats, Self::Error>;
}

pub trait RustyMilkBatchRenderer {
    type Error;

    fn render_batches(
        &mut self,
        batches: &RustyMilkWebGpuFrameSetBatches,
    ) -> Result<RustyMilkRenderStats, Self::Error>;
}

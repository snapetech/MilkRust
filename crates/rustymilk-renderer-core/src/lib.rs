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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headless_capabilities_are_correct() {
        let caps = RustyMilkRendererCapabilities::headless();
        assert_eq!(caps.backend, RustyMilkRendererBackend::Headless);
        assert_eq!(caps.display_name, "RustyMilk headless renderer");
        assert!(!caps.feedback_textures);
        assert_eq!(caps.max_texture_size, None);
        assert!(!caps.named_textures);
        assert!(!caps.shader_translation);
        assert!(caps.supports_capture);
        assert!(!caps.supports_gpu_timing);
        assert!(caps.supports_headless);
        assert!(caps.webgpu_ready);
    }

    #[test]
    fn headless_capabilities_clone_and_eq() {
        let caps1 = RustyMilkRendererCapabilities::headless();
        let caps2 = caps1.clone();
        assert_eq!(caps1, caps2);
    }

    #[test]
    fn render_stats_default_values() {
        let stats = RustyMilkRenderStats {
            frame_entries: 5,
            line_vertices: 10,
            point_vertices: 0,
            textured_vertices: 20,
            triangle_vertices: 15,
        };
        assert_eq!(stats.frame_entries, 5);
        assert_eq!(stats.line_vertices, 10);
        assert_eq!(stats.point_vertices, 0);
        assert_eq!(stats.textured_vertices, 20);
        assert_eq!(stats.triangle_vertices, 15);
    }

    #[test]
    fn render_stats_clone() {
        let stats = RustyMilkRenderStats {
            frame_entries: 1,
            line_vertices: 2,
            point_vertices: 3,
            textured_vertices: 4,
            triangle_vertices: 5,
        };
        let cloned = stats.clone();
        assert_eq!(stats, cloned);
    }

    #[test]
    fn renderer_backend_variants_debug() {
        let backends = [
            RustyMilkRendererBackend::Canvas2d,
            RustyMilkRendererBackend::Headless,
            RustyMilkRendererBackend::WebGl2,
            RustyMilkRendererBackend::WebGpu,
            RustyMilkRendererBackend::Wgpu,
        ];
        for backend in &backends {
            let debug_str = format!("{backend:?}");
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn renderer_capabilities_debug() {
        let caps = RustyMilkRendererCapabilities::headless();
        let debug_str = format!("{caps:?}");
        assert!(debug_str.contains("headless"));
    }
}

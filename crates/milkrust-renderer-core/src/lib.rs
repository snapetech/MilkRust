use milkrust_core::{MilkRustFrameSet, MilkRustWebGpuFrameSetBatches};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MilkRustRendererBackend {
    Canvas2d,
    Headless,
    WebGl2,
    WebGpu,
    Wgpu,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MilkRustRendererCapabilities {
    pub backend: MilkRustRendererBackend,
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

impl MilkRustRendererCapabilities {
    pub fn headless() -> Self {
        Self {
            backend: MilkRustRendererBackend::Headless,
            display_name: "MilkRust headless renderer".to_string(),
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
pub struct MilkRustRenderStats {
    pub frame_entries: usize,
    pub line_vertices: usize,
    pub point_vertices: usize,
    pub textured_vertices: usize,
    pub triangle_vertices: usize,
}

pub trait MilkRustRenderer {
    type Error;

    fn capabilities(&self) -> MilkRustRendererCapabilities;

    fn render_frame_set(
        &mut self,
        frame_set: &MilkRustFrameSet,
    ) -> Result<MilkRustRenderStats, Self::Error>;
}

pub trait MilkRustBatchRenderer {
    type Error;

    fn render_batches(
        &mut self,
        batches: &MilkRustWebGpuFrameSetBatches,
    ) -> Result<MilkRustRenderStats, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headless_capabilities_are_correct() {
        let caps = MilkRustRendererCapabilities::headless();
        assert_eq!(caps.backend, MilkRustRendererBackend::Headless);
        assert_eq!(caps.display_name, "MilkRust headless renderer");
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
        let caps1 = MilkRustRendererCapabilities::headless();
        let caps2 = caps1.clone();
        assert_eq!(caps1, caps2);
    }

    #[test]
    fn render_stats_default_values() {
        let stats = MilkRustRenderStats {
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
        let stats = MilkRustRenderStats {
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
            MilkRustRendererBackend::Canvas2d,
            MilkRustRendererBackend::Headless,
            MilkRustRendererBackend::WebGl2,
            MilkRustRendererBackend::WebGpu,
            MilkRustRendererBackend::Wgpu,
        ];
        for backend in &backends {
            let debug_str = format!("{backend:?}");
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn renderer_capabilities_debug() {
        let caps = MilkRustRendererCapabilities::headless();
        let debug_str = format!("{caps:?}");
        assert!(debug_str.contains("headless"));
    }
}

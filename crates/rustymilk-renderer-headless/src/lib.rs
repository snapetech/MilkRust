use rustymilk_core::{
    create_rustymilk_webgpu_frame_set_batches, RustyMilkFrameSet, RustyMilkPrimitiveMode,
    RustyMilkWebGpuFrameSetBatches,
};
use rustymilk_renderer_core::{
    RustyMilkBatchRenderer, RustyMilkRenderStats, RustyMilkRenderer, RustyMilkRendererCapabilities,
};

#[derive(Clone, Debug, Default)]
pub struct RustyMilkHeadlessRenderer {
    last_stats: Option<RustyMilkRenderStats>,
}

impl RustyMilkHeadlessRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn last_stats(&self) -> Option<&RustyMilkRenderStats> {
        self.last_stats.as_ref()
    }
}

impl RustyMilkRenderer for RustyMilkHeadlessRenderer {
    type Error = std::convert::Infallible;

    fn capabilities(&self) -> RustyMilkRendererCapabilities {
        RustyMilkRendererCapabilities::headless()
    }

    fn render_frame_set(
        &mut self,
        frame_set: &RustyMilkFrameSet,
    ) -> Result<RustyMilkRenderStats, Self::Error> {
        let mut stats = RustyMilkRenderStats {
            frame_entries: frame_set.entries.len(),
            line_vertices: 0,
            point_vertices: 0,
            textured_vertices: 0,
            triangle_vertices: 0,
        };

        for entry in &frame_set.entries {
            for primitive in &entry.frame.primitives {
                let vertices = primitive.vertices.len() / 2;
                match primitive.mode {
                    RustyMilkPrimitiveMode::LineStrip | RustyMilkPrimitiveMode::Lines => {
                        stats.line_vertices += vertices;
                    }
                    RustyMilkPrimitiveMode::Points => {
                        stats.point_vertices += vertices;
                    }
                    RustyMilkPrimitiveMode::TriangleFan | RustyMilkPrimitiveMode::Triangles => {
                        stats.triangle_vertices += vertices;
                    }
                }
            }
            stats.textured_vertices += entry
                .frame
                .textured_primitives
                .iter()
                .map(|primitive| primitive.vertices.len() / 2)
                .sum::<usize>();
        }

        self.last_stats = Some(stats.clone());
        Ok(stats)
    }
}

impl RustyMilkBatchRenderer for RustyMilkHeadlessRenderer {
    type Error = std::convert::Infallible;

    fn render_batches(
        &mut self,
        batches: &RustyMilkWebGpuFrameSetBatches,
    ) -> Result<RustyMilkRenderStats, Self::Error> {
        let stats = RustyMilkRenderStats {
            frame_entries: batches.composite_batches.len().max(1),
            line_vertices: batches.line_vertices.len() / 6,
            point_vertices: batches.point_vertices.len() / 6,
            textured_vertices: batches.textured_vertices.len() / 8,
            triangle_vertices: batches.filled_vertices.len() / 6,
        };
        self.last_stats = Some(stats.clone());
        Ok(stats)
    }
}

pub fn create_headless_batches(frame_set: &RustyMilkFrameSet) -> RustyMilkWebGpuFrameSetBatches {
    create_rustymilk_webgpu_frame_set_batches(frame_set)
}

#[cfg(test)]
mod tests {
    use rustymilk_core::{
        rustymilk_frame_set_from_source_with_audio, RustyMilkPrimitiveMode,
        RustyMilkTexturedPrimitiveMode,
    };
    use rustymilk_renderer_core::{RustyMilkBatchRenderer, RustyMilkRenderer};

    use super::*;

    #[test]
    fn headless_renderer_reports_frame_set_stats() {
        let frame_set = rustymilk_frame_set_from_source_with_audio(
            "name=Headless\nshape00_enabled=1\nshape00_sides=4\nwavecode_0_enabled=1\nwavecode_0_samples=8\nwavecode_0_per_point1=x=i;",
            1.0,
            0.5,
            0.4,
            0.3,
            &[-1.0, 0.0, 1.0],
            &[0.0, 0.5, 1.0],
        );
        let mut renderer = RustyMilkHeadlessRenderer::new();
        let stats = renderer.render_frame_set(&frame_set).unwrap();

        assert_eq!(stats.frame_entries, 1);
        assert!(stats.line_vertices > 0);
        assert!(stats.triangle_vertices > 0);
    }

    #[test]
    fn headless_renderer_reports_batch_stats() {
        let frame_set = rustymilk_frame_set_from_source_with_audio(
            "name=Headless batches\nshape00_enabled=1\nshape00_sides=3\nshape00_textured=1\nsprite00_enabled=1\nsprite00_image=logo.png",
            1.0,
            0.5,
            0.4,
            0.3,
            &[-1.0, 0.0, 1.0],
            &[0.0, 0.5, 1.0],
        );
        let batches = create_headless_batches(&frame_set);
        let mut renderer = RustyMilkHeadlessRenderer::new();
        let stats = renderer.render_batches(&batches).unwrap();

        assert_eq!(stats.frame_entries, 1);
        assert!(stats.textured_vertices > 0);
        assert!(matches!(
            frame_set.entries[0].frame.primitives[0].mode,
            RustyMilkPrimitiveMode::LineStrip
                | RustyMilkPrimitiveMode::Lines
                | RustyMilkPrimitiveMode::TriangleFan
                | RustyMilkPrimitiveMode::Triangles
                | RustyMilkPrimitiveMode::Points
        ));
        assert!(matches!(
            frame_set.entries[0].frame.textured_primitives[0].mode,
            RustyMilkTexturedPrimitiveMode::Quad | RustyMilkTexturedPrimitiveMode::TriangleFan
        ));
    }
}

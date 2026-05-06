use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use rustymilk_core::{
    clamp_unit, create_repeated_rustymilk_colors, get_rustymilk_texture_name_aliases,
    RustyMilkFrame, RustyMilkFrameSet, RustyMilkPrimitiveMode, RustyMilkTexturedPrimitiveMode,
};
use wasm_bindgen::{prelude::*, JsCast};

pub(crate) enum RustyMilkRenderer {
    WebGl(RustyMilkWebGlRendererSet),
    Canvas {
        canvas: web_sys::HtmlCanvasElement,
        context: web_sys::CanvasRenderingContext2d,
    },
}

impl RustyMilkRenderer {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::WebGl(_) => "Rust WebGL2 renderer active",
            Self::Canvas { .. } => "Canvas renderer fallback active",
        }
    }

    pub(crate) fn render_frame_set(&self, frame_set: &RustyMilkFrameSet, time: f64) {
        if frame_set.entries.is_empty() {
            return;
        }
        match self {
            Self::WebGl(renderer_set) => renderer_set.render_frame_set(frame_set, time),
            Self::Canvas { canvas, context } => {
                for (index, entry) in frame_set.entries.iter().enumerate() {
                    context.save();
                    context.set_global_alpha(clamp_unit(entry.blend_alpha));
                    if index == 0 {
                        render_rustymilk_canvas_frame(context, canvas, &entry.frame, time);
                    } else {
                        render_rustymilk_canvas_overlay_frame(context, canvas, &entry.frame, time);
                    }
                    context.restore();
                }
            }
        }
    }
}

pub(crate) struct RustyMilkWebGlRendererSet {
    gl: web_sys::WebGl2RenderingContext,
    renderers: RefCell<Vec<RustyMilkWebGlRenderer>>,
    texture_assets: Rc<RefCell<BTreeMap<String, String>>>,
}

struct RustyMilkWebGlRenderer {
    buffer: web_sys::WebGlBuffer,
    feedback_targets: RefCell<RustyMilkFeedbackTargets>,
    gl: web_sys::WebGl2RenderingContext,
    primitive_buffer: web_sys::WebGlBuffer,
    primitive_color_buffer: web_sys::WebGlBuffer,
    primitive_program: web_sys::WebGlProgram,
    procedural_texture: web_sys::WebGlTexture,
    named_textures: RefCell<BTreeMap<String, web_sys::WebGlTexture>>,
    program: web_sys::WebGlProgram,
    textured_position_buffer: web_sys::WebGlBuffer,
    textured_program: web_sys::WebGlProgram,
    textured_uv_buffer: web_sys::WebGlBuffer,
    translated_program: RefCell<Option<RustyMilkTranslatedProgram>>,
    texture_assets: Rc<RefCell<BTreeMap<String, String>>>,
    u_color: Option<web_sys::WebGlUniformLocation>,
    u_counts: Option<web_sys::WebGlUniformLocation>,
    u_display_only: Option<web_sys::WebGlUniformLocation>,
    u_feedback: Option<web_sys::WebGlUniformLocation>,
    u_motion: Option<web_sys::WebGlUniformLocation>,
    u_output_alpha: Option<web_sys::WebGlUniformLocation>,
    u_previous_frame: Option<web_sys::WebGlUniformLocation>,
    u_primitive_point_size: Option<web_sys::WebGlUniformLocation>,
    u_resolution: Option<web_sys::WebGlUniformLocation>,
    u_textured_alpha: Option<web_sys::WebGlUniformLocation>,
    u_textured_sampler: Option<web_sys::WebGlUniformLocation>,
    u_textured_tint: Option<web_sys::WebGlUniformLocation>,
    u_time: Option<web_sys::WebGlUniformLocation>,
    u_warp_color: Option<web_sys::WebGlUniformLocation>,
    u_warp_feedback: Option<web_sys::WebGlUniformLocation>,
    u_warp_output_alpha: Option<web_sys::WebGlUniformLocation>,
    u_warp_previous_frame: Option<web_sys::WebGlUniformLocation>,
    warp_position_buffer: web_sys::WebGlBuffer,
    warp_program: web_sys::WebGlProgram,
    warp_uv_buffer: web_sys::WebGlBuffer,
}

struct RustyMilkTranslatedProgram {
    program: web_sys::WebGlProgram,
    source: String,
}

struct RustyMilkFeedbackTargets {
    height: i32,
    read_index: usize,
    targets: [RustyMilkFeedbackTarget; 2],
    width: i32,
}

struct RustyMilkFeedbackTarget {
    framebuffer: web_sys::WebGlFramebuffer,
    texture: web_sys::WebGlTexture,
}

pub(crate) fn rustymilk_renderer(
    canvas: &web_sys::HtmlCanvasElement,
    texture_assets: Rc<RefCell<BTreeMap<String, String>>>,
) -> Result<RustyMilkRenderer, JsValue> {
    if let Some(context) = canvas.get_context("webgl2")? {
        if let Ok(gl) = context.dyn_into::<web_sys::WebGl2RenderingContext>() {
            if let Ok(renderer) = RustyMilkWebGlRendererSet::new(gl, texture_assets) {
                return Ok(RustyMilkRenderer::WebGl(renderer));
            }
        }
    }
    let context: web_sys::CanvasRenderingContext2d = canvas
        .get_context("2d")?
        .ok_or_else(|| JsValue::from_str("2D canvas is unavailable"))?
        .dyn_into()?;
    Ok(RustyMilkRenderer::Canvas {
        canvas: canvas.clone(),
        context,
    })
}

impl RustyMilkWebGlRendererSet {
    fn new(
        gl: web_sys::WebGl2RenderingContext,
        texture_assets: Rc<RefCell<BTreeMap<String, String>>>,
    ) -> Result<Self, JsValue> {
        Ok(Self {
            gl,
            renderers: RefCell::new(Vec::new()),
            texture_assets,
        })
    }

    fn render_frame_set(&self, frame_set: &RustyMilkFrameSet, time: f64) {
        if frame_set.entries.is_empty() || !self.ensure_renderer_count(frame_set.entries.len()) {
            return;
        }
        let renderers = self.renderers.borrow();
        for (index, entry) in frame_set.entries.iter().enumerate() {
            if let Some(renderer) = renderers.get(index) {
                renderer.render_with_options(
                    &entry.frame,
                    time,
                    index == 0,
                    &entry.composite_mode,
                    entry.blend_alpha,
                );
            }
        }
    }

    fn ensure_renderer_count(&self, count: usize) -> bool {
        let mut renderers = self.renderers.borrow_mut();
        while renderers.len() < count {
            let Ok(renderer) =
                RustyMilkWebGlRenderer::new(self.gl.clone(), self.texture_assets.clone())
            else {
                return false;
            };
            renderers.push(renderer);
        }
        true
    }
}

impl RustyMilkWebGlRenderer {
    fn new(
        gl: web_sys::WebGl2RenderingContext,
        texture_assets: Rc<RefCell<BTreeMap<String, String>>>,
    ) -> Result<Self, JsValue> {
        let vertex_shader = compile_rustymilk_shader(
            &gl,
            web_sys::WebGl2RenderingContext::VERTEX_SHADER,
            RUSTYMILK_WEBGL_VERTEX_SHADER,
        )?;
        let fragment_shader = compile_rustymilk_shader(
            &gl,
            web_sys::WebGl2RenderingContext::FRAGMENT_SHADER,
            RUSTYMILK_WEBGL_FRAGMENT_SHADER,
        )?;
        let program = link_rustymilk_program(&gl, &vertex_shader, &fragment_shader)?;
        gl.use_program(Some(&program));
        let primitive_vertex_shader = compile_rustymilk_shader(
            &gl,
            web_sys::WebGl2RenderingContext::VERTEX_SHADER,
            RUSTYMILK_PRIMITIVE_VERTEX_SHADER,
        )?;
        let primitive_fragment_shader = compile_rustymilk_shader(
            &gl,
            web_sys::WebGl2RenderingContext::FRAGMENT_SHADER,
            RUSTYMILK_PRIMITIVE_FRAGMENT_SHADER,
        )?;
        let primitive_program =
            link_rustymilk_program(&gl, &primitive_vertex_shader, &primitive_fragment_shader)?;
        let textured_vertex_shader = compile_rustymilk_shader(
            &gl,
            web_sys::WebGl2RenderingContext::VERTEX_SHADER,
            RUSTYMILK_TEXTURED_VERTEX_SHADER,
        )?;
        let textured_fragment_shader = compile_rustymilk_shader(
            &gl,
            web_sys::WebGl2RenderingContext::FRAGMENT_SHADER,
            RUSTYMILK_TEXTURED_FRAGMENT_SHADER,
        )?;
        let textured_program =
            link_rustymilk_program(&gl, &textured_vertex_shader, &textured_fragment_shader)?;
        let warp_vertex_shader = compile_rustymilk_shader(
            &gl,
            web_sys::WebGl2RenderingContext::VERTEX_SHADER,
            RUSTYMILK_WARP_GRID_VERTEX_SHADER,
        )?;
        let warp_fragment_shader = compile_rustymilk_shader(
            &gl,
            web_sys::WebGl2RenderingContext::FRAGMENT_SHADER,
            RUSTYMILK_WARP_GRID_FRAGMENT_SHADER,
        )?;
        let warp_program = link_rustymilk_program(&gl, &warp_vertex_shader, &warp_fragment_shader)?;

        let buffer = gl
            .create_buffer()
            .ok_or_else(|| JsValue::from_str("WebGL buffer allocation failed"))?;
        gl.bind_buffer(web_sys::WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
        let vertices =
            js_sys::Float32Array::from(&[-1.0_f32, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 1.0][..]);
        gl.buffer_data_with_array_buffer_view(
            web_sys::WebGl2RenderingContext::ARRAY_BUFFER,
            &vertices,
            web_sys::WebGl2RenderingContext::STATIC_DRAW,
        );
        let position = gl.get_attrib_location(&program, "position");
        if position >= 0 {
            gl.enable_vertex_attrib_array(position as u32);
            gl.vertex_attrib_pointer_with_i32(
                position as u32,
                2,
                web_sys::WebGl2RenderingContext::FLOAT,
                false,
                0,
                0,
            );
        }
        let primitive_buffer = gl
            .create_buffer()
            .ok_or_else(|| JsValue::from_str("WebGL primitive buffer allocation failed"))?;
        let primitive_color_buffer = gl
            .create_buffer()
            .ok_or_else(|| JsValue::from_str("WebGL primitive color buffer allocation failed"))?;
        let textured_position_buffer = gl
            .create_buffer()
            .ok_or_else(|| JsValue::from_str("WebGL textured position buffer allocation failed"))?;
        let textured_uv_buffer = gl
            .create_buffer()
            .ok_or_else(|| JsValue::from_str("WebGL textured UV buffer allocation failed"))?;
        let procedural_texture = create_rustymilk_procedural_texture(&gl)?;
        let warp_position_buffer = gl
            .create_buffer()
            .ok_or_else(|| JsValue::from_str("WebGL warp position buffer allocation failed"))?;
        let warp_uv_buffer = gl
            .create_buffer()
            .ok_or_else(|| JsValue::from_str("WebGL warp UV buffer allocation failed"))?;
        let feedback_targets = RefCell::new(create_rustymilk_feedback_targets(
            &gl,
            gl.drawing_buffer_width().max(1),
            gl.drawing_buffer_height().max(1),
        )?);

        Ok(Self {
            u_color: gl.get_uniform_location(&program, "u_color"),
            u_counts: gl.get_uniform_location(&program, "u_counts"),
            u_display_only: gl.get_uniform_location(&program, "u_displayOnly"),
            u_feedback: gl.get_uniform_location(&program, "u_feedback"),
            u_motion: gl.get_uniform_location(&program, "u_motion"),
            u_output_alpha: gl.get_uniform_location(&program, "u_outputAlpha"),
            u_previous_frame: gl.get_uniform_location(&program, "u_previousFrame"),
            u_primitive_point_size: gl
                .get_uniform_location(&primitive_program, "u_primitivePointSize"),
            u_resolution: gl.get_uniform_location(&program, "u_resolution"),
            u_textured_alpha: gl.get_uniform_location(&textured_program, "u_alpha"),
            u_textured_sampler: gl.get_uniform_location(&textured_program, "u_texture"),
            u_textured_tint: gl.get_uniform_location(&textured_program, "u_tint"),
            u_time: gl.get_uniform_location(&program, "u_time"),
            u_warp_color: gl.get_uniform_location(&warp_program, "u_color"),
            u_warp_feedback: gl.get_uniform_location(&warp_program, "u_feedback"),
            u_warp_output_alpha: gl.get_uniform_location(&warp_program, "u_outputAlpha"),
            u_warp_previous_frame: gl.get_uniform_location(&warp_program, "u_previousFrame"),
            warp_position_buffer,
            warp_program,
            warp_uv_buffer,
            buffer,
            feedback_targets,
            gl,
            named_textures: RefCell::new(BTreeMap::new()),
            primitive_buffer,
            primitive_color_buffer,
            primitive_program,
            procedural_texture,
            program,
            textured_position_buffer,
            textured_program,
            textured_uv_buffer,
            texture_assets,
            translated_program: RefCell::new(None),
        })
    }

    fn render_with_options(
        &self,
        frame: &RustyMilkFrame,
        time: f64,
        clear_screen: bool,
        composite_mode: &str,
        output_alpha: f64,
    ) {
        self.gl.use_program(Some(&self.program));
        self.gl.bind_buffer(
            web_sys::WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&self.buffer),
        );
        let drawing_width = self.gl.drawing_buffer_width().max(1);
        let drawing_height = self.gl.drawing_buffer_height().max(1);
        let mut targets = self.feedback_targets.borrow_mut();
        if targets.width != drawing_width || targets.height != drawing_height {
            if let Ok(next_targets) =
                create_rustymilk_feedback_targets(&self.gl, drawing_width, drawing_height)
            {
                *targets = next_targets;
            }
        }
        let read_index = targets.read_index;
        let write_index = 1 - read_index;

        self.gl
            .active_texture(web_sys::WebGl2RenderingContext::TEXTURE0);
        self.gl.bind_texture(
            web_sys::WebGl2RenderingContext::TEXTURE_2D,
            Some(&targets.targets[read_index].texture),
        );
        self.gl.bind_framebuffer(
            web_sys::WebGl2RenderingContext::FRAMEBUFFER,
            Some(&targets.targets[write_index].framebuffer),
        );
        if frame.warp_mesh.is_some() {
            self.draw_warp_mesh(frame, drawing_width, drawing_height);
        } else if let Some(program) = self.translated_program_for(&frame.shader_source) {
            self.draw_translated_feedback_quad(
                &program,
                frame,
                time,
                drawing_width,
                drawing_height,
            );
        } else {
            self.draw_feedback_quad(frame, time, drawing_width, drawing_height, false, 1.0);
        }
        self.draw_textured_primitives(frame);
        self.draw_primitives(frame);

        self.gl
            .bind_framebuffer(web_sys::WebGl2RenderingContext::FRAMEBUFFER, None);
        self.gl.bind_texture(
            web_sys::WebGl2RenderingContext::TEXTURE_2D,
            Some(&targets.targets[write_index].texture),
        );
        if clear_screen {
            self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
            self.gl
                .clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);
        }
        let output_alpha = clamp_unit(output_alpha) as f32;
        let should_blend = !clear_screen || output_alpha < 1.0;
        if should_blend {
            self.gl.enable(web_sys::WebGl2RenderingContext::BLEND);
            let (source_factor, destination_factor) =
                rustymilk_webgl_composite_blend_factors(composite_mode);
            self.gl.blend_func(source_factor, destination_factor);
        }
        self.draw_feedback_quad(
            frame,
            time,
            drawing_width,
            drawing_height,
            true,
            output_alpha,
        );
        if should_blend {
            self.gl.disable(web_sys::WebGl2RenderingContext::BLEND);
        }
        targets.read_index = write_index;
    }

    fn draw_warp_mesh(&self, frame: &RustyMilkFrame, drawing_width: i32, drawing_height: i32) {
        let Some(mesh) = frame.warp_mesh.as_ref() else {
            return;
        };
        if mesh.positions.len() < 6 || mesh.positions.len() != mesh.source_uvs.len() {
            return;
        }
        self.gl.use_program(Some(&self.warp_program));
        self.gl.viewport(0, 0, drawing_width, drawing_height);
        self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
        self.gl
            .clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);
        let positions = mesh
            .positions
            .iter()
            .map(|value| *value as f32)
            .collect::<Vec<_>>();
        let source_uvs = mesh
            .source_uvs
            .iter()
            .map(|value| *value as f32)
            .collect::<Vec<_>>();
        let positions_array = js_sys::Float32Array::from(positions.as_slice());
        let source_uvs_array = js_sys::Float32Array::from(source_uvs.as_slice());
        let position = self.gl.get_attrib_location(&self.warp_program, "position");
        let source_uv = self.gl.get_attrib_location(&self.warp_program, "sourceUv");
        self.gl.bind_buffer(
            web_sys::WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&self.warp_position_buffer),
        );
        self.gl.buffer_data_with_array_buffer_view(
            web_sys::WebGl2RenderingContext::ARRAY_BUFFER,
            &positions_array,
            web_sys::WebGl2RenderingContext::DYNAMIC_DRAW,
        );
        if position >= 0 {
            self.gl.enable_vertex_attrib_array(position as u32);
            self.gl.vertex_attrib_pointer_with_i32(
                position as u32,
                2,
                web_sys::WebGl2RenderingContext::FLOAT,
                false,
                0,
                0,
            );
        }
        self.gl.bind_buffer(
            web_sys::WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&self.warp_uv_buffer),
        );
        self.gl.buffer_data_with_array_buffer_view(
            web_sys::WebGl2RenderingContext::ARRAY_BUFFER,
            &source_uvs_array,
            web_sys::WebGl2RenderingContext::DYNAMIC_DRAW,
        );
        if source_uv >= 0 {
            self.gl.enable_vertex_attrib_array(source_uv as u32);
            self.gl.vertex_attrib_pointer_with_i32(
                source_uv as u32,
                2,
                web_sys::WebGl2RenderingContext::FLOAT,
                false,
                0,
                0,
            );
        }
        let (r, g, b) = frame.wave_color;
        self.gl.uniform3f(
            self.u_warp_color.as_ref(),
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
        );
        self.gl.uniform1f(
            self.u_warp_feedback.as_ref(),
            (1.0 - frame.background_alpha).clamp(0.0, 0.985) as f32,
        );
        self.gl.uniform1f(self.u_warp_output_alpha.as_ref(), 1.0);
        self.gl.uniform1i(self.u_warp_previous_frame.as_ref(), 0);
        self.gl.draw_arrays(
            web_sys::WebGl2RenderingContext::TRIANGLES,
            0,
            (positions.len() / 2) as i32,
        );
    }

    fn translated_program_for(&self, source: &str) -> Option<web_sys::WebGlProgram> {
        if source.trim().is_empty() {
            return None;
        }
        if let Some(cached) = self.translated_program.borrow().as_ref() {
            if cached.source == source {
                return Some(cached.program.clone());
            }
        }
        let vertex_shader = compile_rustymilk_shader(
            &self.gl,
            web_sys::WebGl2RenderingContext::VERTEX_SHADER,
            RUSTYMILK_TRANSLATED_VERTEX_SHADER,
        )
        .ok()?;
        let fragment_shader = compile_rustymilk_shader(
            &self.gl,
            web_sys::WebGl2RenderingContext::FRAGMENT_SHADER,
            source,
        )
        .ok()?;
        let program = link_rustymilk_program(&self.gl, &vertex_shader, &fragment_shader).ok()?;
        *self.translated_program.borrow_mut() = Some(RustyMilkTranslatedProgram {
            program: program.clone(),
            source: source.to_string(),
        });
        Some(program)
    }

    fn set_translated_uniforms(
        &self,
        program: &web_sys::WebGlProgram,
        frame: &RustyMilkFrame,
        time: f64,
        drawing_width: i32,
        drawing_height: i32,
    ) {
        let (r, g, b) = frame.wave_color;
        let feedback = (1.0 - frame.background_alpha).clamp(0.0, 0.985) as f32;
        let uniform = |name: &str| self.gl.get_uniform_location(program, name);
        self.gl.uniform3f(
            uniform("color").as_ref(),
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
        );
        self.gl.uniform1i(uniform("previousFrame").as_ref(), 0);
        for index in 0..4 {
            let texture_unit = index + 2;
            self.gl
                .active_texture(web_sys::WebGl2RenderingContext::TEXTURE0 + texture_unit as u32);
            let texture = frame
                .shader_texture_samplers
                .get(index)
                .and_then(|name| self.named_texture_for(name))
                .unwrap_or_else(|| self.procedural_texture.clone());
            self.gl
                .bind_texture(web_sys::WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
            self.gl.uniform1i(
                uniform(&format!("shaderTexture{index}")).as_ref(),
                texture_unit as i32,
            );
        }
        self.gl
            .active_texture(web_sys::WebGl2RenderingContext::TEXTURE0);
        self.gl.uniform1f(uniform("feedback").as_ref(), feedback);
        self.gl.uniform1f(uniform("outputAlpha").as_ref(), 1.0);
        self.gl.uniform1f(uniform("time").as_ref(), time as f32);
        self.gl.uniform1f(uniform("sampleRate").as_ref(), 44_100.0);
        self.gl.uniform2f(
            uniform("resolution").as_ref(),
            drawing_width as f32,
            drawing_height as f32,
        );
        self.gl.uniform2f(
            uniform("pixelSize").as_ref(),
            1.0 / drawing_width.max(1) as f32,
            1.0 / drawing_height.max(1) as f32,
        );
        self.gl.uniform1f(
            uniform("aspect").as_ref(),
            drawing_width as f32 / drawing_height.max(1) as f32,
        );
        self.gl.uniform4f(
            uniform("texsize").as_ref(),
            drawing_width as f32,
            drawing_height as f32,
            1.0 / drawing_width.max(1) as f32,
            1.0 / drawing_height.max(1) as f32,
        );
        self.gl
            .uniform1f(uniform("bass").as_ref(), frame.bass as f32);
        self.gl
            .uniform1f(uniform("bass_att").as_ref(), frame.bass as f32);
        self.gl.uniform1f(uniform("mid").as_ref(), frame.mid as f32);
        self.gl
            .uniform1f(uniform("mid_att").as_ref(), frame.mid as f32);
        self.gl
            .uniform1f(uniform("treb").as_ref(), frame.treble as f32);
        self.gl
            .uniform1f(uniform("treb_att").as_ref(), frame.treble as f32);
        let fft = frame
            .fft_bins
            .iter()
            .map(|value| *value as f32)
            .collect::<Vec<_>>();
        let waveform = frame
            .waveform_bins
            .iter()
            .map(|value| *value as f32)
            .collect::<Vec<_>>();
        self.gl
            .uniform1fv_with_f32_array(uniform("fftBins").as_ref(), &fft);
        self.gl
            .uniform1fv_with_f32_array(uniform("waveformBins").as_ref(), &waveform);
        for (index, value) in frame.q_registers.iter().enumerate() {
            self.gl
                .uniform1f(uniform(&format!("q{}", index + 1)).as_ref(), *value as f32);
        }
    }

    fn draw_translated_feedback_quad(
        &self,
        program: &web_sys::WebGlProgram,
        frame: &RustyMilkFrame,
        time: f64,
        drawing_width: i32,
        drawing_height: i32,
    ) {
        self.gl.use_program(Some(program));
        self.gl.bind_buffer(
            web_sys::WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&self.buffer),
        );
        let position = self.gl.get_attrib_location(program, "position");
        if position >= 0 {
            self.gl.enable_vertex_attrib_array(position as u32);
            self.gl.vertex_attrib_pointer_with_i32(
                position as u32,
                2,
                web_sys::WebGl2RenderingContext::FLOAT,
                false,
                0,
                0,
            );
        }
        self.gl.viewport(0, 0, drawing_width, drawing_height);
        self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
        self.gl
            .clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);
        self.set_translated_uniforms(program, frame, time, drawing_width, drawing_height);
        self.gl
            .draw_arrays(web_sys::WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4);
    }

    fn draw_feedback_quad(
        &self,
        frame: &RustyMilkFrame,
        time: f64,
        drawing_width: i32,
        drawing_height: i32,
        display_only: bool,
        output_alpha: f32,
    ) {
        self.gl.use_program(Some(&self.program));
        self.gl.bind_buffer(
            web_sys::WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&self.buffer),
        );
        let position = self.gl.get_attrib_location(&self.program, "position");
        if position >= 0 {
            self.gl.enable_vertex_attrib_array(position as u32);
            self.gl.vertex_attrib_pointer_with_i32(
                position as u32,
                2,
                web_sys::WebGl2RenderingContext::FLOAT,
                false,
                0,
                0,
            );
        }
        self.gl.viewport(0, 0, drawing_width, drawing_height);
        if !display_only {
            self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
            self.gl
                .clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);
        }
        let (r, g, b) = frame.wave_color;
        let feedback = (1.0 - frame.background_alpha).clamp(0.0, 0.985) as f32;
        self.gl.uniform1i(self.u_previous_frame.as_ref(), 0);
        self.gl.uniform1f(
            self.u_display_only.as_ref(),
            if display_only { 1.0 } else { 0.0 },
        );
        self.gl.uniform1f(self.u_feedback.as_ref(), feedback);
        self.gl
            .uniform1f(self.u_output_alpha.as_ref(), output_alpha);
        self.gl.uniform2f(
            self.u_resolution.as_ref(),
            drawing_width as f32,
            drawing_height as f32,
        );
        self.gl.uniform1f(self.u_time.as_ref(), time as f32);
        self.gl.uniform4f(
            self.u_color.as_ref(),
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            feedback,
        );
        self.gl.uniform4f(
            self.u_motion.as_ref(),
            frame.rotation as f32,
            frame.zoom as f32,
            frame.dx as f32,
            frame.dy as f32,
        );
        self.gl.uniform2f(
            self.u_counts.as_ref(),
            frame.shape_count as f32,
            frame.waveform_count as f32,
        );
        self.gl
            .draw_arrays(web_sys::WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4);
    }

    fn draw_primitives(&self, frame: &RustyMilkFrame) {
        if frame.primitives.is_empty() {
            return;
        }
        self.gl.enable(web_sys::WebGl2RenderingContext::BLEND);
        self.gl.blend_func(
            web_sys::WebGl2RenderingContext::SRC_ALPHA,
            web_sys::WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );
        self.gl.use_program(Some(&self.primitive_program));
        self.gl.bind_buffer(
            web_sys::WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&self.primitive_buffer),
        );
        let position = self
            .gl
            .get_attrib_location(&self.primitive_program, "position");
        let color = self
            .gl
            .get_attrib_location(&self.primitive_program, "color");
        if position >= 0 {
            self.gl.enable_vertex_attrib_array(position as u32);
            self.gl.vertex_attrib_pointer_with_i32(
                position as u32,
                2,
                web_sys::WebGl2RenderingContext::FLOAT,
                false,
                0,
                0,
            );
        }
        for primitive in &frame.primitives {
            if primitive.vertices.len() < 4 {
                continue;
            }
            let vertices = primitive
                .vertices
                .iter()
                .map(|value| *value as f32)
                .collect::<Vec<_>>();
            let vertex_array = js_sys::Float32Array::from(vertices.as_slice());
            self.gl.buffer_data_with_array_buffer_view(
                web_sys::WebGl2RenderingContext::ARRAY_BUFFER,
                &vertex_array,
                web_sys::WebGl2RenderingContext::DYNAMIC_DRAW,
            );
            let vertex_count = vertices.len() / 2;
            let colors = if primitive.vertex_colors.len() == vertex_count * 4 {
                primitive.vertex_colors.clone()
            } else {
                create_repeated_rustymilk_colors(vertex_count, primitive.color)
            };
            let colors = colors.iter().map(|value| *value as f32).collect::<Vec<_>>();
            let color_array = js_sys::Float32Array::from(colors.as_slice());
            self.gl.bind_buffer(
                web_sys::WebGl2RenderingContext::ARRAY_BUFFER,
                Some(&self.primitive_color_buffer),
            );
            self.gl.buffer_data_with_array_buffer_view(
                web_sys::WebGl2RenderingContext::ARRAY_BUFFER,
                &color_array,
                web_sys::WebGl2RenderingContext::DYNAMIC_DRAW,
            );
            if color >= 0 {
                self.gl.enable_vertex_attrib_array(color as u32);
                self.gl.vertex_attrib_pointer_with_i32(
                    color as u32,
                    4,
                    web_sys::WebGl2RenderingContext::FLOAT,
                    false,
                    0,
                    0,
                );
            }
            self.gl.uniform1f(
                self.u_primitive_point_size.as_ref(),
                if primitive.mode == RustyMilkPrimitiveMode::Points {
                    2.0
                } else {
                    1.0
                },
            );
            let mode = match primitive.mode {
                RustyMilkPrimitiveMode::LineStrip => web_sys::WebGl2RenderingContext::LINE_STRIP,
                RustyMilkPrimitiveMode::Lines => web_sys::WebGl2RenderingContext::LINES,
                RustyMilkPrimitiveMode::Points => web_sys::WebGl2RenderingContext::POINTS,
                RustyMilkPrimitiveMode::TriangleFan => {
                    web_sys::WebGl2RenderingContext::TRIANGLE_FAN
                }
                RustyMilkPrimitiveMode::Triangles => web_sys::WebGl2RenderingContext::TRIANGLES,
            };
            self.gl.draw_arrays(mode, 0, vertex_count as i32);
        }
        self.gl.disable(web_sys::WebGl2RenderingContext::BLEND);
        self.gl.use_program(Some(&self.program));
    }

    fn named_texture_for(&self, raw_name: &str) -> Option<web_sys::WebGlTexture> {
        for alias in get_rustymilk_texture_name_aliases(raw_name) {
            if let Some(texture) = self.named_textures.borrow().get(&alias) {
                return Some(texture.clone());
            }
            let data_url = self.texture_assets.borrow().get(&alias).cloned();
            let Some(data_url) = data_url else {
                continue;
            };
            let Ok(texture) = create_rustymilk_texture_from_data_url(&self.gl, &data_url) else {
                continue;
            };
            self.named_textures
                .borrow_mut()
                .insert(alias, texture.clone());
            return Some(texture);
        }
        None
    }

    fn draw_textured_primitives(&self, frame: &RustyMilkFrame) {
        if frame.textured_primitives.is_empty() {
            return;
        }
        self.gl.enable(web_sys::WebGl2RenderingContext::BLEND);
        self.gl.blend_func(
            web_sys::WebGl2RenderingContext::SRC_ALPHA,
            web_sys::WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );
        self.gl.use_program(Some(&self.textured_program));
        self.gl
            .active_texture(web_sys::WebGl2RenderingContext::TEXTURE1);
        self.gl.uniform1i(self.u_textured_sampler.as_ref(), 1);
        let position = self
            .gl
            .get_attrib_location(&self.textured_program, "position");
        let source_uv = self
            .gl
            .get_attrib_location(&self.textured_program, "sourceUv");
        for primitive in &frame.textured_primitives {
            if primitive.vertices.len() < 6 || primitive.vertices.len() != primitive.uvs.len() {
                continue;
            }
            self.gl
                .active_texture(web_sys::WebGl2RenderingContext::TEXTURE1);
            let texture = self
                .named_texture_for(&primitive.texture_name)
                .unwrap_or_else(|| self.procedural_texture.clone());
            self.gl
                .bind_texture(web_sys::WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
            let vertices = primitive
                .vertices
                .iter()
                .map(|value| *value as f32)
                .collect::<Vec<_>>();
            let uvs = primitive
                .uvs
                .iter()
                .map(|value| *value as f32)
                .collect::<Vec<_>>();
            let vertex_array = js_sys::Float32Array::from(vertices.as_slice());
            let uv_array = js_sys::Float32Array::from(uvs.as_slice());
            self.gl.bind_buffer(
                web_sys::WebGl2RenderingContext::ARRAY_BUFFER,
                Some(&self.textured_position_buffer),
            );
            self.gl.buffer_data_with_array_buffer_view(
                web_sys::WebGl2RenderingContext::ARRAY_BUFFER,
                &vertex_array,
                web_sys::WebGl2RenderingContext::DYNAMIC_DRAW,
            );
            if position >= 0 {
                self.gl.enable_vertex_attrib_array(position as u32);
                self.gl.vertex_attrib_pointer_with_i32(
                    position as u32,
                    2,
                    web_sys::WebGl2RenderingContext::FLOAT,
                    false,
                    0,
                    0,
                );
            }
            self.gl.bind_buffer(
                web_sys::WebGl2RenderingContext::ARRAY_BUFFER,
                Some(&self.textured_uv_buffer),
            );
            self.gl.buffer_data_with_array_buffer_view(
                web_sys::WebGl2RenderingContext::ARRAY_BUFFER,
                &uv_array,
                web_sys::WebGl2RenderingContext::DYNAMIC_DRAW,
            );
            if source_uv >= 0 {
                self.gl.enable_vertex_attrib_array(source_uv as u32);
                self.gl.vertex_attrib_pointer_with_i32(
                    source_uv as u32,
                    2,
                    web_sys::WebGl2RenderingContext::FLOAT,
                    false,
                    0,
                    0,
                );
            }
            self.gl.uniform3f(
                self.u_textured_tint.as_ref(),
                primitive.color[0] as f32,
                primitive.color[1] as f32,
                primitive.color[2] as f32,
            );
            self.gl
                .uniform1f(self.u_textured_alpha.as_ref(), primitive.color[3] as f32);
            let draw_mode = match primitive.mode {
                RustyMilkTexturedPrimitiveMode::Quad
                | RustyMilkTexturedPrimitiveMode::TriangleFan => {
                    web_sys::WebGl2RenderingContext::TRIANGLE_FAN
                }
            };
            self.gl
                .draw_arrays(draw_mode, 0, (vertices.len() / 2) as i32);
        }
        self.gl
            .active_texture(web_sys::WebGl2RenderingContext::TEXTURE0);
        self.gl.disable(web_sys::WebGl2RenderingContext::BLEND);
        self.gl.use_program(Some(&self.program));
    }
}

fn rustymilk_webgl_composite_blend_factors(mode: &str) -> (u32, u32) {
    match mode {
        "additive" => (
            web_sys::WebGl2RenderingContext::SRC_ALPHA,
            web_sys::WebGl2RenderingContext::ONE,
        ),
        "screen" => (
            web_sys::WebGl2RenderingContext::ONE,
            web_sys::WebGl2RenderingContext::ONE_MINUS_SRC_COLOR,
        ),
        "multiply" => (
            web_sys::WebGl2RenderingContext::DST_COLOR,
            web_sys::WebGl2RenderingContext::ZERO,
        ),
        _ => (
            web_sys::WebGl2RenderingContext::SRC_ALPHA,
            web_sys::WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        ),
    }
}

fn create_rustymilk_feedback_targets(
    gl: &web_sys::WebGl2RenderingContext,
    width: i32,
    height: i32,
) -> Result<RustyMilkFeedbackTargets, JsValue> {
    let width = width.max(1);
    let height = height.max(1);
    Ok(RustyMilkFeedbackTargets {
        height,
        read_index: 0,
        targets: [
            create_rustymilk_feedback_target(gl, width, height)?,
            create_rustymilk_feedback_target(gl, width, height)?,
        ],
        width,
    })
}

fn create_rustymilk_feedback_target(
    gl: &web_sys::WebGl2RenderingContext,
    width: i32,
    height: i32,
) -> Result<RustyMilkFeedbackTarget, JsValue> {
    let texture = gl
        .create_texture()
        .ok_or_else(|| JsValue::from_str("WebGL feedback texture allocation failed"))?;
    gl.bind_texture(web_sys::WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
    gl.tex_parameteri(
        web_sys::WebGl2RenderingContext::TEXTURE_2D,
        web_sys::WebGl2RenderingContext::TEXTURE_MIN_FILTER,
        web_sys::WebGl2RenderingContext::LINEAR as i32,
    );
    gl.tex_parameteri(
        web_sys::WebGl2RenderingContext::TEXTURE_2D,
        web_sys::WebGl2RenderingContext::TEXTURE_MAG_FILTER,
        web_sys::WebGl2RenderingContext::LINEAR as i32,
    );
    gl.tex_parameteri(
        web_sys::WebGl2RenderingContext::TEXTURE_2D,
        web_sys::WebGl2RenderingContext::TEXTURE_WRAP_S,
        web_sys::WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
    );
    gl.tex_parameteri(
        web_sys::WebGl2RenderingContext::TEXTURE_2D,
        web_sys::WebGl2RenderingContext::TEXTURE_WRAP_T,
        web_sys::WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
    );
    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
        web_sys::WebGl2RenderingContext::TEXTURE_2D,
        0,
        web_sys::WebGl2RenderingContext::RGBA as i32,
        width,
        height,
        0,
        web_sys::WebGl2RenderingContext::RGBA,
        web_sys::WebGl2RenderingContext::UNSIGNED_BYTE,
        None,
    )?;

    let framebuffer = gl
        .create_framebuffer()
        .ok_or_else(|| JsValue::from_str("WebGL feedback framebuffer allocation failed"))?;
    gl.bind_framebuffer(
        web_sys::WebGl2RenderingContext::FRAMEBUFFER,
        Some(&framebuffer),
    );
    gl.framebuffer_texture_2d(
        web_sys::WebGl2RenderingContext::FRAMEBUFFER,
        web_sys::WebGl2RenderingContext::COLOR_ATTACHMENT0,
        web_sys::WebGl2RenderingContext::TEXTURE_2D,
        Some(&texture),
        0,
    );
    let status = gl.check_framebuffer_status(web_sys::WebGl2RenderingContext::FRAMEBUFFER);
    if status != web_sys::WebGl2RenderingContext::FRAMEBUFFER_COMPLETE {
        return Err(JsValue::from_str(
            "WebGL feedback framebuffer is incomplete",
        ));
    }

    Ok(RustyMilkFeedbackTarget {
        framebuffer,
        texture,
    })
}

fn create_rustymilk_procedural_texture(
    gl: &web_sys::WebGl2RenderingContext,
) -> Result<web_sys::WebGlTexture, JsValue> {
    let texture = gl
        .create_texture()
        .ok_or_else(|| JsValue::from_str("WebGL procedural texture allocation failed"))?;
    let mut pixels = Vec::with_capacity(16 * 16 * 4);
    for y in 0..16 {
        for x in 0..16 {
            let checker = if (x / 4 + y / 4) % 2 == 0 { 224 } else { 72 };
            pixels.extend_from_slice(&[checker, 192, 255_u8.saturating_sub(checker / 3), 255]);
        }
    }
    gl.bind_texture(web_sys::WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
    gl.tex_parameteri(
        web_sys::WebGl2RenderingContext::TEXTURE_2D,
        web_sys::WebGl2RenderingContext::TEXTURE_MIN_FILTER,
        web_sys::WebGl2RenderingContext::LINEAR as i32,
    );
    gl.tex_parameteri(
        web_sys::WebGl2RenderingContext::TEXTURE_2D,
        web_sys::WebGl2RenderingContext::TEXTURE_MAG_FILTER,
        web_sys::WebGl2RenderingContext::LINEAR as i32,
    );
    gl.tex_parameteri(
        web_sys::WebGl2RenderingContext::TEXTURE_2D,
        web_sys::WebGl2RenderingContext::TEXTURE_WRAP_S,
        web_sys::WebGl2RenderingContext::REPEAT as i32,
    );
    gl.tex_parameteri(
        web_sys::WebGl2RenderingContext::TEXTURE_2D,
        web_sys::WebGl2RenderingContext::TEXTURE_WRAP_T,
        web_sys::WebGl2RenderingContext::REPEAT as i32,
    );
    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
        web_sys::WebGl2RenderingContext::TEXTURE_2D,
        0,
        web_sys::WebGl2RenderingContext::RGBA as i32,
        16,
        16,
        0,
        web_sys::WebGl2RenderingContext::RGBA,
        web_sys::WebGl2RenderingContext::UNSIGNED_BYTE,
        Some(&pixels),
    )?;
    Ok(texture)
}

fn create_rustymilk_texture_from_data_url(
    gl: &web_sys::WebGl2RenderingContext,
    data_url: &str,
) -> Result<web_sys::WebGlTexture, JsValue> {
    let texture = create_rustymilk_procedural_texture(gl)?;
    let image = web_sys::HtmlImageElement::new()?;
    let gl_for_load = gl.clone();
    let texture_for_load = texture.clone();
    let image_for_load = image.clone();
    let onload = Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new(move |_event| {
        gl_for_load.bind_texture(
            web_sys::WebGl2RenderingContext::TEXTURE_2D,
            Some(&texture_for_load),
        );
        let _ = gl_for_load.tex_image_2d_with_u32_and_u32_and_html_image_element(
            web_sys::WebGl2RenderingContext::TEXTURE_2D,
            0,
            web_sys::WebGl2RenderingContext::RGBA as i32,
            web_sys::WebGl2RenderingContext::RGBA,
            web_sys::WebGl2RenderingContext::UNSIGNED_BYTE,
            &image_for_load,
        );
    }));
    image.set_onload(Some(onload.as_ref().unchecked_ref()));
    onload.forget();
    image.set_src(data_url);
    Ok(texture)
}

fn compile_rustymilk_shader(
    gl: &web_sys::WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<web_sys::WebGlShader, JsValue> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| JsValue::from_str("WebGL shader allocation failed"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);
    if gl
        .get_shader_parameter(&shader, web_sys::WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(JsValue::from_str(
            &gl.get_shader_info_log(&shader)
                .unwrap_or_else(|| "RustyMilk WebGL shader compile failed".to_string()),
        ))
    }
}

fn link_rustymilk_program(
    gl: &web_sys::WebGl2RenderingContext,
    vertex_shader: &web_sys::WebGlShader,
    fragment_shader: &web_sys::WebGlShader,
) -> Result<web_sys::WebGlProgram, JsValue> {
    let program = gl
        .create_program()
        .ok_or_else(|| JsValue::from_str("WebGL program allocation failed"))?;
    gl.attach_shader(&program, vertex_shader);
    gl.attach_shader(&program, fragment_shader);
    gl.link_program(&program);
    if gl
        .get_program_parameter(&program, web_sys::WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(JsValue::from_str(
            &gl.get_program_info_log(&program)
                .unwrap_or_else(|| "RustyMilk WebGL program link failed".to_string()),
        ))
    }
}

const RUSTYMILK_WEBGL_VERTEX_SHADER: &str = r#"#version 300 es
in vec2 position;
out vec2 v_uv;
void main() {
  v_uv = position * 0.5 + vec2(0.5);
  gl_Position = vec4(position, 0.0, 1.0);
}
"#;

const RUSTYMILK_TRANSLATED_VERTEX_SHADER: &str = r#"#version 300 es
in vec2 position;
out vec2 uv;
void main() {
  uv = position * 0.5 + vec2(0.5);
  gl_Position = vec4(position, 0.0, 1.0);
}
"#;

const RUSTYMILK_WEBGL_FRAGMENT_SHADER: &str = r#"#version 300 es
precision highp float;
uniform vec2 u_resolution;
uniform float u_time;
uniform vec4 u_color;
uniform vec4 u_motion;
uniform vec2 u_counts;
uniform sampler2D u_previousFrame;
uniform float u_feedback;
uniform float u_displayOnly;
uniform float u_outputAlpha;
in vec2 v_uv;
out vec4 outColor;

mat2 rotate2d(float angle) {
  float s = sin(angle);
  float c = cos(angle);
  return mat2(c, -s, s, c);
}

void main() {
  vec2 centered = v_uv - vec2(0.5) - u_motion.zw;
  vec2 warped = rotate2d(u_motion.x + u_time * 0.035) * centered / max(u_motion.y, 0.001);
  float radius = length(warped);
  float angle = atan(warped.y, warped.x);
  float rings = 0.5 + 0.5 * sin(radius * 42.0 - u_time * 3.0 + u_counts.x * 0.35);
  float spokes = 0.5 + 0.5 * cos(angle * (5.0 + u_counts.y) + u_time * 1.7);
  float wave = 0.5 + 0.5 * sin((v_uv.x + v_uv.y) * 18.0 + u_time * 4.0);
  float shapePulse = smoothstep(0.24, 0.0, abs(radius - (0.18 + 0.025 * u_counts.x)));
  vec3 tint = mix(u_color.rgb * 0.24, u_color.rgb, max(rings * 0.62, spokes * 0.42));
  tint += vec3(1.0, 0.72, 0.32) * shapePulse * 0.35;
  tint += vec3(0.65, 0.85, 1.0) * wave * 0.08 * max(u_counts.y, 1.0);
  vec2 feedbackUv = v_uv - u_motion.zw * 0.18;
  feedbackUv = (feedbackUv - vec2(0.5)) / max(u_motion.y, 0.001) + vec2(0.5);
  vec3 previous = texture(u_previousFrame, clamp(feedbackUv, vec2(0.001), vec2(0.999))).rgb;
  vec3 composited = mix(tint, previous * 0.996, clamp(u_feedback, 0.0, 0.985));
  outColor = vec4(mix(composited, texture(u_previousFrame, v_uv).rgb, step(0.5, u_displayOnly)), u_outputAlpha);
}
"#;

const RUSTYMILK_PRIMITIVE_VERTEX_SHADER: &str = r#"#version 300 es
in vec2 position;
in vec4 color;
uniform float u_primitivePointSize;
out vec4 v_color;
void main() {
  v_color = color;
  gl_PointSize = u_primitivePointSize;
  gl_Position = vec4(position, 0.0, 1.0);
}
"#;

const RUSTYMILK_PRIMITIVE_FRAGMENT_SHADER: &str = r#"#version 300 es
precision highp float;
in vec4 v_color;
out vec4 outColor;
void main() {
  outColor = v_color;
}
"#;

const RUSTYMILK_TEXTURED_VERTEX_SHADER: &str = r#"#version 300 es
in vec2 position;
in vec2 sourceUv;
out vec2 v_uv;
void main() {
  v_uv = sourceUv;
  gl_Position = vec4(position, 0.0, 1.0);
}
"#;

const RUSTYMILK_TEXTURED_FRAGMENT_SHADER: &str = r#"#version 300 es
precision highp float;
uniform sampler2D u_texture;
uniform vec3 u_tint;
uniform float u_alpha;
in vec2 v_uv;
out vec4 outColor;
void main() {
  vec4 texel = texture(u_texture, v_uv);
  outColor = vec4(texel.rgb * u_tint, texel.a * u_alpha);
}
"#;

const RUSTYMILK_WARP_GRID_VERTEX_SHADER: &str = r#"#version 300 es
in vec2 position;
in vec2 sourceUv;
out vec2 v_sourceUv;
void main() {
  v_sourceUv = sourceUv;
  gl_Position = vec4(position, 0.0, 1.0);
}
"#;

const RUSTYMILK_WARP_GRID_FRAGMENT_SHADER: &str = r#"#version 300 es
precision highp float;
uniform vec3 u_color;
uniform float u_feedback;
uniform float u_outputAlpha;
uniform sampler2D u_previousFrame;
in vec2 v_sourceUv;
out vec4 outColor;
void main() {
  vec3 previous = texture(u_previousFrame, clamp(v_sourceUv, vec2(0.0), vec2(1.0))).rgb;
  vec3 base = mix(u_color * 0.18, previous, clamp(u_feedback, 0.0, 0.985));
  outColor = vec4(base, u_outputAlpha);
}
"#;

fn render_rustymilk_canvas_frame(
    context: &web_sys::CanvasRenderingContext2d,
    canvas: &web_sys::HtmlCanvasElement,
    frame: &RustyMilkFrame,
    time: f64,
) {
    let width = canvas.width() as f64;
    let height = canvas.height() as f64;
    context.set_fill_style_str(&format!("rgba(9, 13, 18, {:.3})", frame.background_alpha));
    context.fill_rect(0.0, 0.0, width, height);

    let (r, g, b) = frame.wave_color;
    let center_x = width * (0.5 + frame.dx);
    let center_y = height * (0.5 + frame.dy);
    let base = width.min(height) * frame.wave_radius * frame.zoom;
    context.save();
    let _ = context.translate(center_x, center_y);
    let _ = context.rotate(frame.rotation + time * 0.08);
    context.set_stroke_style_str(&format!("rgba({r}, {g}, {b}, 0.92)"));
    context.set_line_width(2.0);
    context.begin_path();
    for index in 0..192 {
        let unit = index as f64 / 192.0;
        let angle = unit * std::f64::consts::TAU;
        let radius = base
            * (0.72
                + 0.12 * (time * 1.9 + angle * 3.0).sin()
                + 0.08 * (time * 2.7 + angle * 7.0).cos());
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        if index == 0 {
            context.move_to(x, y);
        } else {
            context.line_to(x, y);
        }
    }
    context.close_path();
    context.stroke();

    context.set_fill_style_str(&format!("rgba({r}, {g}, {b}, 0.14)"));
    context.fill();

    context.set_stroke_style_str("rgba(255, 209, 102, 0.34)");
    context.set_line_width(1.0);
    for ring in 1..=4 {
        context.begin_path();
        let ring_radius = base * ring as f64 * 0.18 * (1.0 + 0.03 * (time + ring as f64).sin());
        let _ = context.arc(0.0, 0.0, ring_radius, 0.0, std::f64::consts::TAU);
        context.stroke();
    }
    for shape in 0..frame.shape_count.min(6) {
        let sides = 3 + (shape % 5);
        let radius = base * (0.16 + shape as f64 * 0.035);
        let spin = time * (0.4 + shape as f64 * 0.09);
        context.begin_path();
        for side in 0..=sides {
            let angle = spin + side as f64 / sides as f64 * std::f64::consts::TAU;
            let x = radius * angle.cos();
            let y = radius * angle.sin();
            if side == 0 {
                context.move_to(x, y);
            } else {
                context.line_to(x, y);
            }
        }
        context.set_stroke_style_str(&format!("rgba({r}, {g}, {b}, 0.42)"));
        context.stroke();
    }
    context.restore();

    context.set_stroke_style_str("rgba(216, 232, 225, 0.36)");
    context.set_line_width(1.5);
    for wave in 0..frame.waveform_count.max(1).min(3) {
        context.begin_path();
        for index in 0..128 {
            let x = width * index as f64 / 127.0;
            let amp = (time * (5.0 + wave as f64) + index as f64 * 0.19).sin()
                * (0.18 + 0.08 * wave as f64 + 0.18 * (time * 2.0).sin().abs());
            let baseline = 0.78 - wave as f64 * 0.16;
            let y = height * (baseline + amp * 0.28);
            if index == 0 {
                context.move_to(x, y);
            } else {
                context.line_to(x, y);
            }
        }
        context.stroke();
    }
    render_rustymilk_canvas_primitives(context, width, height, frame);
    render_rustymilk_canvas_textured_primitives(context, width, height, frame);
}

fn render_rustymilk_canvas_overlay_frame(
    context: &web_sys::CanvasRenderingContext2d,
    canvas: &web_sys::HtmlCanvasElement,
    frame: &RustyMilkFrame,
    time: f64,
) {
    let width = canvas.width() as f64;
    let height = canvas.height() as f64;
    let (r, g, b) = frame.wave_color;
    context.save();
    let _ = context.translate(width * (0.5 + frame.dx), height * (0.5 + frame.dy));
    let _ = context.rotate(frame.rotation + time * 0.05);
    context.set_stroke_style_str(&format!("rgba({r}, {g}, {b}, 0.68)"));
    context.set_line_width(1.5);
    let base = width.min(height) * frame.wave_radius * frame.zoom;
    for ring in 0..frame.shape_count.max(1).min(5) {
        context.begin_path();
        let radius = base * (0.22 + ring as f64 * 0.11);
        let _ = context.arc(0.0, 0.0, radius, 0.0, std::f64::consts::TAU);
        context.stroke();
    }
    context.restore();
    render_rustymilk_canvas_primitives(context, width, height, frame);
    render_rustymilk_canvas_textured_primitives(context, width, height, frame);
}

fn rustymilk_canvas_color(color: [f64; 4]) -> String {
    format!(
        "rgba({:.0}, {:.0}, {:.0}, {:.3})",
        clamp_unit(color[0]) * 255.0,
        clamp_unit(color[1]) * 255.0,
        clamp_unit(color[2]) * 255.0,
        clamp_unit(color[3])
    )
}

fn rustymilk_clip_to_canvas(vertex: &[f64], width: f64, height: f64) -> (f64, f64) {
    (
        (vertex.first().copied().unwrap_or_default() * 0.5 + 0.5) * width,
        (1.0 - (vertex.get(1).copied().unwrap_or_default() * 0.5 + 0.5)) * height,
    )
}

fn render_rustymilk_canvas_primitives(
    context: &web_sys::CanvasRenderingContext2d,
    width: f64,
    height: f64,
    frame: &RustyMilkFrame,
) {
    for primitive in &frame.primitives {
        if primitive.vertices.len() < 4 {
            continue;
        }
        let color = rustymilk_canvas_color(primitive.color);
        match primitive.mode {
            RustyMilkPrimitiveMode::LineStrip => {
                context.begin_path();
                for (index, vertex) in primitive.vertices.chunks(2).enumerate() {
                    let (x, y) = rustymilk_clip_to_canvas(vertex, width, height);
                    if index == 0 {
                        context.move_to(x, y);
                    } else {
                        context.line_to(x, y);
                    }
                }
                context.set_stroke_style_str(&color);
                context.set_line_width(1.5);
                context.stroke();
            }
            RustyMilkPrimitiveMode::Lines => {
                context.set_stroke_style_str(&color);
                context.set_line_width(1.0);
                for line in primitive.vertices.chunks(4) {
                    if line.len() < 4 {
                        continue;
                    }
                    let (x1, y1) = rustymilk_clip_to_canvas(&line[0..2], width, height);
                    let (x2, y2) = rustymilk_clip_to_canvas(&line[2..4], width, height);
                    context.begin_path();
                    context.move_to(x1, y1);
                    context.line_to(x2, y2);
                    context.stroke();
                }
            }
            RustyMilkPrimitiveMode::Points => {
                context.set_fill_style_str(&color);
                for vertex in primitive.vertices.chunks(2) {
                    let (x, y) = rustymilk_clip_to_canvas(vertex, width, height);
                    context.begin_path();
                    let _ = context.arc(x, y, 2.0, 0.0, std::f64::consts::TAU);
                    context.fill();
                }
            }
            RustyMilkPrimitiveMode::TriangleFan => {
                context.begin_path();
                for (index, vertex) in primitive.vertices.chunks(2).enumerate() {
                    let (x, y) = rustymilk_clip_to_canvas(vertex, width, height);
                    if index == 0 {
                        context.move_to(x, y);
                    } else {
                        context.line_to(x, y);
                    }
                }
                context.close_path();
                context.set_fill_style_str(&color);
                context.fill();
            }
            RustyMilkPrimitiveMode::Triangles => {
                context.set_fill_style_str(&color);
                for triangle in primitive.vertices.chunks(6) {
                    if triangle.len() < 6 {
                        continue;
                    }
                    context.begin_path();
                    let (x1, y1) = rustymilk_clip_to_canvas(&triangle[0..2], width, height);
                    let (x2, y2) = rustymilk_clip_to_canvas(&triangle[2..4], width, height);
                    let (x3, y3) = rustymilk_clip_to_canvas(&triangle[4..6], width, height);
                    context.move_to(x1, y1);
                    context.line_to(x2, y2);
                    context.line_to(x3, y3);
                    context.close_path();
                    context.fill();
                }
            }
        }
    }
}

fn render_rustymilk_canvas_textured_primitives(
    context: &web_sys::CanvasRenderingContext2d,
    width: f64,
    height: f64,
    frame: &RustyMilkFrame,
) {
    for primitive in &frame.textured_primitives {
        if primitive.vertices.len() < 6 {
            continue;
        }
        context.begin_path();
        for (index, vertex) in primitive.vertices.chunks(2).enumerate() {
            let (x, y) = rustymilk_clip_to_canvas(vertex, width, height);
            if index == 0 {
                context.move_to(x, y);
            } else {
                context.line_to(x, y);
            }
        }
        context.close_path();
        context.set_fill_style_str(&rustymilk_canvas_color(primitive.color));
        context.fill();
        context.set_stroke_style_str("rgba(255, 255, 255, 0.22)");
        context.set_line_width(1.0);
        context.stroke();
    }
}

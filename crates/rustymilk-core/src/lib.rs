use std::collections::BTreeMap;

pub struct RustyMilkPreset {
    pub decay: f64,
    pub rot: f64,
    pub wave_a: f64,
    pub wave_b: f64,
    pub wave_g: f64,
    pub wave_r: f64,
    pub wave_scale: f64,
    pub zoom: f64,
}

impl Default for RustyMilkPreset {
    fn default() -> Self {
        Self {
            decay: 0.89,
            rot: 0.012,
            wave_a: 0.86,
            wave_b: 0.92,
            wave_g: 0.58,
            wave_r: 0.16,
            wave_scale: 1.25,
            zoom: 1.02,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RustyMilkFrame {
    pub background_alpha: f64,
    pub bass: f64,
    pub dx: f64,
    pub dy: f64,
    pub fft_bins: [f64; 64],
    pub mid: f64,
    pub primitives: Vec<RustyMilkPrimitive>,
    pub q_registers: [f64; 64],
    pub shape_count: usize,
    pub shader_source: String,
    pub shader_texture_samplers: Vec<String>,
    pub textured_primitives: Vec<RustyMilkTexturedPrimitive>,
    pub rotation: f64,
    pub treble: f64,
    pub wave_color: (u8, u8, u8),
    pub waveform_bins: [f64; 64],
    pub wave_radius: f64,
    pub waveform_count: usize,
    pub warp_mesh: Option<RustyMilkWarpMesh>,
    pub zoom: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RustyMilkCompositeFrame {
    pub blend_alpha: f64,
    pub composite_mode: String,
    pub frame: RustyMilkFrame,
    pub index: usize,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RustyMilkFrameSet {
    pub entries: Vec<RustyMilkCompositeFrame>,
    pub preset_count: usize,
    pub title: String,
    pub transition_mode: String,
    pub transition_seconds: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RustyMilkInputState {
    pub mouse_down: f64,
    pub mouse_dx: f64,
    pub mouse_dy: f64,
    pub mouse_x: f64,
    pub mouse_y: f64,
}

impl Default for RustyMilkInputState {
    fn default() -> Self {
        Self {
            mouse_down: 0.0,
            mouse_dx: 0.0,
            mouse_dy: 0.0,
            mouse_x: 0.5,
            mouse_y: 0.5,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RustyMilkPrimitiveMode {
    LineStrip,
    Lines,
    Points,
    TriangleFan,
    Triangles,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RustyMilkPrimitive {
    pub color: [f64; 4],
    pub mode: RustyMilkPrimitiveMode,
    pub vertex_colors: Vec<f64>,
    pub vertices: Vec<f64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RustyMilkTexturedPrimitiveMode {
    Quad,
    TriangleFan,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RustyMilkTexturedPrimitive {
    pub color: [f64; 4],
    pub mode: RustyMilkTexturedPrimitiveMode,
    pub texture_name: String,
    pub uvs: Vec<f64>,
    pub vertices: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RustyMilkWarpMesh {
    pub positions: Vec<f64>,
    pub source_uvs: Vec<f64>,
}

pub fn clamp_unit(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn clamp_range(value: f64, min: f64, max: f64) -> f64 {
    value.clamp(min, max)
}

pub fn parse_rustymilk_preset(source: &str) -> RustyMilkPreset {
    let mut preset = RustyMilkPreset::default();
    for line in source.lines() {
        let Some((raw_key, raw_value)) = line.split_once('=') else {
            continue;
        };
        let key = raw_key.trim().to_ascii_lowercase();
        let Ok(value) = raw_value.trim().parse::<f64>() else {
            continue;
        };
        match key.as_str() {
            "decay" => preset.decay = clamp_range(value, 0.5, 0.99),
            "rot" => preset.rot = clamp_range(value, -0.5, 0.5),
            "wave_a" => preset.wave_a = clamp_unit(value),
            "wave_b" => preset.wave_b = clamp_unit(value),
            "wave_g" => preset.wave_g = clamp_unit(value),
            "wave_r" => preset.wave_r = clamp_unit(value),
            "wave_scale" => preset.wave_scale = clamp_range(value, 0.2, 3.0),
            "zoom" => preset.zoom = clamp_range(value, 0.5, 1.8),
            _ => {}
        }
    }
    preset
}

pub fn rustymilk_frame(
    preset: &RustyMilkPreset,
    time_seconds: f64,
    bass: f64,
    mid: f64,
    treble: f64,
) -> RustyMilkFrame {
    let bass = clamp_unit(bass);
    let mid = clamp_unit(mid);
    let treble = clamp_unit(treble);
    let pulse = (time_seconds * 1.7).sin() * 0.5 + 0.5;
    RustyMilkFrame {
        background_alpha: clamp_range(1.0 - preset.decay, 0.01, 0.5),
        bass,
        dx: 0.0,
        dy: 0.0,
        fft_bins: [0.0; 64],
        mid,
        primitives: Vec::new(),
        q_registers: [0.0; 64],
        shape_count: 0,
        shader_source: String::new(),
        shader_texture_samplers: Vec::new(),
        textured_primitives: Vec::new(),
        rotation: preset.rot + (time_seconds * 0.37).sin() * 0.035 + (treble - 0.5) * 0.05,
        treble,
        wave_color: (
            ((preset.wave_r + bass * 0.35) * 255.0).min(255.0) as u8,
            ((preset.wave_g + mid * 0.30) * 255.0).min(255.0) as u8,
            ((preset.wave_b + treble * 0.25) * 255.0).min(255.0) as u8,
        ),
        waveform_bins: [0.0; 64],
        wave_radius: clamp_range(
            0.18 + preset.wave_scale * 0.09 + bass * 0.12 + pulse * 0.04,
            0.12,
            0.68,
        ),
        waveform_count: 0,
        warp_mesh: None,
        zoom: clamp_range(preset.zoom + (pulse - 0.5) * 0.035, 0.5, 1.8),
    }
}

fn rustymilk_scope_number(
    scope: &BTreeMap<String, RustyMilkValue>,
    key: &str,
    fallback: f64,
) -> f64 {
    scope
        .get(key)
        .and_then(RustyMilkValue::as_number)
        .filter(|value| value.is_finite())
        .unwrap_or(fallback)
}

fn rustymilk_base_number(
    values: &BTreeMap<String, RustyMilkValue>,
    key: &str,
    fallback: f64,
) -> f64 {
    values
        .get(key)
        .and_then(RustyMilkValue::as_number)
        .filter(|value| value.is_finite())
        .unwrap_or(fallback)
}

fn rustymilk_base_number_any(
    values: &BTreeMap<String, RustyMilkValue>,
    keys: &[&str],
    fallback: f64,
) -> f64 {
    keys.iter()
        .find_map(|key| {
            values
                .get(*key)
                .and_then(RustyMilkValue::as_number)
                .filter(|value| value.is_finite())
        })
        .unwrap_or(fallback)
}

fn rustymilk_composite_alpha(preset: &RustyMilkPresetDocument, index: usize) -> f64 {
    if index == 0 {
        return 1.0;
    }
    clamp_unit(rustymilk_base_number_any(
        &preset.base_values,
        &["blend_alpha", "blendalpha", "composite_alpha", "alpha"],
        0.5,
    ))
}

fn normalize_rustymilk_composite_mode(value: &str) -> String {
    let normalized = value
        .trim()
        .to_ascii_lowercase()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>();
    match normalized.as_str() {
        "add" | "additive" | "plus" => "additive".to_string(),
        "screen" => "screen".to_string(),
        "multiply" | "mult" => "multiply".to_string(),
        _ => "alpha".to_string(),
    }
}

fn rustymilk_composite_mode(preset: &RustyMilkPresetDocument, index: usize) -> String {
    if index == 0 {
        return "alpha".to_string();
    }
    let mode = [
        "blend_mode",
        "blendmode",
        "composite_mode",
        "compositemode",
        "mode",
    ]
    .iter()
    .find_map(|key| preset.base_values.get(*key).map(RustyMilkValue::as_text))
    .unwrap_or_default();
    normalize_rustymilk_composite_mode(&mode)
}

fn rustymilk_transition_seconds(parsed: &RustyMilkPresetSet) -> f64 {
    parsed
        .presets
        .first()
        .map(|preset| {
            rustymilk_base_number_any(
                &preset.base_values,
                &[
                    "transition_seconds",
                    "transition_time",
                    "transitiontime",
                    "blend_seconds",
                    "blend_time",
                    "blendtime",
                ],
                1.25,
            )
            .max(0.0)
        })
        .unwrap_or(1.25)
}

fn rustymilk_transition_mode(parsed: &RustyMilkPresetSet) -> String {
    let mode = parsed
        .presets
        .first()
        .and_then(|preset| {
            [
                "transition_mode",
                "transitionmode",
                "transition_style",
                "transitionstyle",
                "blend_transition",
            ]
            .iter()
            .find_map(|key| preset.base_values.get(*key).map(RustyMilkValue::as_text))
        })
        .unwrap_or_else(|| "crossfade".to_string());
    let normalized = mode
        .trim()
        .to_ascii_lowercase()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>();
    match normalized.as_str() {
        "cut" | "instant" | "none" => "cut".to_string(),
        "additive" | "add" => "additive".to_string(),
        _ => "crossfade".to_string(),
    }
}

fn create_rustymilk_scope(
    preset: &RustyMilkPresetDocument,
    time_seconds: f64,
    bass: f64,
    mid: f64,
    treble: f64,
) -> BTreeMap<String, RustyMilkValue> {
    let mut scope = preset.base_values.clone();
    for index in 1..=64 {
        scope
            .entry(format!("q{index}"))
            .or_insert(RustyMilkValue::Number(0.0));
    }
    scope.insert("bass".to_string(), RustyMilkValue::Number(bass));
    scope.insert("bass_att".to_string(), RustyMilkValue::Number(bass));
    scope.insert("mid".to_string(), RustyMilkValue::Number(mid));
    scope.insert("mid_att".to_string(), RustyMilkValue::Number(mid));
    scope.insert("treb".to_string(), RustyMilkValue::Number(treble));
    scope.insert("treb_att".to_string(), RustyMilkValue::Number(treble));
    scope.insert("time".to_string(), RustyMilkValue::Number(time_seconds));
    scope.insert(
        "frame".to_string(),
        RustyMilkValue::Number((time_seconds * 60.0).floor()),
    );
    scope.insert("fps".to_string(), RustyMilkValue::Number(60.0));
    scope.insert(
        "wave_r".to_string(),
        RustyMilkValue::Number(rustymilk_base_number(&preset.base_values, "wave_r", 0.7)),
    );
    scope.insert(
        "wave_g".to_string(),
        RustyMilkValue::Number(rustymilk_base_number(&preset.base_values, "wave_g", 0.7)),
    );
    scope.insert(
        "wave_b".to_string(),
        RustyMilkValue::Number(rustymilk_base_number(&preset.base_values, "wave_b", 0.7)),
    );
    update_rustymilk_scope_input(&mut scope, RustyMilkInputState::default());
    scope
}

fn update_rustymilk_scope_audio(
    scope: &mut BTreeMap<String, RustyMilkValue>,
    time_seconds: f64,
    frame_index: f64,
    bass: f64,
    mid: f64,
    treble: f64,
    waveform: &[f64],
    spectrum: &[f64],
) {
    scope.insert("bass".to_string(), RustyMilkValue::Number(bass));
    scope.insert("bass_att".to_string(), RustyMilkValue::Number(bass));
    scope.insert("mid".to_string(), RustyMilkValue::Number(mid));
    scope.insert("mid_att".to_string(), RustyMilkValue::Number(mid));
    scope.insert("treb".to_string(), RustyMilkValue::Number(treble));
    scope.insert("treb_att".to_string(), RustyMilkValue::Number(treble));
    scope.insert("time".to_string(), RustyMilkValue::Number(time_seconds));
    scope.insert("frame".to_string(), RustyMilkValue::Number(frame_index));
    scope.insert("fps".to_string(), RustyMilkValue::Number(60.0));
    scope.insert("sample_rate".to_string(), RustyMilkValue::Number(44_100.0));
    scope.insert("samplerate".to_string(), RustyMilkValue::Number(44_100.0));
    scope.insert("canvas_width".to_string(), RustyMilkValue::Number(1.0));
    scope.insert("canvas_height".to_string(), RustyMilkValue::Number(1.0));
    scope.insert("aspect".to_string(), RustyMilkValue::Number(1.0));
    if !waveform.is_empty() {
        scope.insert(
            "waveform_data".to_string(),
            RustyMilkValue::Text(rustymilk_sample_text(waveform)),
        );
    }
    if !spectrum.is_empty() {
        scope.insert(
            "frequency_data".to_string(),
            RustyMilkValue::Text(rustymilk_sample_text(spectrum)),
        );
    }
}

fn update_rustymilk_scope_input(
    scope: &mut BTreeMap<String, RustyMilkValue>,
    input: RustyMilkInputState,
) {
    scope.insert(
        "mouse_down".to_string(),
        RustyMilkValue::Number(input.mouse_down),
    );
    scope.insert(
        "mouse_dx".to_string(),
        RustyMilkValue::Number(input.mouse_dx),
    );
    scope.insert(
        "mouse_dy".to_string(),
        RustyMilkValue::Number(input.mouse_dy),
    );
    scope.insert("mouse_x".to_string(), RustyMilkValue::Number(input.mouse_x));
    scope.insert("mouse_y".to_string(), RustyMilkValue::Number(input.mouse_y));
}

fn rustymilk_sample_text(values: &[f64]) -> String {
    values
        .iter()
        .map(|value| format!("{:.6}", value.clamp(-1.0, 1.0)))
        .collect::<Vec<_>>()
        .join(",")
}

pub fn parse_rustymilk_sample_csv(source: &str) -> Vec<f64> {
    source
        .split(|ch: char| ch == ',' || ch == ';' || ch.is_whitespace())
        .filter_map(|item| {
            let trimmed = item.trim();
            if trimmed.is_empty() {
                None
            } else {
                trimmed.parse::<f64>().ok()
            }
        })
        .map(|value| value.clamp(-1.0, 1.0))
        .collect()
}

fn rustymilk_sample_bins(values: &[f64]) -> [f64; 64] {
    let mut bins = [0.0; 64];
    if values.is_empty() {
        return bins;
    }
    let bin_count = bins.len();
    for (index, bin) in bins.iter_mut().enumerate() {
        let sample_index = if bin_count <= 1 {
            0
        } else {
            index * values.len().saturating_sub(1) / bin_count.saturating_sub(1)
        };
        *bin = values
            .get(sample_index)
            .copied()
            .unwrap_or_default()
            .clamp(-1.0, 1.0);
    }
    bins
}

fn rustymilk_q_registers(scope: &BTreeMap<String, RustyMilkValue>) -> [f64; 64] {
    let mut registers = [0.0; 64];
    for index in 1..=64 {
        registers[index - 1] = rustymilk_scope_number(scope, &format!("q{index}"), 0.0);
    }
    registers
}

fn translated_rustymilk_shader_source(preset: &RustyMilkPresetDocument) -> String {
    if !preset.comp_shader.trim().is_empty() {
        let shader = create_translated_rustymilk_fragment_shader(&preset.comp_shader);
        if !shader.is_empty() {
            return shader;
        }
    }
    if !preset.warp_shader.trim().is_empty() {
        let shader = create_translated_rustymilk_fragment_shader(&preset.warp_shader);
        if !shader.is_empty() {
            return shader;
        }
    }
    String::new()
}

fn rustymilk_shader_texture_samplers(preset: &RustyMilkPresetDocument) -> Vec<String> {
    for shader in [&preset.comp_shader, &preset.warp_shader] {
        if shader.trim().is_empty() {
            continue;
        }
        if analyze_rustymilk_shader_support(shader).supported {
            return get_rustymilk_shader_texture_samplers(shader);
        }
    }
    Vec::new()
}

fn create_rustymilk_warp_mesh(
    preset: &RustyMilkPresetDocument,
    frame_scope: &BTreeMap<String, RustyMilkValue>,
) -> Option<RustyMilkWarpMesh> {
    let equations = preset.equations.per_pixel.trim();
    if equations.is_empty() {
        return None;
    }
    let columns = rustymilk_scope_number(frame_scope, "meshx", 8.0)
        .floor()
        .clamp(1.0, 128.0) as usize;
    let rows = rustymilk_scope_number(frame_scope, "meshy", 6.0)
        .floor()
        .clamp(1.0, 128.0) as usize;
    let mut positions = Vec::with_capacity(columns * rows * 12);
    let mut source_uvs = Vec::with_capacity(columns * rows * 12);
    let mut push_point = |x: f64, y: f64| {
        let centered_x = x - 0.5;
        let centered_y = y - 0.5;
        let mut point_scope = frame_scope.clone();
        point_scope.insert(
            "ang".to_string(),
            RustyMilkValue::Number(centered_y.atan2(centered_x)),
        );
        point_scope.insert(
            "rad".to_string(),
            RustyMilkValue::Number((centered_x * centered_x + centered_y * centered_y).sqrt()),
        );
        point_scope.insert("x".to_string(), RustyMilkValue::Number(x));
        point_scope.insert("y".to_string(), RustyMilkValue::Number(y));
        if let Ok(next_scope) = evaluate_rustymilk_equations(equations, &point_scope) {
            point_scope = next_scope;
        }
        let rotation = rustymilk_scope_number(&point_scope, "rot", 0.0);
        let zoom = rustymilk_scope_number(&point_scope, "zoom", 1.0)
            .abs()
            .max(0.001);
        let dx = rustymilk_scope_number(&point_scope, "dx", 0.0);
        let dy = rustymilk_scope_number(&point_scope, "dy", 0.0);
        let scaled_x = centered_x / zoom;
        let scaled_y = centered_y / zoom;
        let sine = rotation.sin();
        let cosine = rotation.cos();
        positions.push(x * 2.0 - 1.0);
        positions.push(y * 2.0 - 1.0);
        source_uvs.push(cosine * scaled_x - sine * scaled_y + 0.5 + dx);
        source_uvs.push(sine * scaled_x + cosine * scaled_y + 0.5 + dy);
    };
    for row in 0..rows {
        for column in 0..columns {
            let left = column as f64 / columns as f64;
            let right = (column + 1) as f64 / columns as f64;
            let top = row as f64 / rows as f64;
            let bottom = (row + 1) as f64 / rows as f64;
            push_point(left, top);
            push_point(left, bottom);
            push_point(right, top);
            push_point(right, top);
            push_point(left, bottom);
            push_point(right, bottom);
        }
    }
    Some(RustyMilkWarpMesh {
        positions,
        source_uvs,
    })
}

pub fn rustymilk_frame_from_source(
    source: &str,
    time_seconds: f64,
    bass: f64,
    mid: f64,
    treble: f64,
) -> RustyMilkFrame {
    rustymilk_frame_from_source_with_audio(source, time_seconds, bass, mid, treble, &[], &[])
}

fn build_rustymilk_frame_from_scope(
    source: &str,
    preset_document: &RustyMilkPresetDocument,
    scope: &BTreeMap<String, RustyMilkValue>,
    time_seconds: f64,
    bass: f64,
    mid: f64,
    treble: f64,
    waveform: &[f64],
    spectrum: &[f64],
) -> RustyMilkFrame {
    let fallback = parse_rustymilk_preset(source);
    let wave_r = clamp_unit(rustymilk_scope_number(scope, "wave_r", fallback.wave_r));
    let wave_g = clamp_unit(rustymilk_scope_number(scope, "wave_g", fallback.wave_g));
    let wave_b = clamp_unit(rustymilk_scope_number(scope, "wave_b", fallback.wave_b));
    let wave_scale = clamp_range(
        rustymilk_scope_number(scope, "wave_scale", fallback.wave_scale),
        0.2,
        3.0,
    );
    let pulse = (time_seconds * 1.7).sin() * 0.5 + 0.5;
    let wave_color = (
        (wave_r * 255.0).min(255.0) as u8,
        (wave_g * 255.0).min(255.0) as u8,
        (wave_b * 255.0).min(255.0) as u8,
    );
    let primitives = create_rustymilk_frame_primitives(
        preset_document,
        scope,
        time_seconds,
        bass,
        mid,
        treble,
        waveform,
        [wave_r, wave_g, wave_b],
    );
    let textured_primitives = create_rustymilk_frame_textured_primitives(
        preset_document,
        scope,
        [wave_r, wave_g, wave_b],
    );
    let q_registers = rustymilk_q_registers(scope);
    let fft_bins = rustymilk_sample_bins(spectrum);
    let waveform_bins = rustymilk_sample_bins(waveform);
    let warp_mesh = create_rustymilk_warp_mesh(preset_document, scope);
    RustyMilkFrame {
        background_alpha: clamp_range(
            1.0 - rustymilk_scope_number(scope, "decay", fallback.decay),
            0.01,
            0.5,
        ),
        bass,
        dx: clamp_range(rustymilk_scope_number(scope, "dx", 0.0), -0.5, 0.5),
        dy: clamp_range(rustymilk_scope_number(scope, "dy", 0.0), -0.5, 0.5),
        fft_bins,
        mid,
        primitives,
        q_registers,
        shape_count: preset_document
            .shapes
            .iter()
            .filter(|shape| rustymilk_base_number(&shape.base_values, "enabled", 0.0) > 0.0)
            .count(),
        rotation: clamp_range(
            rustymilk_scope_number(scope, "rot", fallback.rot),
            -0.5,
            0.5,
        ) + (treble - 0.5) * 0.02,
        shader_source: translated_rustymilk_shader_source(preset_document),
        shader_texture_samplers: rustymilk_shader_texture_samplers(preset_document),
        textured_primitives,
        treble,
        wave_color,
        waveform_bins,
        wave_radius: clamp_range(
            0.18 + wave_scale * 0.09 + bass * 0.12 + pulse * 0.04,
            0.12,
            0.68,
        ),
        waveform_count: preset_document
            .waves
            .iter()
            .filter(|wave| rustymilk_base_number(&wave.base_values, "enabled", 0.0) > 0.0)
            .count(),
        warp_mesh,
        zoom: clamp_range(
            rustymilk_scope_number(scope, "zoom", fallback.zoom),
            0.001,
            1.8,
        ),
    }
}

fn build_rustymilk_frame_from_runtime_scope(
    source: &str,
    preset_document: &mut RustyMilkPresetDocument,
    scope: &mut BTreeMap<String, RustyMilkValue>,
    time_seconds: f64,
    bass: f64,
    mid: f64,
    treble: f64,
    waveform: &[f64],
    spectrum: &[f64],
) -> RustyMilkFrame {
    let fallback = parse_rustymilk_preset(source);
    let wave_r = clamp_unit(rustymilk_scope_number(scope, "wave_r", fallback.wave_r));
    let wave_g = clamp_unit(rustymilk_scope_number(scope, "wave_g", fallback.wave_g));
    let wave_b = clamp_unit(rustymilk_scope_number(scope, "wave_b", fallback.wave_b));
    let wave_scale = clamp_range(
        rustymilk_scope_number(scope, "wave_scale", fallback.wave_scale),
        0.2,
        3.0,
    );
    let pulse = (time_seconds * 1.7).sin() * 0.5 + 0.5;
    let wave_color = (
        (wave_r * 255.0).min(255.0) as u8,
        (wave_g * 255.0).min(255.0) as u8,
        (wave_b * 255.0).min(255.0) as u8,
    );
    let (primitives, textured_primitives) = create_rustymilk_frame_primitives_and_textures_stateful(
        preset_document,
        scope,
        time_seconds,
        bass,
        mid,
        treble,
        waveform,
        spectrum,
        [wave_r, wave_g, wave_b],
    );
    let q_registers = rustymilk_q_registers(scope);
    let fft_bins = rustymilk_sample_bins(spectrum);
    let waveform_bins = rustymilk_sample_bins(waveform);
    let warp_mesh = create_rustymilk_warp_mesh(preset_document, scope);
    RustyMilkFrame {
        background_alpha: clamp_range(
            1.0 - rustymilk_scope_number(scope, "decay", fallback.decay),
            0.01,
            0.5,
        ),
        bass,
        dx: clamp_range(rustymilk_scope_number(scope, "dx", 0.0), -0.5, 0.5),
        dy: clamp_range(rustymilk_scope_number(scope, "dy", 0.0), -0.5, 0.5),
        fft_bins,
        mid,
        primitives,
        q_registers,
        shape_count: preset_document
            .shapes
            .iter()
            .filter(|shape| rustymilk_base_number(&shape.base_values, "enabled", 0.0) > 0.0)
            .count(),
        rotation: clamp_range(
            rustymilk_scope_number(scope, "rot", fallback.rot),
            -0.5,
            0.5,
        ) + (treble - 0.5) * 0.02,
        shader_source: translated_rustymilk_shader_source(preset_document),
        shader_texture_samplers: rustymilk_shader_texture_samplers(preset_document),
        textured_primitives,
        treble,
        wave_color,
        waveform_bins,
        wave_radius: clamp_range(
            0.18 + wave_scale * 0.09 + bass * 0.12 + pulse * 0.04,
            0.12,
            0.68,
        ),
        waveform_count: preset_document
            .waves
            .iter()
            .filter(|wave| rustymilk_base_number(&wave.base_values, "enabled", 0.0) > 0.0)
            .count(),
        warp_mesh,
        zoom: clamp_range(
            rustymilk_scope_number(scope, "zoom", fallback.zoom),
            0.001,
            1.8,
        ),
    }
}

pub fn rustymilk_frame_from_source_with_audio(
    source: &str,
    time_seconds: f64,
    bass: f64,
    mid: f64,
    treble: f64,
    waveform: &[f64],
    spectrum: &[f64],
) -> RustyMilkFrame {
    rustymilk_frame_from_source_with_audio_and_input(
        source,
        time_seconds,
        bass,
        mid,
        treble,
        waveform,
        spectrum,
        RustyMilkInputState::default(),
    )
}

pub fn rustymilk_frame_from_source_with_audio_and_input(
    source: &str,
    time_seconds: f64,
    bass: f64,
    mid: f64,
    treble: f64,
    waveform: &[f64],
    spectrum: &[f64],
    input: RustyMilkInputState,
) -> RustyMilkFrame {
    let parsed = parse_rustymilk_preset_set(source, false);
    let Some(preset_document) = parsed.presets.first() else {
        return rustymilk_frame(&RustyMilkPreset::default(), time_seconds, bass, mid, treble);
    };
    let mut scope = create_rustymilk_scope(preset_document, time_seconds, bass, mid, treble);
    update_rustymilk_scope_audio(
        &mut scope,
        time_seconds,
        (time_seconds * 60.0).floor(),
        bass,
        mid,
        treble,
        waveform,
        spectrum,
    );
    update_rustymilk_scope_input(&mut scope, input);
    if !preset_document.equations.init.trim().is_empty() {
        if let Ok(next_scope) =
            evaluate_rustymilk_equations(&preset_document.equations.init, &scope)
        {
            scope = next_scope;
        }
    }
    if !preset_document.equations.per_frame.trim().is_empty() {
        if let Ok(next_scope) =
            evaluate_rustymilk_equations(&preset_document.equations.per_frame, &scope)
        {
            scope = next_scope;
        }
    }
    build_rustymilk_frame_from_scope(
        source,
        preset_document,
        &scope,
        time_seconds,
        bass,
        mid,
        treble,
        waveform,
        spectrum,
    )
}

pub fn rustymilk_frame_set_from_source_with_audio(
    source: &str,
    time_seconds: f64,
    bass: f64,
    mid: f64,
    treble: f64,
    waveform: &[f64],
    spectrum: &[f64],
) -> RustyMilkFrameSet {
    rustymilk_frame_set_from_source_with_audio_and_input(
        source,
        time_seconds,
        bass,
        mid,
        treble,
        waveform,
        spectrum,
        RustyMilkInputState::default(),
    )
}

pub fn rustymilk_frame_set_from_source_with_audio_and_input(
    source: &str,
    time_seconds: f64,
    bass: f64,
    mid: f64,
    treble: f64,
    waveform: &[f64],
    spectrum: &[f64],
    input: RustyMilkInputState,
) -> RustyMilkFrameSet {
    let parsed =
        parse_rustymilk_preset_set(source, source.to_ascii_lowercase().contains("[preset01]"));
    let title = rustymilk_preset_set_title(&parsed);
    let transition_mode = rustymilk_transition_mode(&parsed);
    let transition_seconds = rustymilk_transition_seconds(&parsed);
    let entries = parsed
        .presets
        .iter()
        .enumerate()
        .map(|(index, preset_document)| {
            let mut scope =
                create_rustymilk_scope(preset_document, time_seconds, bass, mid, treble);
            update_rustymilk_scope_audio(
                &mut scope,
                time_seconds,
                (time_seconds * 60.0).floor(),
                bass,
                mid,
                treble,
                waveform,
                spectrum,
            );
            update_rustymilk_scope_input(&mut scope, input);
            if !preset_document.equations.init.trim().is_empty() {
                if let Ok(next_scope) =
                    evaluate_rustymilk_equations(&preset_document.equations.init, &scope)
                {
                    scope = next_scope;
                }
            }
            if !preset_document.equations.per_frame.trim().is_empty() {
                if let Ok(next_scope) =
                    evaluate_rustymilk_equations(&preset_document.equations.per_frame, &scope)
                {
                    scope = next_scope;
                }
            }
            RustyMilkCompositeFrame {
                blend_alpha: rustymilk_composite_alpha(preset_document, index),
                composite_mode: rustymilk_composite_mode(preset_document, index),
                frame: build_rustymilk_frame_from_scope(
                    source,
                    preset_document,
                    &scope,
                    time_seconds,
                    bass,
                    mid,
                    treble,
                    waveform,
                    spectrum,
                ),
                index,
                title: if preset_document.title.trim().is_empty() {
                    format!("Preset {}", index + 1)
                } else {
                    preset_document.title.clone()
                },
            }
        })
        .collect::<Vec<_>>();

    RustyMilkFrameSet {
        preset_count: entries.len(),
        entries,
        title,
        transition_mode,
        transition_seconds,
    }
}

pub fn rustymilk_frame_set_from_source(
    source: &str,
    time_seconds: f64,
    bass: f64,
    mid: f64,
    treble: f64,
) -> RustyMilkFrameSet {
    rustymilk_frame_set_from_source_with_audio(source, time_seconds, bass, mid, treble, &[], &[])
}

#[derive(Clone, Debug, Default)]
pub struct RustyMilkRuntime {
    initialized: bool,
    preset_document: Option<RustyMilkPresetDocument>,
    scope: BTreeMap<String, RustyMilkValue>,
    source: String,
}

impl RustyMilkRuntime {
    pub fn render_source(
        &mut self,
        source: &str,
        time_seconds: f64,
        bass: f64,
        mid: f64,
        treble: f64,
    ) -> RustyMilkFrame {
        self.render_source_with_audio(source, time_seconds, bass, mid, treble, &[], &[])
    }

    pub fn render_source_with_audio(
        &mut self,
        source: &str,
        time_seconds: f64,
        bass: f64,
        mid: f64,
        treble: f64,
        waveform: &[f64],
        spectrum: &[f64],
    ) -> RustyMilkFrame {
        self.render_source_with_audio_and_input(
            source,
            time_seconds,
            bass,
            mid,
            treble,
            waveform,
            spectrum,
            RustyMilkInputState::default(),
        )
    }

    pub fn render_source_with_audio_and_input(
        &mut self,
        source: &str,
        time_seconds: f64,
        bass: f64,
        mid: f64,
        treble: f64,
        waveform: &[f64],
        spectrum: &[f64],
        input: RustyMilkInputState,
    ) -> RustyMilkFrame {
        if self.source != source || self.preset_document.is_none() {
            let parsed = parse_rustymilk_preset_set(source, false);
            let Some(preset_document) = parsed.presets.first().cloned() else {
                *self = Self::default();
                return rustymilk_frame(
                    &RustyMilkPreset::default(),
                    time_seconds,
                    bass,
                    mid,
                    treble,
                );
            };
            self.scope = create_rustymilk_scope(&preset_document, time_seconds, bass, mid, treble);
            self.source = source.to_string();
            self.preset_document = Some(preset_document);
            self.initialized = false;
        }

        let Some(mut preset_document) = self.preset_document.take() else {
            return rustymilk_frame(&RustyMilkPreset::default(), time_seconds, bass, mid, treble);
        };
        let next_frame = rustymilk_scope_number(&self.scope, "frame", 0.0) + 1.0;
        update_rustymilk_scope_audio(
            &mut self.scope,
            time_seconds,
            next_frame,
            bass,
            mid,
            treble,
            waveform,
            spectrum,
        );
        update_rustymilk_scope_input(&mut self.scope, input);
        if !self.initialized {
            if !preset_document.equations.init.trim().is_empty() {
                if let Ok(next_scope) =
                    evaluate_rustymilk_equations(&preset_document.equations.init, &self.scope)
                {
                    self.scope = next_scope;
                }
            }
            self.initialized = true;
        }
        if !preset_document.equations.per_frame.trim().is_empty() {
            if let Ok(next_scope) =
                evaluate_rustymilk_equations(&preset_document.equations.per_frame, &self.scope)
            {
                self.scope = next_scope;
            }
        }

        let frame = build_rustymilk_frame_from_runtime_scope(
            &self.source,
            &mut preset_document,
            &mut self.scope,
            time_seconds,
            bass,
            mid,
            treble,
            waveform,
            spectrum,
        );
        self.preset_document = Some(preset_document);
        frame
    }
}

#[derive(Clone, Debug, Default)]
pub struct RustyMilkFrameSetRuntime {
    initialized: Vec<bool>,
    preset_documents: Vec<RustyMilkPresetDocument>,
    scopes: Vec<BTreeMap<String, RustyMilkValue>>,
    source: String,
}

impl RustyMilkFrameSetRuntime {
    pub fn render_source(
        &mut self,
        source: &str,
        time_seconds: f64,
        bass: f64,
        mid: f64,
        treble: f64,
    ) -> RustyMilkFrameSet {
        self.render_source_with_audio(source, time_seconds, bass, mid, treble, &[], &[])
    }

    pub fn render_source_with_audio(
        &mut self,
        source: &str,
        time_seconds: f64,
        bass: f64,
        mid: f64,
        treble: f64,
        waveform: &[f64],
        spectrum: &[f64],
    ) -> RustyMilkFrameSet {
        self.render_source_with_audio_and_input(
            source,
            time_seconds,
            bass,
            mid,
            treble,
            waveform,
            spectrum,
            RustyMilkInputState::default(),
        )
    }

    pub fn render_source_with_audio_and_input(
        &mut self,
        source: &str,
        time_seconds: f64,
        bass: f64,
        mid: f64,
        treble: f64,
        waveform: &[f64],
        spectrum: &[f64],
        input: RustyMilkInputState,
    ) -> RustyMilkFrameSet {
        if self.source != source || self.preset_documents.is_empty() {
            let parsed = parse_rustymilk_preset_set(
                source,
                source.to_ascii_lowercase().contains("[preset01]"),
            );
            self.initialized = vec![false; parsed.presets.len()];
            self.scopes = parsed
                .presets
                .iter()
                .map(|preset| create_rustymilk_scope(preset, time_seconds, bass, mid, treble))
                .collect();
            self.preset_documents = parsed.presets;
            self.source = source.to_string();
        }

        let parsed =
            parse_rustymilk_preset_set(source, source.to_ascii_lowercase().contains("[preset01]"));
        let title = rustymilk_preset_set_title(&parsed);
        let transition_mode = rustymilk_transition_mode(&parsed);
        let transition_seconds = rustymilk_transition_seconds(&parsed);
        let mut entries = Vec::with_capacity(self.preset_documents.len());

        for index in 0..self.preset_documents.len() {
            let preset_document = &mut self.preset_documents[index];
            let scope = &mut self.scopes[index];
            let next_frame = rustymilk_scope_number(scope, "frame", 0.0) + 1.0;
            update_rustymilk_scope_audio(
                scope,
                time_seconds,
                next_frame,
                bass,
                mid,
                treble,
                waveform,
                spectrum,
            );
            update_rustymilk_scope_input(scope, input);
            if !self.initialized.get(index).copied().unwrap_or_default() {
                if !preset_document.equations.init.trim().is_empty() {
                    if let Ok(next_scope) =
                        evaluate_rustymilk_equations(&preset_document.equations.init, scope)
                    {
                        *scope = next_scope;
                    }
                }
                if let Some(initialized) = self.initialized.get_mut(index) {
                    *initialized = true;
                }
            }
            if !preset_document.equations.per_frame.trim().is_empty() {
                if let Ok(next_scope) =
                    evaluate_rustymilk_equations(&preset_document.equations.per_frame, scope)
                {
                    *scope = next_scope;
                }
            }
            let blend_alpha = rustymilk_composite_alpha(preset_document, index);
            let composite_mode = rustymilk_composite_mode(preset_document, index);
            let title = if preset_document.title.trim().is_empty() {
                format!("Preset {}", index + 1)
            } else {
                preset_document.title.clone()
            };
            let frame = build_rustymilk_frame_from_runtime_scope(
                source,
                preset_document,
                scope,
                time_seconds,
                bass,
                mid,
                treble,
                waveform,
                spectrum,
            );
            entries.push(RustyMilkCompositeFrame {
                blend_alpha,
                composite_mode,
                frame,
                index,
                title,
            });
        }

        RustyMilkFrameSet {
            preset_count: entries.len(),
            entries,
            title,
            transition_mode,
            transition_seconds,
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub const RUSTYMILK_PRESETS: [&str; 3] = [
    "name=RustyMilk grid smoke\ndecay=0.91\nwave_r=0.12\nwave_g=0.64\nwave_b=0.88\nwave_a=0.86\nwave_scale=1.2\nzoom=1\nrot=0\nper_frame_1=wave_r=0.35+0.25*bass_att;\nper_frame_2=wave_g=0.45+0.2*mid_att;\nper_frame_3=wave_b=0.55+0.2*treb_att;\nper_frame_4=rot=0.01*sin(time*0.7);\nper_frame_5=zoom=1+0.03*sin(time*0.5);\nper_frame_6=dx=0.015*sin(time*0.6);\nper_frame_7=dy=0.015*cos(time*0.5);\nshape00_enabled=1\nshape00_sides=5\nshape00_rad=0.18\nwavecode_0_enabled=1\nwavecode_0_samples=96",
    "name=RustyMilk amber tunnel\ndecay=0.86\nwave_r=0.92\nwave_g=0.52\nwave_b=0.18\nwave_a=0.82\nwave_scale=1.55\nzoom=1.05\nrot=-0.018\nper_frame_1=wave_r=0.65+0.25*bass_att;\nper_frame_2=wave_g=0.32+0.2*mid_att;\nper_frame_3=rot=-0.025*sin(time*0.3);\nshape00_enabled=1\nshape00_sides=3\nshape01_enabled=1\nshape01_sides=6\nwavecode_0_enabled=1",
    "name=RustyMilk green pulse\ndecay=0.91\nwave_r=0.20\nwave_g=0.86\nwave_b=0.44\nwave_a=0.78\nwave_scale=1.1\nzoom=0.98\nrot=0.028\nper_frame_1=wave_g=0.55+0.35*mid_att;\nper_frame_2=wave_b=0.30+0.35*treb_att;\nper_frame_3=zoom=0.98+0.04*sin(time);\nwavecode_0_enabled=1\nwavecode_1_enabled=1",
];

pub fn rustymilk_preset_name(source: &str) -> String {
    source
        .lines()
        .find_map(|line| {
            let (key, value) = line.split_once('=')?;
            if key.trim().eq_ignore_ascii_case("name") {
                let value = value.trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
            None
        })
        .unwrap_or_else(|| "RustyMilk preset".to_string())
}

pub fn validate_rustymilk_import(source: &str) -> Result<String, String> {
    let parsed =
        parse_rustymilk_preset_set(source, source.to_ascii_lowercase().contains("[preset01]"));
    if parsed.presets.is_empty() {
        return Err("RustyMilk preset is empty".to_string());
    };
    let errors = parsed
        .presets
        .iter()
        .enumerate()
        .filter_map(|(index, preset)| {
            let report = analyze_rustymilk_preset_compatibility(preset);
            let error = rustymilk_compatibility_error(&report);
            if error.is_empty() {
                None
            } else if parsed.presets.len() == 1 {
                Some(error)
            } else {
                Some(format!("preset {}: {error}", index + 1))
            }
        })
        .collect::<Vec<_>>();
    if !errors.is_empty() {
        return Err(errors.join("; "));
    }
    let title = rustymilk_preset_set_title(&parsed);
    if title.trim().is_empty() {
        return Ok("Imported RustyMilk preset".to_string());
    }
    Ok(title)
}

fn rustymilk_preset_set_title(parsed: &RustyMilkPresetSet) -> String {
    let titles = parsed
        .presets
        .iter()
        .map(|preset| preset.title.trim())
        .filter(|title| !title.is_empty())
        .collect::<Vec<_>>();
    if titles.is_empty() {
        "Imported RustyMilk preset".to_string()
    } else if titles.len() == 1 {
        titles[0].to_string()
    } else {
        titles.join(" + ")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RustyMilkValue {
    Number(f64),
    Text(String),
}

impl RustyMilkValue {
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            Self::Text(_) => None,
        }
    }

    pub fn as_text(&self) -> String {
        match self {
            Self::Number(value) => {
                if value.fract().abs() < f64::EPSILON {
                    format!("{}", *value as i64)
                } else {
                    format!("{value}")
                }
            }
            Self::Text(value) => value.clone(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RustyMilkEquations {
    pub frame: String,
    pub init: String,
    pub per_frame: String,
    pub per_pixel: String,
    pub point: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RustyMilkIndexedEntry {
    pub base_values: BTreeMap<String, RustyMilkValue>,
    pub equations: RustyMilkEquations,
    pub initialized: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RustyMilkPresetDocument {
    pub base_values: BTreeMap<String, RustyMilkValue>,
    pub comp_shader: String,
    pub format: String,
    pub index: usize,
    pub raw_sections: BTreeMap<String, BTreeMap<String, RustyMilkValue>>,
    pub shapes: Vec<RustyMilkIndexedEntry>,
    pub source: String,
    pub sprites: Vec<RustyMilkIndexedEntry>,
    pub title: String,
    pub warp_shader: String,
    pub waves: Vec<RustyMilkIndexedEntry>,
    pub equations: RustyMilkEquations,
}

impl RustyMilkPresetDocument {
    fn new(source: &str, index: usize) -> Self {
        Self {
            base_values: BTreeMap::new(),
            comp_shader: String::new(),
            equations: RustyMilkEquations::default(),
            format: "milk".to_string(),
            index,
            raw_sections: BTreeMap::new(),
            shapes: Vec::new(),
            source: source.to_string(),
            sprites: Vec::new(),
            title: String::new(),
            warp_shader: String::new(),
            waves: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RustyMilkPresetSet {
    pub format: String,
    pub presets: Vec<RustyMilkPresetDocument>,
}

fn is_numeric_rustymilk_value(value: &str) -> bool {
    value.trim().parse::<f64>().is_ok()
}

fn normalize_rustymilk_value(value: &str) -> RustyMilkValue {
    let trimmed = value.trim();
    if is_numeric_rustymilk_value(trimmed) {
        RustyMilkValue::Number(trimmed.parse::<f64>().unwrap_or(0.0))
    } else {
        RustyMilkValue::Text(trimmed.to_string())
    }
}

fn append_rustymilk_statement(target: &mut String, value: &str) {
    if value.trim().is_empty() {
        return;
    }
    if !target.is_empty() {
        target.push('\n');
    }
    target.push_str(value.trim());
}

fn split_rustymilk_preset_pair(source: &str) -> Vec<String> {
    let normalized = source.replace("\r\n", "\n").replace('\r', "\n");
    let mut offset = 0usize;
    for line in normalized.split_inclusive('\n') {
        if line.trim().eq_ignore_ascii_case("[preset01]") {
            return vec![
                normalized[..offset].to_string(),
                normalized[offset..].to_string(),
            ];
        }
        offset += line.len();
    }
    vec![normalized]
}

fn parse_indexed_key<'a>(key: &'a str, prefix: &str) -> Option<(usize, &'a str)> {
    let rest = key.strip_prefix(prefix)?;
    let digit_count = rest.chars().take_while(|ch| ch.is_ascii_digit()).count();
    if digit_count == 0 || !rest[digit_count..].starts_with('_') {
        return None;
    }
    let index = rest[..digit_count].parse::<usize>().ok()?;
    Some((index, &rest[digit_count + 1..]))
}

fn ensure_rustymilk_entry(
    entries: &mut Vec<RustyMilkIndexedEntry>,
    index: usize,
) -> &mut RustyMilkIndexedEntry {
    while entries.len() <= index {
        entries.push(RustyMilkIndexedEntry::default());
    }
    &mut entries[index]
}

fn assign_rustymilk_equation(equations: &mut RustyMilkEquations, key: &str, value: &str) -> bool {
    if key.starts_with("per_frame") || key.starts_with("frame") {
        append_rustymilk_statement(&mut equations.per_frame, value);
        return true;
    }
    if key.starts_with("per_pixel") || key.starts_with("per_vertex") {
        append_rustymilk_statement(&mut equations.per_pixel, value);
        return true;
    }
    if key.starts_with("init") {
        append_rustymilk_statement(&mut equations.init, value);
        return true;
    }
    matches!(
        key.split('_').next().unwrap_or_default(),
        "per_frame" | "per_pixel" | "per_vertex" | "init"
    )
}

fn assign_rustymilk_indexed_equation(
    equations: &mut RustyMilkEquations,
    key: &str,
    value: &str,
) -> bool {
    if key.starts_with("init") {
        append_rustymilk_statement(&mut equations.init, value);
        return true;
    }
    if key.starts_with("frame") || key.starts_with("per_frame") {
        append_rustymilk_statement(&mut equations.frame, value);
        return true;
    }
    if key.starts_with("point") || key.starts_with("per_point") {
        append_rustymilk_statement(&mut equations.point, value);
        return true;
    }
    false
}

fn parse_rustymilk_preset_text(text: &str, index: usize) -> RustyMilkPresetDocument {
    let mut preset = RustyMilkPresetDocument::new(text, index);
    let mut section = "preset".to_string();

    for raw_line in text.replace("\r\n", "\n").replace('\r', "\n").lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with("//") {
            continue;
        }
        if trimmed.starts_with('[') && trimmed.ends_with(']') && trimmed.len() > 2 {
            section = trimmed[1..trimmed.len() - 1].trim().to_ascii_lowercase();
            preset.raw_sections.entry(section.clone()).or_default();
            continue;
        }
        let Some((raw_key, raw_value)) = raw_line.split_once('=') else {
            continue;
        };
        let key = raw_key.trim().to_ascii_lowercase();
        let raw_value = raw_value.trim();
        let value = normalize_rustymilk_value(raw_value);
        preset
            .raw_sections
            .entry(section.clone())
            .or_default()
            .insert(key.clone(), value.clone());

        if key == "name" || key == "preset_name" {
            preset.title = raw_value.to_string();
            continue;
        }
        if let Some((shape_index, shape_key)) = parse_indexed_key(&key, "shape") {
            let entry = ensure_rustymilk_entry(&mut preset.shapes, shape_index);
            if !assign_rustymilk_indexed_equation(&mut entry.equations, shape_key, raw_value) {
                entry.base_values.insert(shape_key.to_string(), value);
            }
            continue;
        }
        if let Some((sprite_index, sprite_key)) = parse_indexed_key(&key, "sprite") {
            let entry = ensure_rustymilk_entry(&mut preset.sprites, sprite_index);
            if !assign_rustymilk_indexed_equation(&mut entry.equations, sprite_key, raw_value) {
                entry.base_values.insert(sprite_key.to_string(), value);
            }
            continue;
        }
        if let Some((wave_index, wave_key)) = parse_indexed_key(&key, "wavecode_") {
            let entry = ensure_rustymilk_entry(&mut preset.waves, wave_index);
            if !assign_rustymilk_indexed_equation(&mut entry.equations, wave_key, raw_value) {
                entry.base_values.insert(wave_key.to_string(), value);
            }
            continue;
        }
        if key.starts_with("warp_shader") {
            append_rustymilk_statement(&mut preset.warp_shader, raw_value);
            continue;
        }
        if key.starts_with("comp_shader") {
            append_rustymilk_statement(&mut preset.comp_shader, raw_value);
            continue;
        }
        if assign_rustymilk_equation(&mut preset.equations, &key, raw_value) {
            continue;
        }
        preset.base_values.insert(key, value);
    }

    preset
}

pub fn parse_rustymilk_preset_set(source: &str, force_milk2: bool) -> RustyMilkPresetSet {
    let chunks = split_rustymilk_preset_pair(source);
    let format = if force_milk2 || chunks.len() > 1 {
        "milk2"
    } else {
        "milk"
    }
    .to_string();
    let presets = chunks
        .iter()
        .enumerate()
        .map(|(index, chunk)| {
            let mut preset = parse_rustymilk_preset_text(chunk, index);
            preset.format = format.clone();
            preset
        })
        .collect::<Vec<_>>();
    RustyMilkPresetSet { format, presets }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RustyMilkFragment {
    pub entries: Vec<RustyMilkIndexedEntry>,
    pub fragment_type: String,
    pub source: String,
}

fn rustymilk_fragment_type(file_name: &str, requested_type: &str) -> String {
    if requested_type == "shape" || requested_type == "wave" {
        return requested_type.to_string();
    }
    if file_name.to_ascii_lowercase().ends_with(".wave") {
        "wave".to_string()
    } else {
        "shape".to_string()
    }
}

fn parse_standalone_rustymilk_fragment_entry(source: &str) -> RustyMilkIndexedEntry {
    let mut entry = RustyMilkIndexedEntry::default();
    entry
        .base_values
        .insert("enabled".to_string(), RustyMilkValue::Number(1.0));
    for raw_line in source.replace("\r\n", "\n").replace('\r', "\n").lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with(';')
            || trimmed.starts_with("//")
            || (trimmed.starts_with('[') && trimmed.ends_with(']'))
        {
            continue;
        }
        let Some((raw_key, raw_value)) = raw_line.split_once('=') else {
            continue;
        };
        let key = raw_key.trim().to_ascii_lowercase();
        let raw_value = raw_value.trim();
        if !assign_rustymilk_indexed_equation(&mut entry.equations, &key, raw_value) {
            entry
                .base_values
                .insert(key, normalize_rustymilk_value(raw_value));
        }
    }
    entry
}

pub fn parse_rustymilk_fragment(
    source: &str,
    file_name: &str,
    requested_type: &str,
) -> RustyMilkFragment {
    let fragment_type = rustymilk_fragment_type(file_name, requested_type);
    let parsed = parse_rustymilk_preset_set(source, false);
    let parsed_entries = if fragment_type == "wave" {
        parsed
            .presets
            .first()
            .map(|preset| preset.waves.clone())
            .unwrap_or_default()
    } else {
        parsed
            .presets
            .first()
            .map(|preset| preset.shapes.clone())
            .unwrap_or_default()
    };
    let has_prefixed_entries = parsed_entries.iter().any(|entry| {
        !entry.base_values.is_empty() || entry.equations != RustyMilkEquations::default()
    });
    let entries = if has_prefixed_entries {
        parsed_entries
            .into_iter()
            .filter(|entry| {
                !entry.base_values.is_empty() || entry.equations != RustyMilkEquations::default()
            })
            .collect()
    } else {
        vec![parse_standalone_rustymilk_fragment_entry(source)]
    };
    RustyMilkFragment {
        entries,
        fragment_type,
        source: source.to_string(),
    }
}

fn append_rustymilk_equation_lines(lines: &mut Vec<String>, key: &str, equation_text: &str) {
    for (index, line) in equation_text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .enumerate()
    {
        lines.push(format!("{key}_{}={line}", index + 1));
    }
}

fn append_rustymilk_base_value_lines(
    lines: &mut Vec<String>,
    values: &BTreeMap<String, RustyMilkValue>,
    prefix: &str,
) {
    for (key, value) in values {
        lines.push(format!("{prefix}{key}={}", value.as_text()));
    }
}

fn append_rustymilk_indexed_entry_lines(
    lines: &mut Vec<String>,
    prefix: &str,
    entry: &RustyMilkIndexedEntry,
) {
    append_rustymilk_base_value_lines(lines, &entry.base_values, prefix);
    append_rustymilk_equation_lines(lines, &format!("{prefix}init"), &entry.equations.init);
    append_rustymilk_equation_lines(lines, &format!("{prefix}per_frame"), &entry.equations.frame);
    append_rustymilk_equation_lines(lines, &format!("{prefix}per_point"), &entry.equations.point);
}

pub fn serialize_rustymilk_fragment(entry: &RustyMilkIndexedEntry, requested_type: &str) -> String {
    let fragment_type = rustymilk_fragment_type("", requested_type);
    let mut lines = vec![format!("[{fragment_type}]")];
    append_rustymilk_indexed_entry_lines(&mut lines, "", entry);
    format!("{}\n", lines.join("\n"))
}

pub fn serialize_rustymilk_preset_set(parsed: &RustyMilkPresetSet) -> String {
    let include_sections = parsed.format == "milk2" || parsed.presets.len() > 1;
    let mut rendered_presets = Vec::new();
    for (index, preset) in parsed.presets.iter().enumerate() {
        let mut lines = Vec::new();
        if include_sections {
            lines.push(format!("[preset{index:02}]"));
        }
        if !preset.title.is_empty() {
            lines.push(format!("name={}", preset.title));
        }
        append_rustymilk_base_value_lines(&mut lines, &preset.base_values, "");
        append_rustymilk_equation_lines(&mut lines, "init", &preset.equations.init);
        append_rustymilk_equation_lines(&mut lines, "per_frame", &preset.equations.per_frame);
        append_rustymilk_equation_lines(&mut lines, "per_pixel", &preset.equations.per_pixel);
        append_rustymilk_equation_lines(&mut lines, "warp_shader", &preset.warp_shader);
        append_rustymilk_equation_lines(&mut lines, "comp_shader", &preset.comp_shader);
        for (shape_index, shape) in preset.shapes.iter().enumerate() {
            append_rustymilk_indexed_entry_lines(
                &mut lines,
                &format!("shape{shape_index:02}_"),
                shape,
            );
        }
        for (sprite_index, sprite) in preset.sprites.iter().enumerate() {
            append_rustymilk_indexed_entry_lines(
                &mut lines,
                &format!("sprite{sprite_index:02}_"),
                sprite,
            );
        }
        for (wave_index, wave) in preset.waves.iter().enumerate() {
            append_rustymilk_indexed_entry_lines(
                &mut lines,
                &format!("wavecode_{wave_index}_"),
                wave,
            );
        }
        rendered_presets.push(lines.join("\n"));
    }
    format!("{}\n", rendered_presets.join("\n"))
}

fn is_rustymilk_q_variable(key: &str) -> bool {
    key.strip_prefix('q')
        .and_then(|rest| rest.parse::<usize>().ok())
        .is_some_and(|index| (1..=64).contains(&index))
}

fn is_rustymilk_buffer_variable(key: &str) -> bool {
    key.strip_prefix("megabuf_")
        .or_else(|| key.strip_prefix("gmegabuf_"))
        .and_then(|rest| rest.parse::<usize>().ok())
        .is_some()
}

fn persist_rustymilk_scoped_values(
    base_values: &BTreeMap<String, RustyMilkValue>,
    scope: &BTreeMap<String, RustyMilkValue>,
    allowed_keys: &[&str],
) -> BTreeMap<String, RustyMilkValue> {
    let mut next = base_values.clone();
    for (key, value) in scope {
        if allowed_keys.contains(&key.as_str())
            || is_rustymilk_q_variable(key)
            || is_rustymilk_buffer_variable(key)
        {
            next.insert(key.clone(), value.clone());
        }
    }
    next
}

const RUSTYMILK_SHAPE_VALUE_KEYS: &[&str] = &[
    "a",
    "a2",
    "additive",
    "ang",
    "b",
    "b2",
    "badditive",
    "benabled",
    "border_a",
    "border_b",
    "border_g",
    "border_r",
    "enabled",
    "g",
    "g2",
    "numsides",
    "r",
    "r2",
    "rad",
    "radius",
    "sides",
    "tex",
    "tex_ang",
    "texang",
    "tex_name",
    "texname",
    "tex_zoom",
    "texzoom",
    "texture",
    "textured",
    "thickoutline",
    "x",
    "y",
];

const RUSTYMILK_SPRITE_VALUE_KEYS: &[&str] = &[
    "a",
    "additive",
    "ang",
    "b",
    "badditive",
    "benabled",
    "enabled",
    "file",
    "filename",
    "g",
    "h",
    "height",
    "image",
    "img",
    "r",
    "tex",
    "tex_name",
    "texname",
    "texture",
    "w",
    "width",
    "x",
    "y",
];

const RUSTYMILK_WAVE_VALUE_KEYS: &[&str] = &[
    "a",
    "additive",
    "b",
    "badditive",
    "bdrawthick",
    "benabled",
    "bspectrum",
    "bthick",
    "busedots",
    "dots",
    "enabled",
    "g",
    "nsamples",
    "r",
    "samples",
    "scaling",
    "spectrum",
    "thick",
];

fn evaluate_rustymilk_entry_state(
    entry: &RustyMilkIndexedEntry,
    frame_scope: &BTreeMap<String, RustyMilkValue>,
    allowed_keys: &[&str],
) -> RustyMilkIndexedEntry {
    let mut scope = frame_scope.clone();
    scope.extend(entry.base_values.clone());
    if !entry.equations.init.trim().is_empty() {
        if let Ok(next_scope) = evaluate_rustymilk_equations(&entry.equations.init, &scope) {
            scope = next_scope;
        }
    }
    if !entry.equations.frame.trim().is_empty() {
        if let Ok(next_scope) = evaluate_rustymilk_equations(&entry.equations.frame, &scope) {
            scope = next_scope;
        }
    }
    RustyMilkIndexedEntry {
        base_values: persist_rustymilk_scoped_values(&entry.base_values, &scope, allowed_keys),
        equations: entry.equations.clone(),
        initialized: entry.initialized,
    }
}

fn merge_rustymilk_q_registers(
    scope: &mut BTreeMap<String, RustyMilkValue>,
    values: &BTreeMap<String, RustyMilkValue>,
) {
    for index in 1..=64 {
        let key = format!("q{index}");
        if let Some(value) = values.get(&key) {
            scope.insert(key, value.clone());
        }
    }
}

fn evaluate_rustymilk_entry_stateful(
    entry: &mut RustyMilkIndexedEntry,
    frame_scope: &BTreeMap<String, RustyMilkValue>,
    allowed_keys: &[&str],
) -> RustyMilkIndexedEntry {
    let mut scope = frame_scope.clone();
    scope.extend(entry.base_values.clone());
    if !entry.initialized && !entry.equations.init.trim().is_empty() {
        if let Ok(next_scope) = evaluate_rustymilk_equations(&entry.equations.init, &scope) {
            scope = next_scope;
        }
        entry.initialized = true;
    }
    if !entry.equations.frame.trim().is_empty() {
        if let Ok(next_scope) = evaluate_rustymilk_equations(&entry.equations.frame, &scope) {
            scope = next_scope;
        }
    }
    entry.base_values = persist_rustymilk_scoped_values(&entry.base_values, &scope, allowed_keys);
    RustyMilkIndexedEntry {
        base_values: entry.base_values.clone(),
        equations: entry.equations.clone(),
        initialized: entry.initialized,
    }
}

pub fn evaluate_rustymilk_shape_state(
    shape: &RustyMilkIndexedEntry,
    frame_scope: &BTreeMap<String, RustyMilkValue>,
) -> RustyMilkIndexedEntry {
    evaluate_rustymilk_entry_state(shape, frame_scope, RUSTYMILK_SHAPE_VALUE_KEYS)
}

pub fn evaluate_rustymilk_sprite_state(
    sprite: &RustyMilkIndexedEntry,
    frame_scope: &BTreeMap<String, RustyMilkValue>,
) -> RustyMilkIndexedEntry {
    evaluate_rustymilk_entry_state(sprite, frame_scope, RUSTYMILK_SPRITE_VALUE_KEYS)
}

pub fn evaluate_rustymilk_wave_state(
    wave: &RustyMilkIndexedEntry,
    frame_scope: &BTreeMap<String, RustyMilkValue>,
) -> RustyMilkIndexedEntry {
    evaluate_rustymilk_entry_state(wave, frame_scope, RUSTYMILK_WAVE_VALUE_KEYS)
}

fn rustymilk_entry_number(entry: &RustyMilkIndexedEntry, keys: &[&str], fallback: f64) -> f64 {
    keys.iter()
        .find_map(|key| {
            entry
                .base_values
                .get(*key)
                .and_then(RustyMilkValue::as_number)
        })
        .filter(|value| value.is_finite())
        .unwrap_or(fallback)
}

fn rustymilk_entry_flag(entry: &RustyMilkIndexedEntry, keys: &[&str]) -> bool {
    keys.iter()
        .any(|key| rustymilk_entry_number(entry, &[*key], 0.0).abs() > 0.00001)
}

pub fn create_rustymilk_custom_wave_vertices(
    wave: &RustyMilkIndexedEntry,
    samples: &[f64],
    frame_scope: &BTreeMap<String, RustyMilkValue>,
) -> Vec<f64> {
    let sample_count = rustymilk_entry_number(wave, &["samples", "nsamples"], samples.len() as f64)
        .floor()
        .max(1.0) as usize;
    let mut vertices = Vec::with_capacity(sample_count * 2);
    for index in 0..sample_count {
        let sample_index = if sample_count <= 1 {
            0
        } else {
            (index * samples.len().saturating_sub(1)) / sample_count.saturating_sub(1)
        };
        let mut sample = samples.get(sample_index).copied().unwrap_or_default();
        if sample > 1.0 {
            sample /= 255.0;
        }
        let i = if sample_count <= 1 {
            0.0
        } else {
            index as f64 / (sample_count - 1) as f64
        };
        let mut point_scope = frame_scope.clone();
        point_scope.extend(wave.base_values.clone());
        point_scope.insert("i".to_string(), RustyMilkValue::Number(i));
        point_scope.insert("sample".to_string(), RustyMilkValue::Number(sample));
        if !wave.equations.point.trim().is_empty() {
            if let Ok(next_scope) =
                evaluate_rustymilk_equations(&wave.equations.point, &point_scope)
            {
                point_scope = next_scope;
            }
        }
        let x = rustymilk_scope_number(&point_scope, "x", i) * 2.0 - 1.0;
        let y = rustymilk_scope_number(&point_scope, "y", sample) * 2.0 - 1.0;
        vertices.push(x);
        vertices.push(y);
    }
    vertices
}

pub fn create_rustymilk_shape_vertices(shape: &RustyMilkIndexedEntry) -> Vec<f64> {
    if !rustymilk_entry_flag(shape, &["enabled", "benabled"]) {
        return Vec::new();
    }
    let sides = rustymilk_entry_number(shape, &["sides", "numsides"], 4.0)
        .floor()
        .clamp(3.0, 500.0) as usize;
    let radius = rustymilk_entry_number(shape, &["rad", "radius"], 0.1).max(0.0);
    let center_x = rustymilk_entry_number(shape, &["x"], 0.5) * 2.0 - 1.0;
    let center_y = rustymilk_entry_number(shape, &["y"], 0.5) * 2.0 - 1.0;
    let angle = rustymilk_entry_number(shape, &["ang"], 0.0);
    let mut vertices = Vec::with_capacity((sides + 1) * 2);
    for index in 0..=sides {
        let theta = angle + (index as f64 / sides as f64) * std::f64::consts::TAU;
        vertices.push(center_x + theta.cos() * radius);
        vertices.push(center_y + theta.sin() * radius);
    }
    vertices
}

pub fn create_rustymilk_shape_fill_vertices(shape: &RustyMilkIndexedEntry) -> Vec<f64> {
    let outline = create_rustymilk_shape_vertices(shape);
    if outline.is_empty() {
        return outline;
    }
    let mut vertices = Vec::with_capacity(outline.len() + 2);
    vertices.push(rustymilk_entry_number(shape, &["x"], 0.5) * 2.0 - 1.0);
    vertices.push(rustymilk_entry_number(shape, &["y"], 0.5) * 2.0 - 1.0);
    vertices.extend(outline);
    vertices
}

fn rustymilk_entry_text(entry: &RustyMilkIndexedEntry, keys: &[&str]) -> String {
    keys.iter()
        .find_map(|key| entry.base_values.get(*key).map(RustyMilkValue::as_text))
        .unwrap_or_default()
}

fn is_rustymilk_shape_textured(shape: &RustyMilkIndexedEntry) -> bool {
    rustymilk_entry_flag(shape, &["textured", "btextured"])
        || !rustymilk_entry_text(shape, &["texture", "tex_name", "texname", "tex"]).is_empty()
}

fn rustymilk_texture_name(entry: &RustyMilkIndexedEntry) -> String {
    rustymilk_entry_text(
        entry,
        &[
            "texture", "tex", "tex_name", "texname", "image", "img", "file", "filename",
        ],
    )
}

pub fn get_rustymilk_texture_name_aliases(value: &str) -> Vec<String> {
    let normalized = value
        .trim()
        .trim_matches(|ch| ch == '\'' || ch == '"')
        .replace('\\', "/")
        .to_ascii_lowercase();
    let basename = normalized
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .to_string();
    let stem = basename
        .rsplit_once('.')
        .map(|(stem, _)| stem.to_string())
        .unwrap_or_else(|| basename.clone());
    let mut aliases = Vec::new();
    for alias in [normalized, basename, stem] {
        if !alias.is_empty() && !aliases.contains(&alias) {
            aliases.push(alias);
        }
    }
    aliases
}

pub fn create_rustymilk_shape_texture_uvs(shape: &RustyMilkIndexedEntry) -> Vec<f64> {
    let vertex_count = create_rustymilk_shape_fill_vertices(shape).len() / 2;
    if vertex_count == 0 {
        return Vec::new();
    }
    let zoom = rustymilk_entry_number(shape, &["tex_zoom", "texzoom"], 1.0)
        .abs()
        .max(0.001);
    let angle = rustymilk_entry_number(shape, &["tex_ang", "texang"], 0.0);
    let sine = angle.sin();
    let cosine = angle.cos();
    let mut uvs = Vec::with_capacity(vertex_count * 2);
    uvs.push(0.5);
    uvs.push(0.5);
    for index in 1..vertex_count {
        let progress = (index - 1) as f64 / (vertex_count.saturating_sub(2).max(1)) as f64;
        let theta = progress * std::f64::consts::TAU;
        let radius = 0.5 / zoom;
        let x = theta.cos() * radius;
        let y = theta.sin() * radius;
        uvs.push(0.5 + cosine * x - sine * y);
        uvs.push(0.5 + sine * x + cosine * y);
    }
    uvs
}

pub fn create_rustymilk_sprite_vertices(sprite: &RustyMilkIndexedEntry) -> Vec<f64> {
    if !rustymilk_entry_flag(sprite, &["enabled", "benabled"]) {
        return Vec::new();
    }
    let width = rustymilk_entry_number(sprite, &["w", "width"], 0.25)
        .abs()
        .max(0.001);
    let height = rustymilk_entry_number(sprite, &["h", "height"], width)
        .abs()
        .max(0.001);
    let center_x = rustymilk_entry_number(sprite, &["x"], 0.5) * 2.0 - 1.0;
    let center_y = rustymilk_entry_number(sprite, &["y"], 0.5) * 2.0 - 1.0;
    let angle = rustymilk_entry_number(sprite, &["ang"], 0.0);
    let sine = angle.sin();
    let cosine = angle.cos();
    let corners = [
        (-width, -height),
        (width, -height),
        (width, height),
        (-width, height),
        (-width, -height),
    ];
    let mut vertices = Vec::with_capacity(10);
    for (x, y) in corners {
        vertices.push(center_x + cosine * x - sine * y);
        vertices.push(center_y + sine * x + cosine * y);
    }
    vertices
}

pub fn create_rustymilk_sprite_texture_uvs(sprite: &RustyMilkIndexedEntry) -> Vec<f64> {
    if !rustymilk_entry_flag(sprite, &["enabled", "benabled"]) {
        return Vec::new();
    }
    vec![0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]
}

fn rustymilk_shape_fill_color(shape: &RustyMilkIndexedEntry, fallback_color: [f64; 3]) -> [f64; 4] {
    [
        clamp_unit(rustymilk_entry_number(shape, &["r"], fallback_color[0])),
        clamp_unit(rustymilk_entry_number(shape, &["g"], fallback_color[1])),
        clamp_unit(rustymilk_entry_number(shape, &["b"], fallback_color[2])),
        clamp_unit(rustymilk_entry_number(shape, &["a"], 0.6)),
    ]
}

fn rustymilk_shape_fill_edge_color(
    shape: &RustyMilkIndexedEntry,
    fallback_color: [f64; 3],
) -> [f64; 4] {
    [
        clamp_unit(rustymilk_entry_number(
            shape,
            &["r2", "r"],
            fallback_color[0],
        )),
        clamp_unit(rustymilk_entry_number(
            shape,
            &["g2", "g"],
            fallback_color[1],
        )),
        clamp_unit(rustymilk_entry_number(
            shape,
            &["b2", "b"],
            fallback_color[2],
        )),
        clamp_unit(rustymilk_entry_number(shape, &["a2", "a"], 0.6)),
    ]
}

fn rustymilk_shape_border_color(
    shape: &RustyMilkIndexedEntry,
    fallback_color: [f64; 3],
) -> [f64; 4] {
    [
        clamp_unit(rustymilk_entry_number(
            shape,
            &["border_r", "r"],
            fallback_color[0],
        )),
        clamp_unit(rustymilk_entry_number(
            shape,
            &["border_g", "g"],
            fallback_color[1],
        )),
        clamp_unit(rustymilk_entry_number(
            shape,
            &["border_b", "b"],
            fallback_color[2],
        )),
        clamp_unit(rustymilk_entry_number(shape, &["border_a"], 0.85)),
    ]
}

fn rustymilk_sprite_fill_color(
    sprite: &RustyMilkIndexedEntry,
    fallback_color: [f64; 3],
) -> [f64; 4] {
    [
        clamp_unit(rustymilk_entry_number(sprite, &["r"], fallback_color[0])),
        clamp_unit(rustymilk_entry_number(sprite, &["g"], fallback_color[1])),
        clamp_unit(rustymilk_entry_number(sprite, &["b"], fallback_color[2])),
        clamp_unit(rustymilk_entry_number(sprite, &["a"], 1.0)),
    ]
}

fn create_rustymilk_shape_fill_colors(
    shape: &RustyMilkIndexedEntry,
    fallback_color: [f64; 3],
) -> Vec<f64> {
    let vertex_count = create_rustymilk_shape_fill_vertices(shape).len() / 2;
    if vertex_count == 0 {
        return Vec::new();
    }
    let mut colors = Vec::with_capacity(vertex_count * 4);
    colors.extend_from_slice(&rustymilk_shape_fill_color(shape, fallback_color));
    let edge_color = rustymilk_shape_fill_edge_color(shape, fallback_color);
    for _ in 1..vertex_count {
        colors.extend_from_slice(&edge_color);
    }
    colors
}

fn append_rustymilk_webgpu_colored_vertex(
    output: &mut Vec<f64>,
    vertices: &[f64],
    vertex_index: usize,
    color: [f64; 4],
) {
    output.extend_from_slice(&[
        vertices[vertex_index * 2],
        vertices[vertex_index * 2 + 1],
        color[0],
        color[1],
        color[2],
        color[3],
    ]);
}

pub fn create_rustymilk_webgpu_triangle_list_vertices(
    triangle_vertices: &[f64],
    color: [f64; 4],
) -> Vec<f64> {
    let vertex_count = triangle_vertices.len() / 2;
    if vertex_count < 3 {
        return Vec::new();
    }
    let mut vertices = Vec::with_capacity(vertex_count * 6);
    for index in 0..vertex_count {
        append_rustymilk_webgpu_colored_vertex(&mut vertices, triangle_vertices, index, color);
    }
    vertices
}

pub fn create_rustymilk_webgpu_triangle_fan_vertices(
    fan_vertices: &[f64],
    fan_colors: &[f64],
    fallback_color: [f64; 4],
) -> Vec<f64> {
    let vertex_count = fan_vertices.len() / 2;
    if vertex_count < 3 {
        return Vec::new();
    }
    let mut vertices = Vec::with_capacity((vertex_count - 2) * 18);
    let append_vertex = |output: &mut Vec<f64>, vertex_index: usize| {
        let color_offset = vertex_index * 4;
        let color = [
            *fan_colors.get(color_offset).unwrap_or(&fallback_color[0]),
            *fan_colors
                .get(color_offset + 1)
                .unwrap_or(&fallback_color[1]),
            *fan_colors
                .get(color_offset + 2)
                .unwrap_or(&fallback_color[2]),
            *fan_colors
                .get(color_offset + 3)
                .unwrap_or(&fallback_color[3]),
        ];
        append_rustymilk_webgpu_colored_vertex(output, fan_vertices, vertex_index, color);
    };
    for index in 1..vertex_count - 1 {
        append_vertex(&mut vertices, 0);
        append_vertex(&mut vertices, index);
        append_vertex(&mut vertices, index + 1);
    }
    vertices
}

pub fn create_rustymilk_webgpu_textured_triangle_fan_vertices(
    fan_vertices: &[f64],
    fan_uvs: &[f64],
    fan_colors: &[f64],
    fallback_color: [f64; 4],
) -> Vec<f64> {
    let vertex_count = fan_vertices.len() / 2;
    if vertex_count < 3 {
        return Vec::new();
    }
    let mut vertices = Vec::with_capacity((vertex_count - 2) * 24);
    let append_vertex = |output: &mut Vec<f64>, vertex_index: usize| {
        let color_offset = vertex_index * 4;
        output.extend_from_slice(&[
            fan_vertices[vertex_index * 2],
            fan_vertices[vertex_index * 2 + 1],
            *fan_uvs.get(vertex_index * 2).unwrap_or(&0.5),
            *fan_uvs.get(vertex_index * 2 + 1).unwrap_or(&0.5),
            *fan_colors.get(color_offset).unwrap_or(&fallback_color[0]),
            *fan_colors
                .get(color_offset + 1)
                .unwrap_or(&fallback_color[1]),
            *fan_colors
                .get(color_offset + 2)
                .unwrap_or(&fallback_color[2]),
            *fan_colors
                .get(color_offset + 3)
                .unwrap_or(&fallback_color[3]),
        ]);
    };
    for index in 1..vertex_count - 1 {
        append_vertex(&mut vertices, 0);
        append_vertex(&mut vertices, index);
        append_vertex(&mut vertices, index + 1);
    }
    vertices
}

pub fn create_rustymilk_webgpu_line_segment_vertices(
    line_strip_vertices: &[f64],
    color: [f64; 4],
) -> Vec<f64> {
    let vertex_count = line_strip_vertices.len() / 2;
    if vertex_count < 2 {
        return Vec::new();
    }
    let mut vertices = Vec::with_capacity((vertex_count - 1) * 12);
    for index in 0..vertex_count - 1 {
        append_rustymilk_webgpu_colored_vertex(&mut vertices, line_strip_vertices, index, color);
        append_rustymilk_webgpu_colored_vertex(
            &mut vertices,
            line_strip_vertices,
            index + 1,
            color,
        );
    }
    vertices
}

pub fn create_rustymilk_webgpu_line_list_vertices(
    line_list_vertices: &[f64],
    color: [f64; 4],
) -> Vec<f64> {
    let vertex_count = line_list_vertices.len() / 2;
    if vertex_count < 2 {
        return Vec::new();
    }
    let mut vertices = Vec::with_capacity(vertex_count * 6);
    for index in 0..vertex_count {
        append_rustymilk_webgpu_colored_vertex(&mut vertices, line_list_vertices, index, color);
    }
    vertices
}

pub fn create_rustymilk_webgpu_shape_fill_vertices(
    shapes: &[RustyMilkIndexedEntry],
    fallback_color: [f64; 3],
) -> Vec<f64> {
    shapes
        .iter()
        .filter(|shape| !is_rustymilk_shape_textured(shape))
        .flat_map(|shape| {
            create_rustymilk_webgpu_triangle_fan_vertices(
                &create_rustymilk_shape_fill_vertices(shape),
                &create_rustymilk_shape_fill_colors(shape, fallback_color),
                [fallback_color[0], fallback_color[1], fallback_color[2], 0.6],
            )
        })
        .collect()
}

pub fn create_rustymilk_webgpu_textured_shape_vertices(
    shape: &RustyMilkIndexedEntry,
    fallback_color: [f64; 3],
) -> Vec<f64> {
    create_rustymilk_webgpu_textured_triangle_fan_vertices(
        &create_rustymilk_shape_fill_vertices(shape),
        &create_rustymilk_shape_texture_uvs(shape),
        &create_rustymilk_shape_fill_colors(shape, fallback_color),
        [fallback_color[0], fallback_color[1], fallback_color[2], 0.6],
    )
}

pub fn create_rustymilk_webgpu_shape_outline_vertices(
    shapes: &[RustyMilkIndexedEntry],
    fallback_color: [f64; 3],
) -> Vec<f64> {
    shapes
        .iter()
        .flat_map(|shape| {
            create_rustymilk_webgpu_line_segment_vertices(
                &create_rustymilk_shape_vertices(shape),
                rustymilk_shape_border_color(shape, fallback_color),
            )
        })
        .collect()
}

fn create_rustymilk_webgpu_textured_quad_vertices(
    quad_vertices: &[f64],
    quad_uvs: &[f64],
    color: [f64; 4],
) -> Vec<f64> {
    if quad_vertices.len() < 8 || quad_uvs.len() < 8 {
        return Vec::new();
    }
    let mut vertices = Vec::with_capacity(48);
    for vertex_index in [0usize, 1, 2, 0, 2, 3] {
        vertices.extend_from_slice(&[
            quad_vertices[vertex_index * 2],
            quad_vertices[vertex_index * 2 + 1],
            quad_uvs[vertex_index * 2],
            quad_uvs[vertex_index * 2 + 1],
            color[0],
            color[1],
            color[2],
            color[3],
        ]);
    }
    vertices
}

pub fn create_rustymilk_webgpu_sprite_vertices(
    sprites: &[RustyMilkIndexedEntry],
    fallback_color: [f64; 3],
) -> Vec<f64> {
    sprites
        .iter()
        .flat_map(|sprite| {
            let sprite_vertices = create_rustymilk_sprite_vertices(sprite);
            if sprite_vertices.len() < 8 {
                return Vec::new();
            }
            let triangles = vec![
                sprite_vertices[0],
                sprite_vertices[1],
                sprite_vertices[2],
                sprite_vertices[3],
                sprite_vertices[4],
                sprite_vertices[5],
                sprite_vertices[0],
                sprite_vertices[1],
                sprite_vertices[4],
                sprite_vertices[5],
                sprite_vertices[6],
                sprite_vertices[7],
            ];
            create_rustymilk_webgpu_triangle_list_vertices(
                &triangles,
                rustymilk_sprite_fill_color(sprite, fallback_color),
            )
        })
        .collect()
}

pub fn create_rustymilk_webgpu_textured_sprite_vertices(
    sprite: &RustyMilkIndexedEntry,
    fallback_color: [f64; 3],
) -> Vec<f64> {
    create_rustymilk_webgpu_textured_quad_vertices(
        &create_rustymilk_sprite_vertices(sprite),
        &create_rustymilk_sprite_texture_uvs(sprite),
        rustymilk_sprite_fill_color(sprite, fallback_color),
    )
}

pub fn create_rustymilk_webgpu_motion_vector_vertices(
    scope: &BTreeMap<String, RustyMilkValue>,
    fallback_color: [f64; 3],
) -> Vec<f64> {
    create_rustymilk_webgpu_line_list_vertices(
        &create_rustymilk_motion_vector_vertices(scope),
        [
            clamp_unit(rustymilk_scope_number(scope, "mv_r", fallback_color[0])),
            clamp_unit(rustymilk_scope_number(scope, "mv_g", fallback_color[1])),
            clamp_unit(rustymilk_scope_number(scope, "mv_b", fallback_color[2])),
            clamp_unit(rustymilk_scope_number(scope, "mv_a", 0.8)),
        ],
    )
}

pub fn create_rustymilk_webgpu_screen_border_vertices(
    scope: &BTreeMap<String, RustyMilkValue>,
    fallback_color: [f64; 3],
) -> Vec<f64> {
    let mut vertices = create_rustymilk_webgpu_triangle_list_vertices(
        &create_rustymilk_screen_border_vertices(
            rustymilk_scope_number(scope, "ob_size", 0.0),
            0.0,
        ),
        [
            clamp_unit(rustymilk_scope_number(scope, "ob_r", fallback_color[0])),
            clamp_unit(rustymilk_scope_number(scope, "ob_g", fallback_color[1])),
            clamp_unit(rustymilk_scope_number(scope, "ob_b", fallback_color[2])),
            clamp_unit(rustymilk_scope_number(scope, "ob_a", 0.0)),
        ],
    );
    vertices.extend(create_rustymilk_webgpu_triangle_list_vertices(
        &create_rustymilk_screen_border_vertices(
            rustymilk_scope_number(scope, "ib_size", 0.0),
            clamp_unit(rustymilk_scope_number(scope, "ob_size", 0.0)) * 2.0,
        ),
        [
            clamp_unit(rustymilk_scope_number(scope, "ib_r", fallback_color[0])),
            clamp_unit(rustymilk_scope_number(scope, "ib_g", fallback_color[1])),
            clamp_unit(rustymilk_scope_number(scope, "ib_b", fallback_color[2])),
            clamp_unit(rustymilk_scope_number(scope, "ib_a", 0.0)),
        ],
    ));
    vertices
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RustyMilkWebGpuTexturedBatch {
    pub first_vertex: usize,
    pub primitive_index: usize,
    pub texture_aliases: Vec<String>,
    pub texture_name: String,
    pub vertex_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RustyMilkWebGpuFrameBatches {
    pub filled_vertices: Vec<f64>,
    pub line_vertices: Vec<f64>,
    pub point_vertices: Vec<f64>,
    pub textured_batches: Vec<RustyMilkWebGpuTexturedBatch>,
    pub textured_vertices: Vec<f64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RustyMilkWebGpuCompositeBatch {
    pub blend_alpha: f64,
    pub composite_mode: String,
    pub filled_first_vertex: usize,
    pub filled_vertex_count: usize,
    pub index: usize,
    pub line_first_vertex: usize,
    pub line_vertex_count: usize,
    pub point_first_vertex: usize,
    pub point_vertex_count: usize,
    pub textured_batch_count: usize,
    pub textured_batch_first: usize,
    pub textured_first_vertex: usize,
    pub textured_vertex_count: usize,
    pub title: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RustyMilkWebGpuFrameSetBatches {
    pub composite_batches: Vec<RustyMilkWebGpuCompositeBatch>,
    pub filled_vertices: Vec<f64>,
    pub line_vertices: Vec<f64>,
    pub point_vertices: Vec<f64>,
    pub textured_batches: Vec<RustyMilkWebGpuTexturedBatch>,
    pub textured_vertices: Vec<f64>,
}

pub fn create_repeated_rustymilk_colors(vertex_count: usize, color: [f64; 4]) -> Vec<f64> {
    let mut colors = Vec::with_capacity(vertex_count * 4);
    for _ in 0..vertex_count {
        colors.extend_from_slice(&color);
    }
    colors
}

pub fn create_rustymilk_webgpu_frame_batches(
    frame: &RustyMilkFrame,
) -> RustyMilkWebGpuFrameBatches {
    let mut batches = RustyMilkWebGpuFrameBatches::default();
    for primitive in &frame.primitives {
        match primitive.mode {
            RustyMilkPrimitiveMode::Triangles => {
                batches
                    .filled_vertices
                    .extend(create_rustymilk_webgpu_triangle_list_vertices(
                        &primitive.vertices,
                        primitive.color,
                    ));
            }
            RustyMilkPrimitiveMode::TriangleFan => {
                let vertex_colors = if primitive.vertex_colors.is_empty() {
                    create_repeated_rustymilk_colors(primitive.vertices.len() / 2, primitive.color)
                } else {
                    primitive.vertex_colors.clone()
                };
                batches
                    .filled_vertices
                    .extend(create_rustymilk_webgpu_triangle_fan_vertices(
                        &primitive.vertices,
                        &vertex_colors,
                        primitive.color,
                    ));
            }
            RustyMilkPrimitiveMode::Lines => {
                batches
                    .line_vertices
                    .extend(create_rustymilk_webgpu_line_list_vertices(
                        &primitive.vertices,
                        primitive.color,
                    ));
            }
            RustyMilkPrimitiveMode::LineStrip => {
                batches
                    .line_vertices
                    .extend(create_rustymilk_webgpu_line_segment_vertices(
                        &primitive.vertices,
                        primitive.color,
                    ));
            }
            RustyMilkPrimitiveMode::Points => {
                batches
                    .point_vertices
                    .extend(create_rustymilk_webgpu_line_list_vertices(
                        &primitive.vertices,
                        primitive.color,
                    ));
            }
        }
    }

    for (primitive_index, primitive) in frame.textured_primitives.iter().enumerate() {
        let vertices = match primitive.mode {
            RustyMilkTexturedPrimitiveMode::Quad => create_rustymilk_webgpu_textured_quad_vertices(
                &primitive.vertices,
                &primitive.uvs,
                primitive.color,
            ),
            RustyMilkTexturedPrimitiveMode::TriangleFan => {
                create_rustymilk_webgpu_textured_triangle_fan_vertices(
                    &primitive.vertices,
                    &primitive.uvs,
                    &create_repeated_rustymilk_colors(
                        primitive.vertices.len() / 2,
                        primitive.color,
                    ),
                    primitive.color,
                )
            }
        };
        if vertices.is_empty() {
            continue;
        }
        let first_vertex = batches.textured_vertices.len() / 8;
        let vertex_count = vertices.len() / 8;
        batches.textured_vertices.extend(vertices);
        batches.textured_batches.push(RustyMilkWebGpuTexturedBatch {
            first_vertex,
            primitive_index,
            texture_aliases: get_rustymilk_texture_name_aliases(&primitive.texture_name),
            texture_name: primitive.texture_name.clone(),
            vertex_count,
        });
    }
    batches
}

pub fn create_rustymilk_webgpu_frame_set_batches(
    frame_set: &RustyMilkFrameSet,
) -> RustyMilkWebGpuFrameSetBatches {
    let mut batches = RustyMilkWebGpuFrameSetBatches::default();
    for entry in &frame_set.entries {
        let entry_batches = create_rustymilk_webgpu_frame_batches(&entry.frame);
        let filled_first_vertex = batches.filled_vertices.len() / 6;
        let line_first_vertex = batches.line_vertices.len() / 6;
        let point_first_vertex = batches.point_vertices.len() / 6;
        let textured_first_vertex = batches.textured_vertices.len() / 8;
        let textured_batch_first = batches.textured_batches.len();
        let textured_primitive_offset = batches.textured_batches.len();

        let filled_vertex_count = entry_batches.filled_vertices.len() / 6;
        let line_vertex_count = entry_batches.line_vertices.len() / 6;
        let point_vertex_count = entry_batches.point_vertices.len() / 6;
        let textured_vertex_count = entry_batches.textured_vertices.len() / 8;
        let textured_batch_count = entry_batches.textured_batches.len();

        batches
            .filled_vertices
            .extend(entry_batches.filled_vertices);
        batches.line_vertices.extend(entry_batches.line_vertices);
        batches.point_vertices.extend(entry_batches.point_vertices);
        batches
            .textured_vertices
            .extend(entry_batches.textured_vertices);
        batches
            .textured_batches
            .extend(entry_batches.textured_batches.into_iter().map(|batch| {
                RustyMilkWebGpuTexturedBatch {
                    first_vertex: batch.first_vertex + textured_first_vertex,
                    primitive_index: batch.primitive_index + textured_primitive_offset,
                    texture_aliases: batch.texture_aliases,
                    texture_name: batch.texture_name,
                    vertex_count: batch.vertex_count,
                }
            }));
        batches
            .composite_batches
            .push(RustyMilkWebGpuCompositeBatch {
                blend_alpha: entry.blend_alpha,
                composite_mode: entry.composite_mode.clone(),
                filled_first_vertex,
                filled_vertex_count,
                index: entry.index,
                line_first_vertex,
                line_vertex_count,
                point_first_vertex,
                point_vertex_count,
                textured_batch_count,
                textured_batch_first,
                textured_first_vertex,
                textured_vertex_count,
                title: entry.title.clone(),
            });
    }
    batches
}

fn rounded_rustymilk_buffer_sample(values: &[f64], count: usize) -> Vec<f64> {
    values
        .iter()
        .take(count)
        .map(|value| (value * 1000.0).round() / 1000.0)
        .collect()
}

pub fn rustymilk_webgpu_batch_summary_json(
    source: &str,
    time_seconds: f64,
    bass: f64,
    mid: f64,
    treble: f64,
    waveform: &[f64],
    spectrum: &[f64],
) -> String {
    let frame_set = rustymilk_frame_set_from_source_with_audio(
        source,
        time_seconds,
        bass,
        mid,
        treble,
        waveform,
        spectrum,
    );
    let frame = frame_set
        .entries
        .first()
        .map(|entry| entry.frame.clone())
        .unwrap_or_else(|| {
            rustymilk_frame(&RustyMilkPreset::default(), time_seconds, bass, mid, treble)
        });
    let batches = create_rustymilk_webgpu_frame_batches(&frame);
    let frame_set_batches = create_rustymilk_webgpu_frame_set_batches(&frame_set);
    let textured_batches = batches
        .textured_batches
        .iter()
        .map(|batch| {
            serde_json::json!({
                "firstVertex": batch.first_vertex,
                "primitiveIndex": batch.primitive_index,
                "textureAliases": batch.texture_aliases,
                "textureName": batch.texture_name,
                "vertexCount": batch.vertex_count,
            })
        })
        .collect::<Vec<_>>();
    let composite_batches = frame_set_batches
        .composite_batches
        .iter()
        .map(|batch| {
            serde_json::json!({
                "blendAlpha": batch.blend_alpha,
                "compositeMode": batch.composite_mode,
                "filledFirstVertex": batch.filled_first_vertex,
                "filledVertexCount": batch.filled_vertex_count,
                "index": batch.index,
                "lineFirstVertex": batch.line_first_vertex,
                "lineVertexCount": batch.line_vertex_count,
                "pointFirstVertex": batch.point_first_vertex,
                "pointVertexCount": batch.point_vertex_count,
                "texturedBatchCount": batch.textured_batch_count,
                "texturedBatchFirst": batch.textured_batch_first,
                "texturedFirstVertex": batch.textured_first_vertex,
                "texturedVertexCount": batch.textured_vertex_count,
                "title": batch.title,
            })
        })
        .collect::<Vec<_>>();
    let composite_entries = frame_set
        .entries
        .iter()
        .map(|entry| {
            serde_json::json!({
                "blendAlpha": entry.blend_alpha,
                "compositeMode": entry.composite_mode,
                "index": entry.index,
                "linePrimitives": entry.frame.primitives.iter().filter(|primitive| matches!(primitive.mode, RustyMilkPrimitiveMode::LineStrip | RustyMilkPrimitiveMode::Lines)).count(),
                "shapeCount": entry.frame.shape_count,
                "texturedPrimitives": entry.frame.textured_primitives.len(),
                "title": entry.title,
                "trianglePrimitives": entry.frame.primitives.iter().filter(|primitive| matches!(primitive.mode, RustyMilkPrimitiveMode::TriangleFan | RustyMilkPrimitiveMode::Triangles)).count(),
                "waveformCount": entry.frame.waveform_count,
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "backend": "webgpu",
        "frameSet": {
            "entries": composite_entries,
            "presetCount": frame_set.preset_count,
            "title": frame_set.title,
            "transitionMode": frame_set.transition_mode,
            "transitionSeconds": frame_set.transition_seconds,
        },
        "frame": {
            "bass": frame.bass,
            "fftBins": frame.fft_bins.len(),
            "linePrimitives": frame.primitives.iter().filter(|primitive| matches!(primitive.mode, RustyMilkPrimitiveMode::LineStrip | RustyMilkPrimitiveMode::Lines)).count(),
            "pointPrimitives": frame.primitives.iter().filter(|primitive| primitive.mode == RustyMilkPrimitiveMode::Points).count(),
            "q1": frame.q_registers[0],
            "shapeCount": frame.shape_count,
            "shaderTextureSamplers": frame.shader_texture_samplers.clone(),
            "texturedPrimitives": frame.textured_primitives.len(),
            "texturedTextureNames": frame.textured_primitives.iter().map(|primitive| primitive.texture_name.clone()).filter(|name| !name.is_empty()).collect::<Vec<_>>(),
            "trianglePrimitives": frame.primitives.iter().filter(|primitive| matches!(primitive.mode, RustyMilkPrimitiveMode::TriangleFan | RustyMilkPrimitiveMode::Triangles)).count(),
            "waveformBins": frame.waveform_bins.len(),
            "waveformCount": frame.waveform_count,
            "warpMeshTriangles": frame.warp_mesh.as_ref().map(|mesh| mesh.positions.len() / 6).unwrap_or_default(),
        },
        "packed": {
            "filledFloats": batches.filled_vertices.len(),
            "filledVertices": batches.filled_vertices.len() / 6,
            "filledSample": rounded_rustymilk_buffer_sample(&batches.filled_vertices, 18),
            "lineFloats": batches.line_vertices.len(),
            "lineVertices": batches.line_vertices.len() / 6,
            "lineSample": rounded_rustymilk_buffer_sample(&batches.line_vertices, 12),
            "pointFloats": batches.point_vertices.len(),
            "pointVertices": batches.point_vertices.len() / 6,
            "texturedBatches": textured_batches,
            "texturedFloats": batches.textured_vertices.len(),
            "texturedSample": rounded_rustymilk_buffer_sample(&batches.textured_vertices, 24),
            "texturedVertices": batches.textured_vertices.len() / 8,
        },
        "packedFrameSet": {
            "compositeBatches": composite_batches,
            "filledVertices": frame_set_batches.filled_vertices.len() / 6,
            "lineVertices": frame_set_batches.line_vertices.len() / 6,
            "pointVertices": frame_set_batches.point_vertices.len() / 6,
            "texturedBatches": frame_set_batches.textured_batches.len(),
            "texturedVertices": frame_set_batches.textured_vertices.len() / 8,
        },
    })
    .to_string()
}

fn append_rustymilk_quad(vertices: &mut Vec<f64>, left: f64, bottom: f64, right: f64, top: f64) {
    vertices.extend_from_slice(&[
        left, bottom, right, bottom, left, top, left, top, right, bottom, right, top,
    ]);
}

pub fn create_rustymilk_screen_border_vertices(size: f64, inset: f64) -> Vec<f64> {
    let safe_inset = inset.clamp(0.0, 0.95);
    let extent = (1.0 - safe_inset).max(0.0);
    let thickness = (size * 2.0).clamp(0.0, extent);
    if extent <= 0.0 || thickness <= 0.0 {
        return Vec::new();
    }
    let outer_left = -extent;
    let outer_right = extent;
    let outer_bottom = -extent;
    let outer_top = extent;
    let inner_left = outer_left + thickness;
    let inner_right = outer_right - thickness;
    let inner_bottom = outer_bottom + thickness;
    let inner_top = outer_top - thickness;
    if inner_left >= inner_right || inner_bottom >= inner_top {
        return vec![
            outer_left,
            outer_bottom,
            outer_right,
            outer_bottom,
            outer_right,
            outer_top,
            outer_left,
            outer_bottom,
            outer_right,
            outer_top,
            outer_left,
            outer_top,
        ];
    }
    let mut vertices = Vec::with_capacity(48);
    append_rustymilk_quad(
        &mut vertices,
        outer_left,
        outer_bottom,
        outer_right,
        inner_bottom,
    );
    append_rustymilk_quad(&mut vertices, outer_left, inner_top, outer_right, outer_top);
    append_rustymilk_quad(
        &mut vertices,
        outer_left,
        inner_bottom,
        inner_left,
        inner_top,
    );
    append_rustymilk_quad(
        &mut vertices,
        inner_right,
        inner_bottom,
        outer_right,
        inner_top,
    );
    vertices
}

pub fn create_rustymilk_motion_vector_vertices(
    scope: &BTreeMap<String, RustyMilkValue>,
) -> Vec<f64> {
    let columns = rustymilk_scope_number(scope, "mv_x", 0.0)
        .floor()
        .clamp(0.0, 128.0) as usize;
    let rows = rustymilk_scope_number(scope, "mv_y", 0.0)
        .floor()
        .clamp(0.0, 128.0) as usize;
    if columns < 1 || rows < 1 {
        return Vec::new();
    }
    let delta_x = rustymilk_scope_number(scope, "mv_dx", 0.02);
    let delta_y = rustymilk_scope_number(scope, "mv_dy", 0.02);
    let length = rustymilk_scope_number(scope, "mv_l", 0.05).max(0.0);
    let mut vertices = Vec::with_capacity(columns * rows * 4);
    for row in 0..rows {
        for column in 0..columns {
            let x = if columns == 1 {
                0.0
            } else {
                column as f64 / (columns - 1) as f64 * 2.0 - 1.0
            };
            let y = if rows == 1 {
                0.0
            } else {
                row as f64 / (rows - 1) as f64 * 2.0 - 1.0
            };
            vertices.extend_from_slice(&[
                x,
                y,
                x + delta_x * length * 2.0,
                y + delta_y * length * 2.0,
            ]);
        }
    }
    vertices
}

fn create_rustymilk_audio_samples(
    time_seconds: f64,
    bass: f64,
    mid: f64,
    treble: f64,
    count: usize,
) -> Vec<f64> {
    let count = count.max(2);
    (0..count)
        .map(|index| {
            let unit = index as f64 / (count - 1) as f64;
            (unit * std::f64::consts::TAU * 3.0 + time_seconds * 2.1).sin() * bass * 0.42
                + (unit * std::f64::consts::TAU * 7.0 + time_seconds * 1.3).sin() * mid * 0.24
                + (unit * std::f64::consts::TAU * 13.0 + time_seconds * 3.2).sin() * treble * 0.14
        })
        .collect()
}

pub fn create_rustymilk_waveform_vertices(
    samples: &[f64],
    frame_scope: &BTreeMap<String, RustyMilkValue>,
) -> Vec<f64> {
    let count = samples.len();
    if count < 2 {
        return Vec::new();
    }
    let mode = rustymilk_scope_number(frame_scope, "wave_mode", 0.0).floor() as i64;
    let scale = rustymilk_scope_number(frame_scope, "wave_scale", 1.0);
    let scale = if scale == 0.0 { 1.0 } else { scale };
    let smoothing = clamp_unit(rustymilk_scope_number(frame_scope, "wave_smoothing", 0.0));
    let center_x = rustymilk_scope_number(frame_scope, "wave_x", 0.5) * 2.0 - 1.0;
    let center_y = rustymilk_scope_number(frame_scope, "wave_y", 0.5) * 2.0 - 1.0;
    let mut vertices = Vec::with_capacity(count * 2);
    for index in 0..count {
        let sample = samples.get(index).copied().unwrap_or_default();
        let smoothed = if smoothing > 0.0 && index > 0 {
            samples.get(index - 1).copied().unwrap_or_default() * smoothing
                + sample * (1.0 - smoothing)
        } else {
            sample
        };
        let progress = if count <= 1 {
            0.0
        } else {
            index as f64 / (count - 1) as f64
        };
        let value = smoothed * scale;
        match mode {
            2 => {
                vertices.push((center_x + value).clamp(-1.0, 1.0));
                vertices.push(progress * 2.0 - 1.0);
            }
            3 => {
                let angle = progress * std::f64::consts::TAU;
                let radius = (0.35 + value * 0.18).clamp(0.0, 1.0);
                vertices.push((center_x + angle.cos() * radius).clamp(-1.0, 1.0));
                vertices.push((center_y + angle.sin() * radius).clamp(-1.0, 1.0));
            }
            1 => {
                vertices.push(progress * 2.0 - 1.0);
                vertices.push((center_y + value).clamp(-1.0, 1.0));
            }
            _ => {
                vertices.push(progress * 2.0 - 1.0);
                vertices.push((0.5 + value * 0.5).clamp(0.0, 1.0) * 2.0 - 1.0);
            }
        }
    }
    vertices
}

fn create_rustymilk_frame_primitives(
    preset: &RustyMilkPresetDocument,
    frame_scope: &BTreeMap<String, RustyMilkValue>,
    time_seconds: f64,
    bass: f64,
    mid: f64,
    treble: f64,
    waveform: &[f64],
    fallback_color: [f64; 3],
) -> Vec<RustyMilkPrimitive> {
    let generated_samples = create_rustymilk_audio_samples(time_seconds, bass, mid, treble, 192);
    let samples = if waveform.is_empty() {
        generated_samples.as_slice()
    } else {
        waveform
    };
    let mut primitives = Vec::new();
    let waveform_vertices = create_rustymilk_waveform_vertices(samples, frame_scope);
    let waveform_alpha = clamp_unit(rustymilk_scope_number(frame_scope, "wave_a", 1.0));
    if waveform_vertices.len() >= 4 && waveform_alpha > 0.0 {
        primitives.push(RustyMilkPrimitive {
            color: [
                fallback_color[0],
                fallback_color[1],
                fallback_color[2],
                waveform_alpha,
            ],
            mode: RustyMilkPrimitiveMode::LineStrip,
            vertex_colors: Vec::new(),
            vertices: waveform_vertices,
        });
    }
    for (prefix, inset, fallback_alpha) in [
        ("ob", 0.0, 0.0),
        (
            "ib",
            clamp_unit(rustymilk_scope_number(frame_scope, "ob_size", 0.0)) * 2.0,
            0.0,
        ),
    ] {
        let size = rustymilk_scope_number(frame_scope, &format!("{prefix}_size"), 0.0);
        let vertices = create_rustymilk_screen_border_vertices(size, inset);
        let alpha = clamp_unit(rustymilk_scope_number(
            frame_scope,
            &format!("{prefix}_a"),
            fallback_alpha,
        ));
        if vertices.len() >= 6 && alpha > 0.0 {
            primitives.push(RustyMilkPrimitive {
                color: [
                    clamp_unit(rustymilk_scope_number(
                        frame_scope,
                        &format!("{prefix}_r"),
                        fallback_color[0],
                    )),
                    clamp_unit(rustymilk_scope_number(
                        frame_scope,
                        &format!("{prefix}_g"),
                        fallback_color[1],
                    )),
                    clamp_unit(rustymilk_scope_number(
                        frame_scope,
                        &format!("{prefix}_b"),
                        fallback_color[2],
                    )),
                    alpha,
                ],
                mode: RustyMilkPrimitiveMode::Triangles,
                vertex_colors: Vec::new(),
                vertices,
            });
        }
    }
    let motion_vertices = create_rustymilk_motion_vector_vertices(frame_scope);
    let motion_alpha = clamp_unit(rustymilk_scope_number(frame_scope, "mv_a", 0.8));
    if motion_vertices.len() >= 4 && motion_alpha > 0.0 {
        primitives.push(RustyMilkPrimitive {
            color: [
                clamp_unit(rustymilk_scope_number(
                    frame_scope,
                    "mv_r",
                    fallback_color[0],
                )),
                clamp_unit(rustymilk_scope_number(
                    frame_scope,
                    "mv_g",
                    fallback_color[1],
                )),
                clamp_unit(rustymilk_scope_number(
                    frame_scope,
                    "mv_b",
                    fallback_color[2],
                )),
                motion_alpha,
            ],
            mode: RustyMilkPrimitiveMode::Lines,
            vertex_colors: Vec::new(),
            vertices: motion_vertices,
        });
    }
    for wave in &preset.waves {
        let evaluated = evaluate_rustymilk_wave_state(wave, frame_scope);
        if !rustymilk_entry_flag(&evaluated, &["enabled", "benabled"]) {
            continue;
        }
        let vertices = create_rustymilk_custom_wave_vertices(&evaluated, samples, frame_scope);
        if vertices.len() < 4 {
            continue;
        }
        primitives.push(RustyMilkPrimitive {
            color: [
                clamp_unit(rustymilk_entry_number(
                    &evaluated,
                    &["r"],
                    fallback_color[0],
                )),
                clamp_unit(rustymilk_entry_number(
                    &evaluated,
                    &["g"],
                    fallback_color[1],
                )),
                clamp_unit(rustymilk_entry_number(
                    &evaluated,
                    &["b"],
                    fallback_color[2],
                )),
                clamp_unit(rustymilk_entry_number(&evaluated, &["a"], 1.0)),
            ],
            mode: if rustymilk_entry_flag(&evaluated, &["dots", "busedots"]) {
                RustyMilkPrimitiveMode::Points
            } else {
                RustyMilkPrimitiveMode::LineStrip
            },
            vertex_colors: Vec::new(),
            vertices,
        });
    }
    for shape in &preset.shapes {
        let evaluated = evaluate_rustymilk_shape_state(shape, frame_scope);
        if !rustymilk_entry_flag(&evaluated, &["enabled", "benabled"]) {
            continue;
        }
        let fill_vertices = create_rustymilk_shape_fill_vertices(&evaluated);
        if fill_vertices.len() >= 6 {
            primitives.push(RustyMilkPrimitive {
                color: [
                    clamp_unit(rustymilk_entry_number(
                        &evaluated,
                        &["r"],
                        fallback_color[0],
                    )),
                    clamp_unit(rustymilk_entry_number(
                        &evaluated,
                        &["g"],
                        fallback_color[1],
                    )),
                    clamp_unit(rustymilk_entry_number(
                        &evaluated,
                        &["b"],
                        fallback_color[2],
                    )),
                    clamp_unit(rustymilk_entry_number(&evaluated, &["a"], 0.6)),
                ],
                mode: RustyMilkPrimitiveMode::TriangleFan,
                vertex_colors: create_rustymilk_shape_fill_colors(&evaluated, fallback_color),
                vertices: fill_vertices,
            });
        }
        let outline_vertices = create_rustymilk_shape_vertices(&evaluated);
        if outline_vertices.len() >= 8 {
            primitives.push(RustyMilkPrimitive {
                color: [
                    clamp_unit(rustymilk_entry_number(
                        &evaluated,
                        &["border_r", "r"],
                        fallback_color[0],
                    )),
                    clamp_unit(rustymilk_entry_number(
                        &evaluated,
                        &["border_g", "g"],
                        fallback_color[1],
                    )),
                    clamp_unit(rustymilk_entry_number(
                        &evaluated,
                        &["border_b", "b"],
                        fallback_color[2],
                    )),
                    clamp_unit(rustymilk_entry_number(&evaluated, &["border_a"], 0.85)),
                ],
                mode: RustyMilkPrimitiveMode::LineStrip,
                vertex_colors: Vec::new(),
                vertices: outline_vertices,
            });
        }
    }
    primitives
}

fn create_rustymilk_frame_textured_primitives(
    preset: &RustyMilkPresetDocument,
    frame_scope: &BTreeMap<String, RustyMilkValue>,
    fallback_color: [f64; 3],
) -> Vec<RustyMilkTexturedPrimitive> {
    let mut primitives = Vec::new();
    for shape in &preset.shapes {
        let evaluated = evaluate_rustymilk_shape_state(shape, frame_scope);
        if !is_rustymilk_shape_textured(&evaluated) {
            continue;
        }
        let vertices = create_rustymilk_shape_fill_vertices(&evaluated);
        let uvs = create_rustymilk_shape_texture_uvs(&evaluated);
        if vertices.len() >= 6 && vertices.len() == uvs.len() {
            primitives.push(RustyMilkTexturedPrimitive {
                color: [
                    clamp_unit(rustymilk_entry_number(
                        &evaluated,
                        &["r"],
                        fallback_color[0],
                    )),
                    clamp_unit(rustymilk_entry_number(
                        &evaluated,
                        &["g"],
                        fallback_color[1],
                    )),
                    clamp_unit(rustymilk_entry_number(
                        &evaluated,
                        &["b"],
                        fallback_color[2],
                    )),
                    clamp_unit(rustymilk_entry_number(&evaluated, &["a"], 0.6)),
                ],
                mode: RustyMilkTexturedPrimitiveMode::TriangleFan,
                texture_name: rustymilk_texture_name(&evaluated),
                uvs,
                vertices,
            });
        }
    }
    for sprite in &preset.sprites {
        let evaluated = evaluate_rustymilk_sprite_state(sprite, frame_scope);
        let vertices = create_rustymilk_sprite_vertices(&evaluated);
        let uvs = create_rustymilk_sprite_texture_uvs(&evaluated);
        if vertices.len() >= 8 && vertices.len() == uvs.len() {
            primitives.push(RustyMilkTexturedPrimitive {
                color: [
                    clamp_unit(rustymilk_entry_number(
                        &evaluated,
                        &["r"],
                        fallback_color[0],
                    )),
                    clamp_unit(rustymilk_entry_number(
                        &evaluated,
                        &["g"],
                        fallback_color[1],
                    )),
                    clamp_unit(rustymilk_entry_number(
                        &evaluated,
                        &["b"],
                        fallback_color[2],
                    )),
                    clamp_unit(rustymilk_entry_number(&evaluated, &["a"], 1.0)),
                ],
                mode: RustyMilkTexturedPrimitiveMode::Quad,
                texture_name: rustymilk_texture_name(&evaluated),
                uvs,
                vertices,
            });
        }
    }
    primitives
}

fn create_rustymilk_frame_primitives_and_textures_stateful(
    preset: &mut RustyMilkPresetDocument,
    frame_scope: &mut BTreeMap<String, RustyMilkValue>,
    time_seconds: f64,
    bass: f64,
    mid: f64,
    treble: f64,
    waveform: &[f64],
    spectrum: &[f64],
    fallback_color: [f64; 3],
) -> (Vec<RustyMilkPrimitive>, Vec<RustyMilkTexturedPrimitive>) {
    let generated_samples = create_rustymilk_audio_samples(time_seconds, bass, mid, treble, 192);
    let waveform_samples = if waveform.is_empty() {
        generated_samples.as_slice()
    } else {
        waveform
    };
    let spectrum_samples = if spectrum.is_empty() {
        waveform_samples
    } else {
        spectrum
    };
    let mut primitives = Vec::new();
    let mut textured_primitives = Vec::new();
    let waveform_vertices = create_rustymilk_waveform_vertices(waveform_samples, frame_scope);
    let waveform_alpha = clamp_unit(rustymilk_scope_number(frame_scope, "wave_a", 1.0));
    if waveform_vertices.len() >= 4 && waveform_alpha > 0.0 {
        primitives.push(RustyMilkPrimitive {
            color: [
                fallback_color[0],
                fallback_color[1],
                fallback_color[2],
                waveform_alpha,
            ],
            mode: RustyMilkPrimitiveMode::LineStrip,
            vertex_colors: Vec::new(),
            vertices: waveform_vertices,
        });
    }

    for (prefix, inset, fallback_alpha) in [
        ("ob", 0.0, 0.0),
        (
            "ib",
            clamp_unit(rustymilk_scope_number(frame_scope, "ob_size", 0.0)) * 2.0,
            0.0,
        ),
    ] {
        let size = rustymilk_scope_number(frame_scope, &format!("{prefix}_size"), 0.0);
        let vertices = create_rustymilk_screen_border_vertices(size, inset);
        let alpha = clamp_unit(rustymilk_scope_number(
            frame_scope,
            &format!("{prefix}_a"),
            fallback_alpha,
        ));
        if vertices.len() >= 6 && alpha > 0.0 {
            primitives.push(RustyMilkPrimitive {
                color: [
                    clamp_unit(rustymilk_scope_number(
                        frame_scope,
                        &format!("{prefix}_r"),
                        fallback_color[0],
                    )),
                    clamp_unit(rustymilk_scope_number(
                        frame_scope,
                        &format!("{prefix}_g"),
                        fallback_color[1],
                    )),
                    clamp_unit(rustymilk_scope_number(
                        frame_scope,
                        &format!("{prefix}_b"),
                        fallback_color[2],
                    )),
                    alpha,
                ],
                mode: RustyMilkPrimitiveMode::Triangles,
                vertex_colors: Vec::new(),
                vertices,
            });
        }
    }

    let motion_vertices = create_rustymilk_motion_vector_vertices(frame_scope);
    let motion_alpha = clamp_unit(rustymilk_scope_number(frame_scope, "mv_a", 0.8));
    if motion_vertices.len() >= 4 && motion_alpha > 0.0 {
        primitives.push(RustyMilkPrimitive {
            color: [
                clamp_unit(rustymilk_scope_number(
                    frame_scope,
                    "mv_r",
                    fallback_color[0],
                )),
                clamp_unit(rustymilk_scope_number(
                    frame_scope,
                    "mv_g",
                    fallback_color[1],
                )),
                clamp_unit(rustymilk_scope_number(
                    frame_scope,
                    "mv_b",
                    fallback_color[2],
                )),
                motion_alpha,
            ],
            mode: RustyMilkPrimitiveMode::Lines,
            vertex_colors: Vec::new(),
            vertices: motion_vertices,
        });
    }

    for wave in &mut preset.waves {
        let evaluated =
            evaluate_rustymilk_entry_stateful(wave, frame_scope, RUSTYMILK_WAVE_VALUE_KEYS);
        merge_rustymilk_q_registers(frame_scope, &evaluated.base_values);
        if !rustymilk_entry_flag(&evaluated, &["enabled", "benabled"]) {
            continue;
        }
        let samples = if rustymilk_entry_flag(&evaluated, &["spectrum", "bspectrum"]) {
            spectrum_samples
        } else {
            waveform_samples
        };
        let vertices = create_rustymilk_custom_wave_vertices(&evaluated, samples, frame_scope);
        if vertices.len() < 4 {
            continue;
        }
        primitives.push(RustyMilkPrimitive {
            color: [
                clamp_unit(rustymilk_entry_number(
                    &evaluated,
                    &["r"],
                    fallback_color[0],
                )),
                clamp_unit(rustymilk_entry_number(
                    &evaluated,
                    &["g"],
                    fallback_color[1],
                )),
                clamp_unit(rustymilk_entry_number(
                    &evaluated,
                    &["b"],
                    fallback_color[2],
                )),
                clamp_unit(rustymilk_entry_number(&evaluated, &["a"], 1.0)),
            ],
            mode: if rustymilk_entry_flag(&evaluated, &["dots", "busedots"]) {
                RustyMilkPrimitiveMode::Points
            } else {
                RustyMilkPrimitiveMode::LineStrip
            },
            vertex_colors: Vec::new(),
            vertices,
        });
    }

    for shape in &mut preset.shapes {
        let evaluated =
            evaluate_rustymilk_entry_stateful(shape, frame_scope, RUSTYMILK_SHAPE_VALUE_KEYS);
        merge_rustymilk_q_registers(frame_scope, &evaluated.base_values);
        if !rustymilk_entry_flag(&evaluated, &["enabled", "benabled"]) {
            continue;
        }
        let fill_vertices = create_rustymilk_shape_fill_vertices(&evaluated);
        if fill_vertices.len() >= 6 {
            if is_rustymilk_shape_textured(&evaluated) {
                let uvs = create_rustymilk_shape_texture_uvs(&evaluated);
                if fill_vertices.len() == uvs.len() {
                    textured_primitives.push(RustyMilkTexturedPrimitive {
                        color: [
                            clamp_unit(rustymilk_entry_number(
                                &evaluated,
                                &["r"],
                                fallback_color[0],
                            )),
                            clamp_unit(rustymilk_entry_number(
                                &evaluated,
                                &["g"],
                                fallback_color[1],
                            )),
                            clamp_unit(rustymilk_entry_number(
                                &evaluated,
                                &["b"],
                                fallback_color[2],
                            )),
                            clamp_unit(rustymilk_entry_number(&evaluated, &["a"], 0.6)),
                        ],
                        mode: RustyMilkTexturedPrimitiveMode::TriangleFan,
                        texture_name: rustymilk_texture_name(&evaluated),
                        uvs,
                        vertices: fill_vertices,
                    });
                }
            } else {
                primitives.push(RustyMilkPrimitive {
                    color: [
                        clamp_unit(rustymilk_entry_number(
                            &evaluated,
                            &["r"],
                            fallback_color[0],
                        )),
                        clamp_unit(rustymilk_entry_number(
                            &evaluated,
                            &["g"],
                            fallback_color[1],
                        )),
                        clamp_unit(rustymilk_entry_number(
                            &evaluated,
                            &["b"],
                            fallback_color[2],
                        )),
                        clamp_unit(rustymilk_entry_number(&evaluated, &["a"], 0.6)),
                    ],
                    mode: RustyMilkPrimitiveMode::TriangleFan,
                    vertex_colors: create_rustymilk_shape_fill_colors(&evaluated, fallback_color),
                    vertices: fill_vertices,
                });
            }
        }
        let outline_vertices = create_rustymilk_shape_vertices(&evaluated);
        if outline_vertices.len() >= 8 {
            primitives.push(RustyMilkPrimitive {
                color: [
                    clamp_unit(rustymilk_entry_number(
                        &evaluated,
                        &["border_r", "r"],
                        fallback_color[0],
                    )),
                    clamp_unit(rustymilk_entry_number(
                        &evaluated,
                        &["border_g", "g"],
                        fallback_color[1],
                    )),
                    clamp_unit(rustymilk_entry_number(
                        &evaluated,
                        &["border_b", "b"],
                        fallback_color[2],
                    )),
                    clamp_unit(rustymilk_entry_number(&evaluated, &["border_a"], 0.85)),
                ],
                mode: RustyMilkPrimitiveMode::LineStrip,
                vertex_colors: Vec::new(),
                vertices: outline_vertices,
            });
        }
    }

    for sprite in &mut preset.sprites {
        let evaluated =
            evaluate_rustymilk_entry_stateful(sprite, frame_scope, RUSTYMILK_SPRITE_VALUE_KEYS);
        merge_rustymilk_q_registers(frame_scope, &evaluated.base_values);
        let vertices = create_rustymilk_sprite_vertices(&evaluated);
        let uvs = create_rustymilk_sprite_texture_uvs(&evaluated);
        if vertices.len() >= 8 && vertices.len() == uvs.len() {
            textured_primitives.push(RustyMilkTexturedPrimitive {
                color: [
                    clamp_unit(rustymilk_entry_number(
                        &evaluated,
                        &["r"],
                        fallback_color[0],
                    )),
                    clamp_unit(rustymilk_entry_number(
                        &evaluated,
                        &["g"],
                        fallback_color[1],
                    )),
                    clamp_unit(rustymilk_entry_number(
                        &evaluated,
                        &["b"],
                        fallback_color[2],
                    )),
                    clamp_unit(rustymilk_entry_number(&evaluated, &["a"], 1.0)),
                ],
                mode: RustyMilkTexturedPrimitiveMode::Quad,
                texture_name: rustymilk_texture_name(&evaluated),
                uvs,
                vertices,
            });
        }
    }

    (primitives, textured_primitives)
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RustyMilkPresetCompatibility {
    pub shader_sections: Vec<String>,
    pub unsupported_functions: Vec<String>,
}

pub fn is_rustymilk_function_supported(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "abs"
            | "above"
            | "acos"
            | "asin"
            | "atan"
            | "atan2"
            | "below"
            | "band"
            | "bnot"
            | "bor"
            | "bxor"
            | "ceil"
            | "cos"
            | "div"
            | "equal"
            | "exp"
            | "floor"
            | "gmegabuf"
            | "get_fft"
            | "get_fft_hz"
            | "get_waveform"
            | "if"
            | "int"
            | "log"
            | "log10"
            | "max"
            | "megabuf"
            | "min"
            | "mod"
            | "pow"
            | "rand"
            | "sign"
            | "sigmoid"
            | "sin"
            | "sqr"
            | "sqrt"
            | "tan"
    )
}

fn collect_rustymilk_functions(text: &str, unsupported: &mut Vec<String>) {
    let chars = text.chars().collect::<Vec<_>>();
    let mut index = 0usize;
    while index < chars.len() {
        if !(chars[index].is_ascii_alphabetic() || chars[index] == '_') {
            index += 1;
            continue;
        }
        let start = index;
        index += 1;
        while index < chars.len()
            && (chars[index].is_ascii_alphanumeric() || chars[index] == '_' || chars[index] == '.')
        {
            index += 1;
        }
        let name = chars[start..index]
            .iter()
            .collect::<String>()
            .to_ascii_lowercase();
        let mut lookahead = index;
        while lookahead < chars.len() && chars[lookahead].is_whitespace() {
            lookahead += 1;
        }
        if lookahead < chars.len()
            && chars[lookahead] == '('
            && !is_rustymilk_function_supported(&name)
            && !unsupported.contains(&name)
        {
            unsupported.push(name);
        }
    }
}

fn collect_rustymilk_equation_functions(
    equations: &RustyMilkEquations,
    unsupported: &mut Vec<String>,
) {
    collect_rustymilk_functions(&equations.init, unsupported);
    collect_rustymilk_functions(&equations.frame, unsupported);
    collect_rustymilk_functions(&equations.per_frame, unsupported);
    collect_rustymilk_functions(&equations.per_pixel, unsupported);
    collect_rustymilk_functions(&equations.point, unsupported);
}

pub fn analyze_rustymilk_preset_compatibility(
    preset: &RustyMilkPresetDocument,
) -> RustyMilkPresetCompatibility {
    let mut unsupported_functions = Vec::new();
    collect_rustymilk_equation_functions(&preset.equations, &mut unsupported_functions);
    for shape in &preset.shapes {
        collect_rustymilk_equation_functions(&shape.equations, &mut unsupported_functions);
    }
    for sprite in &preset.sprites {
        collect_rustymilk_equation_functions(&sprite.equations, &mut unsupported_functions);
    }
    for wave in &preset.waves {
        collect_rustymilk_equation_functions(&wave.equations, &mut unsupported_functions);
    }
    unsupported_functions.sort();

    let mut shader_sections = Vec::new();
    if !preset.warp_shader.trim().is_empty()
        && !analyze_rustymilk_shader_support(&preset.warp_shader).supported
    {
        shader_sections.push("warp_shader".to_string());
    }
    if !preset.comp_shader.trim().is_empty()
        && !analyze_rustymilk_shader_support(&preset.comp_shader).supported
    {
        shader_sections.push("comp_shader".to_string());
    }

    RustyMilkPresetCompatibility {
        shader_sections,
        unsupported_functions,
    }
}

pub fn rustymilk_compatibility_error(report: &RustyMilkPresetCompatibility) -> String {
    let mut messages = Vec::new();
    if !report.unsupported_functions.is_empty() {
        messages.push(format!(
            "unsupported functions: {}",
            report.unsupported_functions.join(", ")
        ));
    }
    if !report.shader_sections.is_empty() {
        messages.push(format!(
            "shader translation pending: {}",
            report.shader_sections.join(", ")
        ));
    }
    if messages.is_empty() {
        String::new()
    } else {
        format!("RustyMilk preset has {}.", messages.join("; "))
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RustyMilkPresetMetrics {
    pub max_q_register_index: usize,
    pub q_registers: Vec<String>,
    pub q_register_count: usize,
    pub shape_count: usize,
    pub sprite_count: usize,
    pub wave_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RustyMilkCompatibilityPresetReport {
    pub error: String,
    pub index: usize,
    pub metrics: RustyMilkPresetMetrics,
    pub shader_sections: Vec<String>,
    pub title: String,
    pub unsupported_functions: Vec<String>,
    pub webgpu_shader_sections: Vec<String>,
    pub webgpu_supported: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RustyMilkCompatibilityEntry {
    pub file_name: String,
    pub format: String,
    pub id: String,
    pub metrics: RustyMilkCompatibilitySummary,
    pub preset_count: usize,
    pub preset_reports: Vec<RustyMilkCompatibilityPresetReport>,
    pub shader_sections: Vec<String>,
    pub supported: bool,
    pub unsupported_functions: Vec<String>,
    pub webgpu_shader_sections: Vec<String>,
    pub webgpu_supported: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RustyMilkCompatibilitySummary {
    pub max_q_register_index: usize,
    pub max_shape_count: usize,
    pub max_sprite_count: usize,
    pub max_wave_count: usize,
    pub preset_count: usize,
    pub q_registers: Vec<String>,
    pub supported_count: usize,
    pub total_count: usize,
    pub total_shapes: usize,
    pub total_sprites: usize,
    pub total_waves: usize,
    pub unsupported_count: usize,
    pub unsupported_functions: Vec<String>,
    pub unsupported_shader_sections: Vec<String>,
    pub webgpu_supported_count: usize,
    pub webgpu_unsupported_count: usize,
    pub webgpu_unsupported_shader_sections: Vec<String>,
}

fn rustymilk_entry_has_content(entry: &RustyMilkIndexedEntry) -> bool {
    !entry.base_values.is_empty() || entry.equations != RustyMilkEquations::default()
}

fn merge_rustymilk_unique(mut values: Vec<String>) -> Vec<String> {
    values.retain(|value| !value.is_empty());
    values.sort();
    values.dedup();
    values
}

fn collect_q_registers_from_text(text: &str, registers: &mut Vec<String>) {
    let chars = text.chars().collect::<Vec<_>>();
    let mut index = 0usize;
    while index < chars.len() {
        if chars[index].to_ascii_lowercase() != 'q' {
            index += 1;
            continue;
        }
        let start = index + 1;
        let mut end = start;
        while end < chars.len() && chars[end].is_ascii_digit() {
            end += 1;
        }
        if end > start {
            if let Ok(number) = chars[start..end]
                .iter()
                .collect::<String>()
                .parse::<usize>()
            {
                if (1..=64).contains(&number) {
                    let register = format!("q{number}");
                    if !registers.contains(&register) {
                        registers.push(register);
                    }
                }
            }
        }
        index = end.max(index + 1);
    }
}

fn collect_q_registers_from_equations(equations: &RustyMilkEquations, registers: &mut Vec<String>) {
    collect_q_registers_from_text(&equations.init, registers);
    collect_q_registers_from_text(&equations.frame, registers);
    collect_q_registers_from_text(&equations.per_frame, registers);
    collect_q_registers_from_text(&equations.per_pixel, registers);
    collect_q_registers_from_text(&equations.point, registers);
}

fn collect_rustymilk_q_registers(preset: &RustyMilkPresetDocument) -> Vec<String> {
    let mut registers = Vec::new();
    for key in preset.base_values.keys() {
        collect_q_registers_from_text(key, &mut registers);
    }
    collect_q_registers_from_equations(&preset.equations, &mut registers);
    collect_q_registers_from_text(&preset.warp_shader, &mut registers);
    collect_q_registers_from_text(&preset.comp_shader, &mut registers);
    for entry in preset
        .shapes
        .iter()
        .chain(preset.sprites.iter())
        .chain(preset.waves.iter())
    {
        for key in entry.base_values.keys() {
            collect_q_registers_from_text(key, &mut registers);
        }
        collect_q_registers_from_equations(&entry.equations, &mut registers);
    }
    registers.sort_by_key(|register| {
        register
            .strip_prefix('q')
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or_default()
    });
    registers
}

fn max_q_register_index(registers: &[String]) -> usize {
    registers
        .iter()
        .filter_map(|register| register.strip_prefix('q')?.parse::<usize>().ok())
        .max()
        .unwrap_or_default()
}

pub fn rustymilk_preset_metrics(preset: &RustyMilkPresetDocument) -> RustyMilkPresetMetrics {
    let q_registers = collect_rustymilk_q_registers(preset);
    RustyMilkPresetMetrics {
        max_q_register_index: max_q_register_index(&q_registers),
        q_register_count: q_registers.len(),
        q_registers,
        shape_count: preset
            .shapes
            .iter()
            .filter(|entry| rustymilk_entry_has_content(entry))
            .count(),
        sprite_count: preset
            .sprites
            .iter()
            .filter(|entry| rustymilk_entry_has_content(entry))
            .count(),
        wave_count: preset
            .waves
            .iter()
            .filter(|entry| rustymilk_entry_has_content(entry))
            .count(),
    }
}

fn webgpu_shader_sections(preset: &RustyMilkPresetDocument) -> Vec<String> {
    let mut sections = Vec::new();
    if !preset.warp_shader.trim().is_empty()
        && !analyze_rustymilk_webgpu_shader_support(&preset.warp_shader).supported
    {
        sections.push("warp_shader".to_string());
    }
    if !preset.comp_shader.trim().is_empty()
        && !analyze_rustymilk_webgpu_shader_support(&preset.comp_shader).supported
    {
        sections.push("comp_shader".to_string());
    }
    sections
}

fn merge_rustymilk_metrics(metrics: &[RustyMilkPresetMetrics]) -> RustyMilkCompatibilitySummary {
    let mut summary = RustyMilkCompatibilitySummary::default();
    for metric in metrics {
        summary.max_q_register_index = summary
            .max_q_register_index
            .max(metric.max_q_register_index);
        summary.max_shape_count = summary.max_shape_count.max(metric.shape_count);
        summary.max_sprite_count = summary.max_sprite_count.max(metric.sprite_count);
        summary.max_wave_count = summary.max_wave_count.max(metric.wave_count);
        summary.total_shapes += metric.shape_count;
        summary.total_sprites += metric.sprite_count;
        summary.total_waves += metric.wave_count;
        summary.q_registers.extend(metric.q_registers.clone());
    }
    summary.q_registers = merge_rustymilk_unique(summary.q_registers);
    summary.q_registers.sort_by_key(|register| {
        register
            .strip_prefix('q')
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or_default()
    });
    summary
}

pub fn build_rustymilk_compatibility_entry(
    id: &str,
    file_name: &str,
    source: &str,
    force_milk2: bool,
) -> RustyMilkCompatibilityEntry {
    let parsed = parse_rustymilk_preset_set(source, force_milk2);
    let preset_reports = parsed
        .presets
        .iter()
        .map(|preset| {
            let report = analyze_rustymilk_preset_compatibility(preset);
            let webgpu_shader_sections = webgpu_shader_sections(preset);
            RustyMilkCompatibilityPresetReport {
                error: rustymilk_compatibility_error(&report),
                index: preset.index,
                metrics: rustymilk_preset_metrics(preset),
                shader_sections: report.shader_sections,
                title: preset.title.clone(),
                unsupported_functions: report.unsupported_functions,
                webgpu_supported: webgpu_shader_sections.is_empty(),
                webgpu_shader_sections,
            }
        })
        .collect::<Vec<_>>();
    let metrics = merge_rustymilk_metrics(
        &preset_reports
            .iter()
            .map(|report| report.metrics.clone())
            .collect::<Vec<_>>(),
    );
    let shader_sections = merge_rustymilk_unique(
        preset_reports
            .iter()
            .flat_map(|report| report.shader_sections.clone())
            .collect(),
    );
    let unsupported_functions = merge_rustymilk_unique(
        preset_reports
            .iter()
            .flat_map(|report| report.unsupported_functions.clone())
            .collect(),
    );
    let webgpu_shader_sections = merge_rustymilk_unique(
        preset_reports
            .iter()
            .flat_map(|report| report.webgpu_shader_sections.clone())
            .collect(),
    );
    let supported = preset_reports.iter().all(|report| report.error.is_empty());
    let webgpu_supported = preset_reports.iter().all(|report| report.webgpu_supported);
    RustyMilkCompatibilityEntry {
        file_name: file_name.to_string(),
        format: parsed.format,
        id: if id.is_empty() {
            if file_name.is_empty() {
                "preset"
            } else {
                file_name
            }
            .to_string()
        } else {
            id.to_string()
        },
        metrics,
        preset_count: preset_reports.len(),
        preset_reports,
        shader_sections,
        supported,
        unsupported_functions,
        webgpu_shader_sections,
        webgpu_supported,
    }
}

pub fn summarize_rustymilk_compatibility_matrix(
    entries: &[RustyMilkCompatibilityEntry],
) -> RustyMilkCompatibilitySummary {
    let mut summary = RustyMilkCompatibilitySummary::default();
    for entry in entries {
        summary.max_q_register_index = summary
            .max_q_register_index
            .max(entry.metrics.max_q_register_index);
        summary.max_shape_count = summary.max_shape_count.max(entry.metrics.max_shape_count);
        summary.max_sprite_count = summary.max_sprite_count.max(entry.metrics.max_sprite_count);
        summary.max_wave_count = summary.max_wave_count.max(entry.metrics.max_wave_count);
        summary.preset_count += entry.preset_count;
        summary.supported_count += usize::from(entry.supported);
        summary.total_count += 1;
        summary.unsupported_count += usize::from(!entry.supported);
        summary.webgpu_supported_count += usize::from(entry.webgpu_supported);
        summary.webgpu_unsupported_count += usize::from(!entry.webgpu_supported);
        summary
            .q_registers
            .extend(entry.metrics.q_registers.clone());
        summary
            .unsupported_functions
            .extend(entry.unsupported_functions.clone());
        summary
            .unsupported_shader_sections
            .extend(entry.shader_sections.clone());
        summary
            .webgpu_unsupported_shader_sections
            .extend(entry.webgpu_shader_sections.clone());
    }
    summary.q_registers = merge_rustymilk_unique(summary.q_registers);
    summary.q_registers.sort_by_key(|register| {
        register
            .strip_prefix('q')
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or_default()
    });
    summary.unsupported_functions = merge_rustymilk_unique(summary.unsupported_functions);
    summary.unsupported_shader_sections =
        merge_rustymilk_unique(summary.unsupported_shader_sections);
    summary.webgpu_unsupported_shader_sections =
        merge_rustymilk_unique(summary.webgpu_unsupported_shader_sections);
    summary
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RustyMilkShaderProgram {
    pub declarations: Vec<String>,
    pub expression: String,
    pub texture_samplers: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RustyMilkShaderSupport {
    pub supported: bool,
}

fn strip_rustymilk_shader_comments(source: &str) -> String {
    let mut output = String::new();
    let mut chars = source.chars().peekable();
    let mut in_block = false;
    while let Some(ch) = chars.next() {
        if in_block {
            if ch == '*' && chars.peek() == Some(&'/') {
                let _ = chars.next();
                in_block = false;
            }
            continue;
        }
        if ch == '/' && chars.peek() == Some(&'*') {
            let _ = chars.next();
            in_block = true;
            continue;
        }
        if ch == '/' && chars.peek() == Some(&'/') {
            for next in chars.by_ref() {
                if next == '\n' {
                    output.push('\n');
                    break;
                }
            }
            continue;
        }
        output.push(ch);
    }
    output.trim().to_string()
}

fn unwrap_rustymilk_shader_body(source: &str) -> String {
    let mut source = strip_rustymilk_shader_comments(source);
    let lower = source.to_ascii_lowercase();
    if let Some(index) = lower.find("shader_body") {
        source.replace_range(index..index + "shader_body".len(), "");
    }
    let trimmed = source.trim();
    let trimmed = trimmed.strip_prefix('{').unwrap_or(trimmed).trim();
    let trimmed = trimmed.strip_suffix('}').unwrap_or(trimmed).trim();
    trimmed.to_string()
}

fn normalize_simple_rustymilk_conditional_return(source: &str) -> String {
    let unwrapped = unwrap_rustymilk_shader_body(source);
    let compact = unwrapped
        .replace('{', " ")
        .replace('}', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let lower = compact.to_ascii_lowercase();
    if !lower.starts_with("if") || !lower.contains(" else ") {
        return source.to_string();
    }
    let Some(condition_start) = compact.find('(') else {
        return source.to_string();
    };
    let Some(condition_end) = compact[condition_start + 1..].find(')') else {
        return source.to_string();
    };
    let condition_end = condition_start + 1 + condition_end;
    let condition = compact[condition_start + 1..condition_end].trim();
    let rest = compact[condition_end + 1..].trim();
    let lower_rest = rest.to_ascii_lowercase();
    let Some(else_index) = lower_rest.find(" else ") else {
        return source.to_string();
    };
    let true_part = rest[..else_index].trim();
    let false_part = rest[else_index + " else ".len()..].trim();
    let extract_ret = |part: &str| -> Option<String> {
        let part = part.trim();
        let lower = part.to_ascii_lowercase();
        let value = lower.strip_prefix("ret")?;
        let value = value.trim_start();
        let value = value.strip_prefix('=')?.trim();
        Some(value.trim_end_matches(';').trim().to_string())
    };
    let Some(true_ret) = extract_ret(true_part) else {
        return source.to_string();
    };
    let Some(false_ret) = extract_ret(false_part) else {
        return source.to_string();
    };
    format!("ret = ({condition}) ? ({true_ret}) : ({false_ret});")
}

fn is_rustymilk_main_sampler(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "previousframe" | "sampler_main" | "sampler_fc_main" | "sampler_sampler_main"
    )
}

pub fn get_rustymilk_shader_texture_samplers(source: &str) -> Vec<String> {
    let cleaned = strip_rustymilk_shader_comments(source);
    let mut samplers = Vec::new();
    for marker in ["tex2D(", "tex("] {
        let mut rest = cleaned.as_str();
        while let Some(index) = rest.to_ascii_lowercase().find(&marker.to_ascii_lowercase()) {
            let after = &rest[index + marker.len()..];
            let sampler = after
                .trim_start()
                .chars()
                .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
                .collect::<String>();
            if !sampler.is_empty()
                && !is_rustymilk_main_sampler(&sampler)
                && !samplers.contains(&sampler)
            {
                samplers.push(sampler);
            }
            rest = &after[after.find(',').unwrap_or(after.len())..];
        }
    }
    samplers.truncate(4);
    samplers
}

fn normalize_rustymilk_shader_expression(expression: &str) -> String {
    expression
        .replace("float4(", "vec4(")
        .replace("float3(", "vec3(")
        .replace("float2(", "vec2(")
        .replace("saturate(", "clamp01(")
        .replace("lerp(", "mix(")
        .replace("frac(", "fract(")
        .replace("fmod(", "mod(")
        .replace("rsqrt(", "inversesqrt(")
        .replace("atan2(", "atan(")
}

fn normalize_rustymilk_shader_type(shader_type: &str) -> String {
    shader_type
        .to_ascii_lowercase()
        .replace("float2", "vec2")
        .replace("float3", "vec3")
        .replace("float4", "vec4")
}

fn normalize_rustymilk_shader_source(source: &str, texture_samplers: &[String]) -> String {
    let mut normalized =
        unwrap_rustymilk_shader_body(&normalize_simple_rustymilk_conditional_return(source));
    for sampler in ["tex2D", "tex"] {
        loop {
            let Some(index) = normalized
                .to_ascii_lowercase()
                .find(&format!("{}(", sampler.to_ascii_lowercase()))
            else {
                break;
            };
            let start = index + sampler.len() + 1;
            let after = &normalized[start..];
            let name = after
                .trim_start()
                .chars()
                .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
                .collect::<String>();
            if name.is_empty() {
                break;
            }
            let whitespace = after.len() - after.trim_start().len();
            let name_start = start + whitespace;
            let name_end = name_start + name.len();
            let replacement = if is_rustymilk_main_sampler(&name) {
                "previousFrame".to_string()
            } else if let Some(texture_index) =
                texture_samplers.iter().position(|value| value == &name)
            {
                format!("shaderTexture{texture_index}")
            } else {
                name.clone()
            };
            normalized.replace_range(index..name_end, &format!("texture({replacement}"));
        }
    }
    normalized
}

fn is_safe_rustymilk_shader_expression(expression: &str) -> bool {
    if expression.trim().is_empty() {
        return false;
    }
    if !expression.chars().all(|ch| {
        ch.is_ascii_alphanumeric()
            || matches!(
                ch,
                '_' | '.'
                    | ','
                    | '+'
                    | '-'
                    | '*'
                    | '/'
                    | '%'
                    | '<'
                    | '>'
                    | '='
                    | '!'
                    | '&'
                    | '|'
                    | '^'
                    | '~'
                    | '?'
                    | ':'
                    | '('
                    | ')'
                    | ' '
            )
    }) {
        return false;
    }
    if expression.contains("texture(")
        && !(expression.contains("previousFrame") || expression.contains("shaderTexture"))
    {
        return false;
    }
    true
}

fn split_rustymilk_shader_declaration(statement: &str) -> Option<(&str, &str, &str)> {
    for shader_type in [
        "float4", "float3", "float2", "float", "vec4", "vec3", "vec2",
    ] {
        let Some(rest) = statement.strip_prefix(shader_type) else {
            continue;
        };
        let rest = rest.trim_start();
        let Some((name, expression)) = rest.split_once('=') else {
            return None;
        };
        let name = name.trim();
        if !name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        {
            return None;
        }
        return Some((shader_type, name, expression.trim()));
    }
    None
}

fn split_rustymilk_shader_assignment(statement: &str) -> Option<(&str, &str, &str)> {
    for operator in ["+=", "-=", "*=", "/=", "="] {
        let Some((name, expression)) = statement.split_once(operator) else {
            continue;
        };
        let name = name.trim();
        if !name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        {
            return None;
        }
        return Some((name, operator, expression.trim()));
    }
    None
}

fn parse_rustymilk_shader_program(source: &str) -> Option<RustyMilkShaderProgram> {
    let normalized_source = normalize_simple_rustymilk_conditional_return(source);
    let lowered = normalized_source.to_ascii_lowercase();
    if lowered.contains("for (")
        || lowered.contains("while (")
        || lowered.contains("float3x")
        || lowered.contains("float4x")
        || lowered.contains("mul(")
        || lowered.contains("sampler2d ")
    {
        return None;
    }
    if lowered.starts_with("if") {
        return None;
    }
    let texture_samplers = get_rustymilk_shader_texture_samplers(&normalized_source);
    let cleaned = normalize_rustymilk_shader_source(&normalized_source, &texture_samplers);
    let mut declarations = Vec::new();
    let mut mutable_variables = Vec::new();
    let mut expression = String::new();

    for statement in cleaned
        .split(';')
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if let Some(ret_expression) = statement
            .strip_prefix("ret")
            .and_then(|rest| rest.trim_start().strip_prefix('='))
        {
            if !expression.is_empty() {
                return None;
            }
            expression = normalize_rustymilk_shader_expression(ret_expression.trim());
            continue;
        }
        if !expression.is_empty() {
            return None;
        }
        if let Some((shader_type, name, declaration_expression)) =
            split_rustymilk_shader_declaration(statement)
        {
            let declaration_expression =
                normalize_rustymilk_shader_expression(declaration_expression);
            if !is_safe_rustymilk_shader_expression(&declaration_expression) {
                return None;
            }
            mutable_variables.push(name.to_string());
            declarations.push(format!(
                "{} {name} = {declaration_expression};",
                normalize_rustymilk_shader_type(shader_type)
            ));
            continue;
        }
        if let Some((name, operator, assignment_expression)) =
            split_rustymilk_shader_assignment(statement)
        {
            if !mutable_variables.iter().any(|value| value == name) {
                return None;
            }
            let assignment_expression =
                normalize_rustymilk_shader_expression(assignment_expression);
            if !is_safe_rustymilk_shader_expression(&assignment_expression) {
                return None;
            }
            declarations.push(format!("{name} {operator} {assignment_expression};"));
            continue;
        }
        return None;
    }

    if !is_safe_rustymilk_shader_expression(&expression) {
        return None;
    }
    Some(RustyMilkShaderProgram {
        declarations,
        expression,
        texture_samplers,
    })
}

pub fn translate_rustymilk_shader_expression(source: &str) -> String {
    parse_rustymilk_shader_program(source)
        .map(|program| program.expression)
        .unwrap_or_default()
}

pub fn create_translated_rustymilk_fragment_shader(source: &str) -> String {
    let Some(program) = parse_rustymilk_shader_program(source) else {
        return String::new();
    };
    let uniforms = (1..=64)
        .map(|index| format!("uniform float q{index};"))
        .chain(
            ["bass", "bass_att", "mid", "mid_att", "treb", "treb_att"]
                .into_iter()
                .map(|name| format!("uniform float {name};")),
        )
        .collect::<Vec<_>>()
        .join("\n");
    let texture_uniforms = program
        .texture_samplers
        .iter()
        .enumerate()
        .map(|(index, _)| format!("uniform sampler2D shaderTexture{index};"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"#version 300 es
precision highp float;
uniform vec3 color;
uniform sampler2D previousFrame;
{texture_uniforms}
uniform float feedback;
uniform float outputAlpha;
uniform float time;
uniform float sampleRate;
uniform float fftBins[64];
uniform float waveformBins[64];
uniform vec2 resolution;
uniform vec2 pixelSize;
uniform float aspect;
uniform vec4 texsize;
{uniforms}
in vec2 uv;
out vec4 outColor;
float clamp01(float value) {{ return clamp(value, 0.0, 1.0); }}
vec2 clamp01(vec2 value) {{ return clamp(value, vec2(0.0), vec2(1.0)); }}
vec3 clamp01(vec3 value) {{ return clamp(value, vec3(0.0), vec3(1.0)); }}
vec4 clamp01(vec4 value) {{ return clamp(value, vec4(0.0), vec4(1.0)); }}
float get_fft(float position) {{ int index = int(clamp(position, 0.0, 1.0) * 63.0); return fftBins[index]; }}
float get_fft_hz(float hz) {{ float nyquist = max(sampleRate * 0.5, 1.0); return get_fft(hz / nyquist); }}
float get_waveform(float position) {{ int index = int(clamp(position, 0.0, 1.0) * 63.0); return waveformBins[index]; }}
void main() {{
  float x = uv.x;
  float y = uv.y;
  vec2 centeredUv = uv - vec2(0.5);
  float rad = length(centeredUv);
  float ang = atan(centeredUv.y, centeredUv.x);
  {}
  vec3 ret = vec3({});
  vec3 previous = texture(previousFrame, clamp(uv, vec2(0.0), vec2(1.0))).rgb;
  outColor = vec4(mix(ret, previous, feedback), outputAlpha);
}}"#,
        program.declarations.join("\n  "),
        program.expression
    )
}

fn split_top_level_rustymilk_ternary(expression: &str) -> Option<(&str, &str, &str)> {
    let chars = expression.char_indices().collect::<Vec<_>>();
    let mut depth = 0i32;
    let mut question_index = None;
    for (index, ch) in &chars {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            '?' if depth == 0 => {
                question_index = Some(*index);
                break;
            }
            _ => {}
        }
    }
    let question_index = question_index?;
    let mut depth = 0i32;
    let mut nested = 0i32;
    for (index, ch) in chars
        .into_iter()
        .filter(|(index, _)| *index > question_index)
    {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            '?' if depth == 0 => nested += 1,
            ':' if depth == 0 && nested == 0 => {
                return Some((
                    expression[..question_index].trim(),
                    expression[question_index + 1..index].trim(),
                    expression[index + 1..].trim(),
                ));
            }
            ':' if depth == 0 => nested -= 1,
            _ => {}
        }
    }
    None
}

fn normalize_rustymilk_wgsl_expression(expression: &str) -> String {
    if let Some((condition, when_true, when_false)) = split_top_level_rustymilk_ternary(expression)
    {
        return format!(
            "select({}, {}, {})",
            normalize_rustymilk_wgsl_expression(when_false),
            normalize_rustymilk_wgsl_expression(when_true),
            normalize_rustymilk_wgsl_expression(condition)
        );
    }
    expression
        .replace(
            "texture(previousFrame,",
            "textureSample(previousFrame, previousSampler,",
        )
        .replace(
            "texture(shaderTexture0,",
            "textureSample(shaderTexture0, shaderTextureSampler,",
        )
        .replace(
            "texture(shaderTexture1,",
            "textureSample(shaderTexture1, shaderTextureSampler,",
        )
        .replace(
            "texture(shaderTexture2,",
            "textureSample(shaderTexture2, shaderTextureSampler,",
        )
        .replace(
            "texture(shaderTexture3,",
            "textureSample(shaderTexture3, shaderTextureSampler,",
        )
        .replace("vec2(", "vec2f(")
        .replace("vec3(", "vec3f(")
        .replace("vec4(", "vec4f(")
        .replace("clamp01(vec2f(", "clamp01v2(vec2f(")
        .replace("clamp01(vec3f(", "clamp01v3(vec3f(")
        .replace("clamp01(vec4f(", "clamp01v4(vec4f(")
        .replace("atan(", "atan2(")
}

fn normalize_rustymilk_wgsl_declaration(declaration: &str) -> String {
    let declaration = normalize_rustymilk_wgsl_expression(declaration);
    for prefix in ["vec2 ", "vec3 ", "vec4 ", "float "] {
        if let Some(rest) = declaration.strip_prefix(prefix) {
            return format!("var {rest}");
        }
    }
    declaration
}

pub fn create_translated_rustymilk_wgsl_shader(source: &str) -> String {
    let Some(program) = parse_rustymilk_shader_program(source) else {
        return String::new();
    };
    if std::iter::once(&program.expression)
        .chain(program.declarations.iter())
        .map(|statement| normalize_rustymilk_wgsl_expression(statement))
        .any(|statement| {
            statement.contains('&')
                || statement.contains('|')
                || statement.contains('^')
                || statement.contains('~')
                || statement.contains('?')
        })
    {
        return String::new();
    }
    let q_fields = (1..=64)
        .map(|index| format!("  q{index}: f32,"))
        .collect::<Vec<_>>()
        .join("\n");
    let q_locals = (1..=64)
        .map(|index| format!("  let q{index} = uniforms.q{index};"))
        .collect::<Vec<_>>()
        .join("\n");
    let texture_declarations = program
        .texture_samplers
        .iter()
        .enumerate()
        .map(|(index, _)| {
            format!(
                "@group(0) @binding({}) var shaderTexture{index}: texture_2d<f32>;",
                index + 3
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let declarations = program
        .declarations
        .iter()
        .map(|declaration| format!("  {}", normalize_rustymilk_wgsl_declaration(declaration)))
        .collect::<Vec<_>>()
        .join("\n");
    let expression = normalize_rustymilk_wgsl_expression(&program.expression);
    format!(
        r#"struct Uniforms {{
  color: vec4f,
  time: f32,
  bass: f32,
  mid: f32,
  treb: f32,
  bass_att: f32,
  mid_att: f32,
  treb_att: f32,
  feedback: f32,
  outputAlpha: f32,
  sampleRate: f32,
{q_fields}
  fft63: f32,
  waveform63: f32,
}};
@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var previousFrame: texture_2d<f32>;
@group(0) @binding(2) var previousSampler: sampler;
@group(0) @binding(7) var shaderTextureSampler: sampler;
{texture_declarations}
fn get_fft(position: f32) -> f32 {{ return uniforms.fft63; }}
fn get_fft_hz(hz: f32) -> f32 {{ return get_fft(hz / max(uniforms.sampleRate * 0.5, 1.0)); }}
fn get_waveform(position: f32) -> f32 {{ return uniforms.waveform63; }}
@fragment
fn fragmentMain() -> @location(0) vec4f {{
  let uv = vec2f(0.5);
  let color = uniforms.color.rgb;
  let time = uniforms.time;
  let bass = uniforms.bass;
  let bass_att = uniforms.bass_att;
{q_locals}
{declarations}
  let ret = vec3f({expression});
  return vec4f(ret, uniforms.outputAlpha);
}}"#
    )
}

pub fn analyze_rustymilk_shader_support(source: &str) -> RustyMilkShaderSupport {
    RustyMilkShaderSupport {
        supported: source.trim().is_empty()
            || !create_translated_rustymilk_fragment_shader(source).is_empty(),
    }
}

pub fn analyze_rustymilk_webgpu_shader_support(source: &str) -> RustyMilkShaderSupport {
    RustyMilkShaderSupport {
        supported: source.trim().is_empty()
            || !create_translated_rustymilk_wgsl_shader(source).is_empty(),
    }
}

#[derive(Clone, Debug, PartialEq)]
enum RustyMilkToken {
    Ident(String),
    Number(f64),
    Op(String),
}

fn tokenize_rustymilk_expression(expression: &str) -> Result<Vec<RustyMilkToken>, String> {
    let chars = expression.chars().collect::<Vec<_>>();
    let mut tokens = Vec::new();
    let mut index = 0usize;
    while index < chars.len() {
        let ch = chars[index];
        if ch.is_whitespace() {
            index += 1;
            continue;
        }
        if ch.is_ascii_alphabetic() || ch == '_' {
            let start = index;
            index += 1;
            while index < chars.len()
                && (chars[index].is_ascii_alphanumeric()
                    || chars[index] == '_'
                    || chars[index] == '.')
            {
                index += 1;
            }
            tokens.push(RustyMilkToken::Ident(
                chars[start..index]
                    .iter()
                    .collect::<String>()
                    .to_ascii_lowercase(),
            ));
            continue;
        }
        if ch.is_ascii_digit() || ch == '.' {
            let start = index;
            index += 1;
            while index < chars.len() && (chars[index].is_ascii_digit() || chars[index] == '.') {
                index += 1;
            }
            if index < chars.len() && matches!(chars[index], 'e' | 'E') {
                index += 1;
                if index < chars.len() && matches!(chars[index], '+' | '-') {
                    index += 1;
                }
                while index < chars.len() && chars[index].is_ascii_digit() {
                    index += 1;
                }
            }
            let value = chars[start..index]
                .iter()
                .collect::<String>()
                .parse::<f64>()
                .map_err(|_| format!("Unsupported RustyMilk expression syntax: {expression}"))?;
            tokens.push(RustyMilkToken::Number(value));
            continue;
        }
        let two = if index + 1 < chars.len() {
            Some([chars[index], chars[index + 1]].iter().collect::<String>())
        } else {
            None
        };
        if let Some(two) = two.as_deref().filter(|value| {
            matches!(
                *value,
                "&&" | "||" | "<<" | ">>" | "==" | "!=" | "<=" | ">="
            )
        }) {
            tokens.push(RustyMilkToken::Op(two.to_string()));
            index += 2;
            continue;
        }
        if matches!(
            ch,
            '(' | ')'
                | '+'
                | '-'
                | '*'
                | '/'
                | '%'
                | ','
                | '?'
                | ':'
                | '<'
                | '>'
                | '&'
                | '|'
                | '^'
                | '!'
                | '~'
        ) {
            tokens.push(RustyMilkToken::Op(ch.to_string()));
            index += 1;
            continue;
        }
        return Err(format!(
            "Unsupported RustyMilk expression syntax: {expression}"
        ));
    }
    Ok(tokens)
}

fn rustymilk_number(scope: &BTreeMap<String, RustyMilkValue>, name: &str) -> f64 {
    scope
        .get(name)
        .and_then(RustyMilkValue::as_number)
        .unwrap_or(0.0)
}

fn rustymilk_buffer_key(name: &str, index: f64) -> String {
    let prefix = if name.eq_ignore_ascii_case("gmegabuf") {
        "gmegabuf"
    } else {
        "megabuf"
    };
    let index = if index.is_finite() {
        index.trunc().max(0.0) as usize
    } else {
        0
    };
    format!("{prefix}_{index}")
}

fn rustymilk_buffer_number(
    scope: &BTreeMap<String, RustyMilkValue>,
    name: &str,
    index: f64,
) -> f64 {
    rustymilk_number(scope, &rustymilk_buffer_key(name, index))
}

fn rustymilk_indexed_sample(values: &[f64], position: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let normalized = position.clamp(0.0, 1.0);
    let index = ((normalized * values.len() as f64).floor() as usize).min(values.len() - 1);
    let value = values[index];
    if value > 1.0 {
        value / 255.0
    } else {
        value
    }
}

fn mix_rustymilk_rand_seed(mut seed: u64, value: f64) -> u64 {
    seed ^= value.to_bits().wrapping_add(0x9e37_79b9_7f4a_7c15);
    seed = seed.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    seed ^ (seed >> 31)
}

fn rustymilk_pseudo_random_unit(scope: &BTreeMap<String, RustyMilkValue>, counter: usize) -> f64 {
    let mut seed = 0x4d49_4c4b_4452_4f50u64 ^ counter as u64;
    for key in [
        "time", "frame", "bass", "mid", "treb", "bass_att", "mid_att", "treb_att",
    ] {
        seed = mix_rustymilk_rand_seed(seed, rustymilk_number(scope, key));
    }
    seed = mix_rustymilk_rand_seed(seed, counter as f64 + 0.123_456_789);
    seed ^= seed >> 12;
    seed ^= seed << 25;
    seed ^= seed >> 27;
    let value = seed.wrapping_mul(0x2545_f491_4f6c_dd1d);
    (value as f64) / (u64::MAX as f64)
}

struct RustyMilkExpressionParser<'a> {
    rand_counter: usize,
    scope: &'a BTreeMap<String, RustyMilkValue>,
    tokens: Vec<RustyMilkToken>,
    index: usize,
}

impl<'a> RustyMilkExpressionParser<'a> {
    fn new(
        tokens: Vec<RustyMilkToken>,
        scope: &'a BTreeMap<String, RustyMilkValue>,
        rand_counter: usize,
    ) -> Self {
        Self {
            rand_counter,
            scope,
            tokens,
            index: 0,
        }
    }

    fn peek_op(&self) -> Option<&str> {
        match self.tokens.get(self.index) {
            Some(RustyMilkToken::Op(value)) => Some(value),
            _ => None,
        }
    }

    fn match_op(&mut self, expected: &str) -> bool {
        if self.peek_op() == Some(expected) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn consume(&mut self) -> Option<RustyMilkToken> {
        let token = self.tokens.get(self.index).cloned();
        if token.is_some() {
            self.index += 1;
        }
        token
    }

    fn parse(&mut self) -> Result<f64, String> {
        let value = self.parse_conditional()?;
        if self.index < self.tokens.len() {
            return Err("Unexpected trailing RustyMilk token".to_string());
        }
        Ok(value)
    }

    fn parse_primary(&mut self) -> Result<f64, String> {
        match self.consume() {
            Some(RustyMilkToken::Number(value)) => Ok(value),
            Some(RustyMilkToken::Op(op)) if op == "(" => {
                let value = self.parse_conditional()?;
                if !self.match_op(")") {
                    return Err("Unclosed RustyMilk expression group.".to_string());
                }
                Ok(value)
            }
            Some(RustyMilkToken::Ident(name)) => {
                if self.match_op("(") {
                    let mut args = Vec::new();
                    if self.peek_op() != Some(")") {
                        loop {
                            args.push(self.parse_conditional()?);
                            if !self.match_op(",") {
                                break;
                            }
                        }
                    }
                    if !self.match_op(")") {
                        return Err(format!("Unclosed function call: {name}"));
                    }
                    self.call_function(&name, &args)
                } else {
                    Ok(match name.as_str() {
                        "e" => std::f64::consts::E,
                        "pi" => std::f64::consts::PI,
                        _ => rustymilk_number(self.scope, &name),
                    })
                }
            }
            Some(token) => Err(format!("Unexpected RustyMilk token: {token:?}")),
            None => Err("Unexpected end of RustyMilk expression.".to_string()),
        }
    }

    fn call_function(&mut self, name: &str, args: &[f64]) -> Result<f64, String> {
        let arg = |index: usize, default: f64| args.get(index).copied().unwrap_or(default);
        let out = match name {
            "abs" => arg(0, 0.0).abs(),
            "above" => (arg(0, 0.0) > arg(1, 0.0)) as i32 as f64,
            "acos" => arg(0, 0.0).clamp(-1.0, 1.0).acos(),
            "asin" => arg(0, 0.0).clamp(-1.0, 1.0).asin(),
            "atan" => arg(0, 0.0).atan(),
            "atan2" => arg(0, 0.0).atan2(arg(1, 0.0)),
            "below" => (arg(0, 0.0) < arg(1, 0.0)) as i32 as f64,
            "band" => ((arg(0, 0.0).trunc() as i64) & (arg(1, 0.0).trunc() as i64)) as f64,
            "bor" => ((arg(0, 0.0).trunc() as i64) | (arg(1, 0.0).trunc() as i64)) as f64,
            "bnot" => (!(arg(0, 0.0).trunc() as i64)) as f64,
            "bxor" => ((arg(0, 0.0).trunc() as i64) ^ (arg(1, 0.0).trunc() as i64)) as f64,
            "ceil" => arg(0, 0.0).ceil(),
            "cos" => arg(0, 0.0).cos(),
            "div" => {
                let right = arg(1, 0.0);
                if right == 0.0 {
                    0.0
                } else {
                    arg(0, 0.0) / right
                }
            }
            "equal" => ((arg(0, 0.0) - arg(1, 0.0)).abs() < 0.00001) as i32 as f64,
            "exp" => arg(0, 0.0).exp(),
            "floor" => arg(0, 0.0).floor(),
            "gmegabuf" => rustymilk_buffer_number(self.scope, name, arg(0, 0.0)),
            "get_fft" => {
                let values = rustymilk_frequency_data(self.scope);
                rustymilk_indexed_sample(&values, arg(0, 0.0))
            }
            "get_fft_hz" => {
                let sample_rate = rustymilk_number(self.scope, "sample_rate")
                    .max(rustymilk_number(self.scope, "samplerate"))
                    .max(44100.0);
                let nyquist = sample_rate / 2.0;
                let values = rustymilk_frequency_data(self.scope);
                rustymilk_indexed_sample(
                    &values,
                    if nyquist > 0.0 {
                        arg(0, 0.0) / nyquist
                    } else {
                        0.0
                    },
                )
            }
            "get_waveform" => {
                let values = rustymilk_waveform_data(self.scope);
                rustymilk_indexed_sample(&values, arg(0, 0.0))
            }
            "if" => {
                if arg(0, 0.0) != 0.0 {
                    arg(1, 0.0)
                } else {
                    arg(2, 0.0)
                }
            }
            "int" => arg(0, 0.0).trunc(),
            "log" => {
                if arg(0, 0.0) <= 0.0 {
                    0.0
                } else {
                    arg(0, 0.0).ln()
                }
            }
            "log10" => {
                if arg(0, 0.0) <= 0.0 {
                    0.0
                } else {
                    arg(0, 0.0).log10()
                }
            }
            "max" => arg(0, 0.0).max(arg(1, 0.0)),
            "megabuf" => rustymilk_buffer_number(self.scope, name, arg(0, 0.0)),
            "min" => arg(0, 0.0).min(arg(1, 0.0)),
            "mod" => {
                let right = arg(1, 0.0);
                if right == 0.0 {
                    0.0
                } else {
                    arg(0, 0.0) % right
                }
            }
            "pow" => arg(0, 0.0).powf(arg(1, 0.0)),
            "rand" => {
                let upper = arg(0, 1.0).trunc().max(0.0);
                if upper <= 0.0 {
                    0.0
                } else {
                    let counter = self.rand_counter;
                    self.rand_counter += 1;
                    (rustymilk_pseudo_random_unit(self.scope, counter) * upper)
                        .floor()
                        .min(upper - 1.0)
                }
            }
            "sign" => arg(0, 0.0).signum(),
            "sigmoid" => {
                let constraint = if arg(1, 1.0) == 0.0 { 1.0 } else { arg(1, 1.0) };
                1.0 / (1.0 + (-arg(0, 0.0) * constraint).exp())
            }
            "sin" => arg(0, 0.0).sin(),
            "sqr" => arg(0, 0.0) * arg(0, 0.0),
            "sqrt" => {
                if arg(0, 0.0) < 0.0 {
                    0.0
                } else {
                    arg(0, 0.0).sqrt()
                }
            }
            "tan" => arg(0, 0.0).tan(),
            _ => return Err(format!("Unsupported RustyMilk function: {name}")),
        };
        Ok(if out.is_finite() { out } else { 0.0 })
    }

    fn parse_unary(&mut self) -> Result<f64, String> {
        if self.match_op("+") {
            return self.parse_unary();
        }
        if self.match_op("-") {
            return Ok(-self.parse_unary()?);
        }
        if self.match_op("!") {
            return Ok((self.parse_unary()? == 0.0) as i32 as f64);
        }
        if self.match_op("~") {
            return Ok((!(self.parse_unary()?.trunc() as i64)) as f64);
        }
        self.parse_primary()
    }

    fn parse_factor(&mut self) -> Result<f64, String> {
        let mut value = self.parse_unary()?;
        while let Some(op) = self
            .peek_op()
            .filter(|op| matches!(*op, "*" | "/" | "%"))
            .map(str::to_string)
        {
            self.index += 1;
            let right = self.parse_unary()?;
            value = match op.as_str() {
                "*" => value * right,
                "/" => {
                    if right == 0.0 {
                        0.0
                    } else {
                        value / right
                    }
                }
                "%" => {
                    if right == 0.0 {
                        0.0
                    } else {
                        value % right
                    }
                }
                _ => value,
            };
        }
        Ok(value)
    }

    fn parse_term(&mut self) -> Result<f64, String> {
        let mut value = self.parse_factor()?;
        while let Some(op) = self
            .peek_op()
            .filter(|op| matches!(*op, "+" | "-"))
            .map(str::to_string)
        {
            self.index += 1;
            let right = self.parse_factor()?;
            value = if op == "+" {
                value + right
            } else {
                value - right
            };
        }
        Ok(value)
    }

    fn parse_shift(&mut self) -> Result<f64, String> {
        let mut value = self.parse_term()?;
        while let Some(op) = self
            .peek_op()
            .filter(|op| matches!(*op, "<<" | ">>"))
            .map(str::to_string)
        {
            self.index += 1;
            let right = self.parse_term()?;
            value = if op == "<<" {
                ((value.trunc() as i64) << (right.trunc() as u32)) as f64
            } else {
                ((value.trunc() as i64) >> (right.trunc() as u32)) as f64
            };
        }
        Ok(value)
    }

    fn parse_comparison(&mut self) -> Result<f64, String> {
        let mut value = self.parse_shift()?;
        while let Some(op) = self
            .peek_op()
            .filter(|op| matches!(*op, "<" | ">" | "<=" | ">=" | "==" | "!="))
            .map(str::to_string)
        {
            self.index += 1;
            let right = self.parse_shift()?;
            value = match op.as_str() {
                "<" => (value < right) as i32 as f64,
                ">" => (value > right) as i32 as f64,
                "<=" => (value <= right) as i32 as f64,
                ">=" => (value >= right) as i32 as f64,
                "==" => ((value - right).abs() < 0.00001) as i32 as f64,
                "!=" => ((value - right).abs() >= 0.00001) as i32 as f64,
                _ => value,
            };
        }
        Ok(value)
    }

    fn parse_bitwise_and(&mut self) -> Result<f64, String> {
        let mut value = self.parse_comparison()?;
        while self.match_op("&") {
            value = ((value.trunc() as i64) & (self.parse_comparison()?.trunc() as i64)) as f64;
        }
        Ok(value)
    }

    fn parse_bitwise_xor(&mut self) -> Result<f64, String> {
        let mut value = self.parse_bitwise_and()?;
        while self.match_op("^") {
            value = ((value.trunc() as i64) ^ (self.parse_bitwise_and()?.trunc() as i64)) as f64;
        }
        Ok(value)
    }

    fn parse_bitwise_or(&mut self) -> Result<f64, String> {
        let mut value = self.parse_bitwise_xor()?;
        while self.match_op("|") {
            value = ((value.trunc() as i64) | (self.parse_bitwise_xor()?.trunc() as i64)) as f64;
        }
        Ok(value)
    }

    fn parse_logical_and(&mut self) -> Result<f64, String> {
        let mut value = self.parse_bitwise_or()?;
        while self.match_op("&&") {
            value = (value != 0.0 && self.parse_bitwise_or()? != 0.0) as i32 as f64;
        }
        Ok(value)
    }

    fn parse_logical_or(&mut self) -> Result<f64, String> {
        let mut value = self.parse_logical_and()?;
        while self.match_op("||") {
            value = (value != 0.0 || self.parse_logical_and()? != 0.0) as i32 as f64;
        }
        Ok(value)
    }

    fn parse_conditional(&mut self) -> Result<f64, String> {
        let condition = self.parse_logical_or()?;
        if !self.match_op("?") {
            return Ok(condition);
        }
        let when_true = self.parse_conditional()?;
        if !self.match_op(":") {
            return Err("Unclosed RustyMilk conditional expression.".to_string());
        }
        let when_false = self.parse_conditional()?;
        Ok(if condition != 0.0 {
            when_true
        } else {
            when_false
        })
    }
}

fn rustymilk_frequency_data(scope: &BTreeMap<String, RustyMilkValue>) -> Vec<f64> {
    [
        "frequency_data",
        "frequencies",
        "frequency",
        "spectrum",
        "fft",
    ]
    .into_iter()
    .find_map(|name| match scope.get(name) {
        Some(RustyMilkValue::Text(value)) => Some(
            value
                .split(',')
                .filter_map(|item| item.trim().parse::<f64>().ok())
                .collect::<Vec<_>>(),
        ),
        Some(RustyMilkValue::Number(value)) => Some(vec![*value]),
        None => None,
    })
    .unwrap_or_default()
}

fn rustymilk_waveform_data(scope: &BTreeMap<String, RustyMilkValue>) -> Vec<f64> {
    ["waveform_data", "waveform", "samples", "wave"]
        .into_iter()
        .find_map(|name| match scope.get(name) {
            Some(RustyMilkValue::Text(value)) => Some(
                value
                    .split(',')
                    .filter_map(|item| item.trim().parse::<f64>().ok())
                    .collect::<Vec<_>>(),
            ),
            Some(RustyMilkValue::Number(value)) => Some(vec![*value]),
            None => None,
        })
        .unwrap_or_default()
}

pub fn evaluate_rustymilk_expression(
    expression: &str,
    variables: &BTreeMap<String, RustyMilkValue>,
) -> Result<f64, String> {
    evaluate_rustymilk_expression_with_rand_counter(expression, variables, 0)
        .map(|(value, _)| value)
}

fn evaluate_rustymilk_expression_with_rand_counter(
    expression: &str,
    variables: &BTreeMap<String, RustyMilkValue>,
    rand_counter: usize,
) -> Result<(f64, usize), String> {
    let scope = variables
        .iter()
        .map(|(key, value)| (key.to_ascii_lowercase(), value.clone()))
        .collect::<BTreeMap<_, _>>();
    let tokens = tokenize_rustymilk_expression(expression)?;
    let mut parser = RustyMilkExpressionParser::new(tokens, &scope, rand_counter);
    let value = parser.parse()?;
    Ok((value, parser.rand_counter))
}

pub fn evaluate_rustymilk_equations(
    equations: &str,
    variables: &BTreeMap<String, RustyMilkValue>,
) -> Result<BTreeMap<String, RustyMilkValue>, String> {
    let mut scope = variables
        .iter()
        .map(|(key, value)| (key.to_ascii_lowercase(), value.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut rand_counter = rustymilk_number(&scope, "__rand_counter").max(0.0) as usize;
    for statement in equations
        .split(';')
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if let Some((buffer_name, index_expression, operator, expression)) =
            split_rustymilk_buffer_assignment(statement)
        {
            let (buffer_index, next_rand_counter) =
                evaluate_rustymilk_expression_with_rand_counter(
                    index_expression,
                    &scope,
                    rand_counter,
                )?;
            rand_counter = next_rand_counter;
            let key = rustymilk_buffer_key(&buffer_name, buffer_index);
            let current = rustymilk_number(&scope, &key);
            let (next, next_rand_counter) =
                evaluate_rustymilk_expression_with_rand_counter(expression, &scope, rand_counter)?;
            rand_counter = next_rand_counter;
            let value = apply_rustymilk_assignment_operator(current, operator, next);
            scope.insert(key, RustyMilkValue::Number(value));
            continue;
        }
        let Some((name, operator, expression)) = split_rustymilk_assignment(statement) else {
            let (_, next_rand_counter) =
                evaluate_rustymilk_expression_with_rand_counter(statement, &scope, rand_counter)?;
            rand_counter = next_rand_counter;
            continue;
        };
        let current = rustymilk_number(&scope, &name);
        let (next, next_rand_counter) =
            evaluate_rustymilk_expression_with_rand_counter(expression, &scope, rand_counter)?;
        rand_counter = next_rand_counter;
        let value = apply_rustymilk_assignment_operator(current, operator, next);
        scope.insert(name, RustyMilkValue::Number(value));
    }
    scope.insert(
        "__rand_counter".to_string(),
        RustyMilkValue::Number(rand_counter as f64),
    );
    Ok(scope)
}

fn apply_rustymilk_assignment_operator(current: f64, operator: &str, next: f64) -> f64 {
    match operator {
        "=" => next,
        "+=" => current + next,
        "-=" => current - next,
        "*=" => current * next,
        "/=" => {
            if next == 0.0 {
                0.0
            } else {
                current / next
            }
        }
        _ => next,
    }
}

fn split_rustymilk_buffer_assignment(
    statement: &str,
) -> Option<(String, &str, &'static str, &str)> {
    for operator in ["+=", "-=", "*=", "/=", "="] {
        let Some((raw_name, expression)) = statement.split_once(operator) else {
            continue;
        };
        let raw_name = raw_name.trim();
        let open = raw_name.find('(')?;
        let close = raw_name.rfind(')')?;
        if close <= open || close != raw_name.len() - 1 {
            continue;
        }
        let name = raw_name[..open].trim().to_ascii_lowercase();
        if name != "megabuf" && name != "gmegabuf" {
            continue;
        }
        let index_expression = raw_name[open + 1..close].trim();
        if index_expression.is_empty() {
            continue;
        }
        return Some((name, index_expression, operator, expression.trim()));
    }
    None
}

fn split_rustymilk_assignment(statement: &str) -> Option<(String, &'static str, &str)> {
    for operator in ["+=", "-=", "*=", "/=", "="] {
        if let Some((raw_name, expression)) = statement.split_once(operator) {
            let name = raw_name.trim();
            if !name.is_empty()
                && name
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '.')
                && name
                    .chars()
                    .next()
                    .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
            {
                return Some((name.to_ascii_lowercase(), operator, expression.trim()));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ImportedMilkdropFixture {
        id: &'static str,
        source: String,
        supported: bool,
        expected_error: &'static str,
    }

    fn dense_primitive_fixture_source() -> String {
        let mut lines = vec![
            "name=Fixture Dense Primitive Pack".to_string(),
            "decay=0.84".to_string(),
            "wave_r=0.4".to_string(),
            "wave_g=0.8".to_string(),
            "wave_b=0.95".to_string(),
        ];
        for index in 0..40 {
            lines.push(format!("shape{index:02}_enabled=1"));
            lines.push(format!("shape{index:02}_sides={}", 3 + (index % 6)));
            lines.push(format!(
                "shape{index:02}_rad={:.3}",
                0.035 + f64::from(index % 5) * 0.006
            ));
            lines.push(format!(
                "shape{index:02}_x={:.3}",
                0.08 + f64::from(index % 10) * 0.09
            ));
            lines.push(format!(
                "shape{index:02}_y={:.3}",
                0.12 + f64::from(index / 10) * 0.2
            ));
            lines.push(format!("shape{index:02}_a=0.18"));
            lines.push(format!(
                "shape{index:02}_per_frame1=ang=time*{:.3};",
                0.05 + f64::from(index) * 0.001
            ));
        }
        for index in 0..20 {
            lines.push(format!("wavecode_{index}_enabled=1"));
            lines.push(format!("wavecode_{index}_samples=32"));
            lines.push(format!(
                "wavecode_{index}_r={:.3}",
                0.25 + f64::from(index % 4) * 0.14
            ));
            lines.push(format!(
                "wavecode_{index}_g={:.3}",
                0.45 + f64::from(index % 5) * 0.08
            ));
            lines.push(format!(
                "wavecode_{index}_b={:.3}",
                0.75 - f64::from(index % 3) * 0.12
            ));
            lines.push(format!("wavecode_{index}_a=0.34"));
            lines.push(format!("wavecode_{index}_per_point1=x=i;"));
            lines.push(format!(
                "wavecode_{index}_per_point2=y={:.3}+sample*0.08;",
                0.05 + f64::from(index) * 0.045
            ));
        }
        lines.join("\n")
    }

    fn imported_milkdrop_fixtures() -> Vec<ImportedMilkdropFixture> {
        vec![
            ImportedMilkdropFixture {
                id: "classic-primitives",
                supported: true,
                expected_error: "",
                source: r#"
name=Fixture Classic Primitives
decay=0.82
wave_r=0.25
wave_g=0.7
wave_b=0.95
wave_scale=1.4
meshx=3
meshy=2
per_frame_1=rot=0.015*sin(time);
per_pixel_1=dx=0.02*sin((x+time)*6.283);
per_pixel_2=dy=0.02*cos((y+time)*6.283);
mv_x=4
mv_y=3
mv_l=0.2
mv_a=0.45
shape00_enabled=1
shape00_textured=1
shape00_sides=5
shape00_rad=0.22
shape00_x=0.5
shape00_y=0.5
shape00_r=0.9
shape00_g=0.85
shape00_b=0.15
shape00_a=0.45
sprite00_enabled=1
sprite00_image=fixture-logo.png
sprite00_x=0.22
sprite00_y=0.78
sprite00_w=0.08
sprite00_h=0.08
sprite00_a=0.35
wavecode_0_enabled=1
wavecode_0_samples=32
wavecode_0_spectrum=1
wavecode_0_dots=1
wavecode_0_r=0.8
wavecode_0_g=1
wavecode_0_b=0.3
wavecode_0_a=0.9
wavecode_0_per_point1=x=i;
wavecode_0_per_point2=y=0.1+sample*0.65;
"#
                .to_string(),
            },
            ImportedMilkdropFixture {
                id: "shader-subset",
                supported: true,
                expected_error: "",
                source: r#"
name=Fixture Shader Subset
decay=0.78
wave_r=0.6
wave_g=0.25
wave_b=0.9
warp_shader=shader_body {
warp_shader_1=  float3 tint = vec3(0.8, 0.95, aspect);
warp_shader_2=  float3 noise = tex2D(sampler_noise, uv).rgb;
warp_shader_3=  ret = tex2D(sampler_main, uv).rgb * tint * noise;
warp_shader_4=}
comp_shader=shader_body {
comp_shader_1=  ret = saturate(vec3(x, y, 0.45 + 0.35 * sin(time)));
comp_shader_2=}
shape00_enabled=1
shape00_sides=6
shape00_rad=0.14
shape00_a=0.22
"#
                .to_string(),
            },
            ImportedMilkdropFixture {
                id: "milk2-double",
                supported: true,
                expected_error: "",
                source: r#"
[preset00]
name=Fixture Double Left
zoom=1
per_frame_1=q33=sin(time);
[preset01]
name=Fixture Double Right
blend_alpha=0.65
zoom=0.9
per_frame_1=q34=cos(time);
"#
                .to_string(),
            },
            ImportedMilkdropFixture {
                id: "milkdrop3-q-registers",
                supported: true,
                expected_error: "",
                source: r#"
[preset00]
name=Fixture Q Register Coverage A
q64=0.64
per_frame_1=q1=bass_att+q64;
per_frame_2=q32=sin(time)+q1;
warp_shader=ret = tex2D(sampler_main, uv).rgb * vec3(q1, q32, q64);
wavecode_0_enabled=1
wavecode_0_samples=16
wavecode_0_per_frame1=q48=q32+0.1;
wavecode_0_per_point1=y=sample+q48;
[preset01]
name=Fixture Q Register Coverage B
blend_alpha=0.35
composite_mode=screen
per_frame_1=q63=treb_att+q2;
comp_shader=ret = vec3(q2, q63, q64);
shape00_enabled=1
shape00_sides=4
shape00_per_frame1=q64=max(q64,0.75);
sprite00_enabled=1
sprite00_image=fixture-logo.png
sprite00_per_frame1=q16=q63*0.5;
"#
                .to_string(),
            },
            ImportedMilkdropFixture {
                id: "milkdrop3-dense-primitives",
                supported: true,
                expected_error: "",
                source: dense_primitive_fixture_source(),
            },
            ImportedMilkdropFixture {
                id: "unsupported-control-flow-shader",
                supported: false,
                expected_error: "shader translation pending: comp_shader",
                source: r#"
name=Fixture Unsupported Shader
wave_r=1
comp_shader=for (;;) { ret = vec3(1.0); }
"#
                .to_string(),
            },
        ]
    }

    #[test]
    fn rustymilk_core_parses_milk2_frame_sets() {
        let frame_set = rustymilk_frame_set_from_source(
            r#"
[preset00]
name=One
wave_r=1
[preset01]
name=Two
wave_g=1
"#,
            1.25,
            0.4,
            0.3,
            0.2,
        );

        assert_eq!(frame_set.preset_count, 2);
        assert_eq!(frame_set.entries.len(), 2);
        assert_eq!(frame_set.entries[0].title, "One");
        assert_eq!(frame_set.entries[1].title, "Two");
    }

    #[test]
    fn rustymilk_core_exports_webgpu_batch_summary_json() {
        let summary = rustymilk_webgpu_batch_summary_json(
            "name=Batch Probe\nshape00_enabled=1\nshape00_sides=4\nshape00_a=1",
            0.5,
            0.7,
            0.2,
            0.1,
            &parse_rustymilk_sample_csv("-1,0,1"),
            &parse_rustymilk_sample_csv("0,0.5,1"),
        );
        let value: serde_json::Value = serde_json::from_str(&summary).unwrap();

        assert_eq!(value["frameSet"]["presetCount"], 1);
        assert!(value["packed"]["filledVertices"].as_u64().unwrap() > 0);
    }

    #[test]
    fn rustymilk_core_validates_unsupported_functions_before_rendering() {
        let error = validate_rustymilk_import("name=Bad\nper_frame_1=q1=unsupported_call(1);")
            .expect_err("unsupported functions should be rejected");

        assert!(error.contains("unsupported_call"));
    }

    #[test]
    fn rustymilk_core_expression_vm_keeps_buffer_state() {
        let scope = evaluate_rustymilk_equations(
            "megabuf(2)=0.75; gmegabuf(4)=1.5; q2=megabuf(2)+gmegabuf(4);",
            &BTreeMap::new(),
        )
        .unwrap();

        assert_eq!(scope.get("megabuf_2"), Some(&RustyMilkValue::Number(0.75)));
        assert_eq!(scope.get("gmegabuf_4"), Some(&RustyMilkValue::Number(1.5)));
        assert_eq!(scope.get("q2"), Some(&RustyMilkValue::Number(2.25)));
    }

    #[test]
    fn rustymilk_core_matches_imported_expression_vm_helpers() {
        fn assert_close(actual: Option<&RustyMilkValue>, expected: f64) {
            let Some(RustyMilkValue::Number(actual)) = actual else {
                panic!("expected numeric RustyMilk value");
            };
            assert!(
                (*actual - expected).abs() < 0.00001,
                "{actual} != {expected}"
            );
        }

        let mut scope = BTreeMap::new();
        scope.insert("bass_att".to_string(), RustyMilkValue::Number(2.0));
        assert_eq!(
            evaluate_rustymilk_expression("pow(bass_att, 2) + sqr(3)", &scope).unwrap(),
            13.0
        );

        let mut scope = BTreeMap::new();
        scope.insert("treb".to_string(), RustyMilkValue::Number(2.0));
        assert_eq!(
            evaluate_rustymilk_expression("if(above(treb, 1.5), sin(0), 7)", &scope).unwrap(),
            0.0
        );
        assert_eq!(
            evaluate_rustymilk_expression("div(10, 0) + sqrt(-1)", &BTreeMap::new()).unwrap(),
            0.0
        );

        let mut scope = BTreeMap::new();
        scope.insert("q33".to_string(), RustyMilkValue::Number(2.0));
        assert_eq!(
            evaluate_rustymilk_expression("q33 >= 2", &scope).unwrap(),
            1.0
        );

        let mut scope = BTreeMap::new();
        scope.insert("bass_att".to_string(), RustyMilkValue::Number(3.0));
        scope.insert("treb_att".to_string(), RustyMilkValue::Number(1.0));
        scope.insert("wave_r".to_string(), RustyMilkValue::Number(0.8));
        scope.insert("zoom".to_string(), RustyMilkValue::Number(1.0));
        let evaluated = evaluate_rustymilk_equations(
            "q1=bass_att*0.2; zoom+=q1; q33=if(below(treb_att,2),7,9); wave_r*=0.5;",
            &scope,
        )
        .unwrap();
        assert_close(evaluated.get("q1"), 0.6);
        assert_close(evaluated.get("zoom"), 1.6);
        assert_eq!(evaluated.get("q33"), Some(&RustyMilkValue::Number(7.0)));
        assert_close(evaluated.get("wave_r"), 0.4);
    }

    #[test]
    fn rustymilk_core_matches_imported_expression_vm_audio_and_bitwise_helpers() {
        let mut scope = BTreeMap::new();
        scope.insert(
            "frequency_data".to_string(),
            RustyMilkValue::Text("0,0.501960784314,1,0.250980392157".to_string()),
        );
        scope.insert("sample_rate".to_string(), RustyMilkValue::Number(44_100.0));
        assert!(
            (evaluate_rustymilk_expression("get_fft(0.5)", &scope).unwrap() - 1.0).abs() < 0.00001
        );
        assert!(
            (evaluate_rustymilk_expression("get_fft_hz(5512.5)", &scope).unwrap() - 0.501960784314)
                .abs()
                < 0.00001
        );

        assert!(
            (evaluate_rustymilk_expression("sin(pi/2)+log(e)+log10(100)", &BTreeMap::new())
                .unwrap()
                - 4.0)
                .abs()
                < 0.00001
        );
        assert!(
            (evaluate_rustymilk_expression("atan2(1, 0)", &BTreeMap::new()).unwrap()
                - std::f64::consts::FRAC_PI_2)
                .abs()
                < 0.00001
        );
        assert!(
            (evaluate_rustymilk_expression("asin(2)+acos(-2)", &BTreeMap::new()).unwrap()
                - std::f64::consts::PI * 1.5)
                .abs()
                < 0.00001
        );
        assert_eq!(
            evaluate_rustymilk_expression("band(7, 3)+bor(4, 1)+bxor(7, 3)", &BTreeMap::new())
                .unwrap(),
            12.0
        );
        assert_eq!(
            evaluate_rustymilk_expression("(7 & 3) + (4 | 1) + (7 ^ 3)", &BTreeMap::new()).unwrap(),
            12.0
        );
        assert_eq!(
            evaluate_rustymilk_expression("(1 << 3) + (8 >> 1)", &BTreeMap::new()).unwrap(),
            12.0
        );
        assert_eq!(
            evaluate_rustymilk_expression("~0 + !0 + !2", &BTreeMap::new()).unwrap(),
            0.0
        );
    }

    #[test]
    fn rustymilk_core_matches_imported_shader_translation_subset() {
        assert_eq!(
            translate_rustymilk_shader_expression(
                "ret = tex2D(sampler_main, uv).rgb * vec3(0.5, 1.0, 0.25);"
            ),
            "texture(previousFrame, uv).rgb * vec3(0.5, 1.0, 0.25)"
        );

        let shader = create_translated_rustymilk_fragment_shader(
            "ret = saturate(vec3(uv.x, uv.y, sin(time)));",
        );
        assert!(shader.contains("uniform sampler2D previousFrame;"));
        assert!(shader.contains("uniform float fftBins[64];"));
        assert!(shader.contains("uniform float waveformBins[64];"));
        assert!(shader.contains("uniform vec2 resolution;"));
        assert!(shader.contains("uniform vec2 pixelSize;"));
        assert!(shader.contains("uniform float aspect;"));
        assert!(shader.contains("uniform vec4 texsize;"));
        assert!(shader.contains("float rad = length(centeredUv);"));
        assert!(shader.contains("float ang = atan(centeredUv.y, centeredUv.x);"));
        assert!(shader.contains("float get_fft(float position)"));
        assert!(shader.contains("float get_fft_hz(float hz)"));
        assert!(shader.contains("float get_waveform(float position)"));
        assert!(shader.contains("uniform float bass_att;"));
        assert!(shader.contains("uniform float q64;"));
        assert!(shader.contains("vec3 ret = vec3(clamp01(vec3(uv.x, uv.y, sin(time))));"));
        assert!(analyze_rustymilk_shader_support("ret = vec3(q64, mid_att, bass);").supported);

        let shader = create_translated_rustymilk_fragment_shader(
            "ret = vec3(get_fft(0.25), get_fft_hz(1000), get_waveform(0.5));",
        );
        assert!(shader.contains(
            "vec3 ret = vec3(vec3(get_fft(0.25), get_fft_hz(1000), get_waveform(0.5)));"
        ));
    }

    #[test]
    fn rustymilk_core_matches_imported_shader_texture_body_and_webgpu_subset() {
        let shader = create_translated_rustymilk_fragment_shader(
            r#"
shader_body {
  float3 tint = saturate(vec3(x, y, aspect));
  ret = tint * tex2D(sampler_main, uv).rgb;
}
"#,
        );
        assert!(shader.contains("vec3 tint = clamp01(vec3(x, y, aspect));"));
        assert!(shader.contains("vec3 ret = vec3(tint * texture(previousFrame, uv).rgb);"));
        assert_eq!(
            translate_rustymilk_shader_expression("shader_body { ret = vec3(q1); }"),
            "vec3(q1)"
        );

        let shader = create_translated_rustymilk_fragment_shader(
            r#"
float3 noise = tex2D(sampler_noise, uv).rgb;
float3 overlay = tex2D(album_art, uv).rgb;
ret = noise * 0.5 + overlay * 0.5 + tex2D(sampler_main, uv).rgb * 0.1;
"#,
        );
        assert_eq!(
            get_rustymilk_shader_texture_samplers(
                "ret = tex2D(sampler_noise, uv).rgb + tex2D(album_art, uv).rgb;"
            ),
            vec!["sampler_noise".to_string(), "album_art".to_string()]
        );
        assert!(shader.contains("uniform sampler2D shaderTexture0;"));
        assert!(shader.contains("uniform sampler2D shaderTexture1;"));
        assert!(shader.contains("vec3 noise = texture(shaderTexture0, uv).rgb;"));
        assert!(shader.contains("vec3 overlay = texture(shaderTexture1, uv).rgb;"));

        let shader = create_translated_rustymilk_wgsl_shader(
            r#"
float3 tint = saturate(vec3(q1, bass_att, uv.x));
tint *= tex2D(sampler_main, uv).rgb;
ret = tint + vec3(time * 0.01, get_fft(0.25), get_waveform(0.5));
"#,
        );
        assert!(shader.contains("@fragment"));
        assert!(shader.contains("q64: f32"));
        assert!(shader.contains("fft63: f32"));
        assert!(shader.contains("waveform63: f32"));
        assert!(shader.contains("let q1 = uniforms.q1;"));
        assert!(shader.contains("fn get_fft(position: f32) -> f32"));
        assert!(shader.contains("fn get_fft_hz(hz: f32) -> f32"));
        assert!(shader.contains("fn get_waveform(position: f32) -> f32"));
        assert!(shader.contains("var tint = clamp01v3(vec3f(q1, bass_att, uv.x));"));
        assert!(shader.contains("tint *= textureSample(previousFrame, previousSampler, uv).rgb;"));
        assert!(shader.contains(
            "let ret = vec3f(tint + vec3f(time * 0.01, get_fft(0.25), get_waveform(0.5)));"
        ));

        assert_eq!(
            translate_rustymilk_shader_expression("for (;;) { ret = vec3(1.0); }"),
            ""
        );
        assert_eq!(
            translate_rustymilk_shader_expression("float3 tint; ret = tint;"),
            ""
        );
        assert!(
            analyze_rustymilk_webgpu_shader_support("ret = q1 > 0.5 ? vec3(1.0) : vec3(0.0);")
                .supported
        );
    }

    #[test]
    fn rustymilk_core_preserves_imported_milkdrop_fixture_summaries() {
        let summaries = imported_milkdrop_fixtures()
            .into_iter()
            .map(|fixture| {
                let parsed =
                    parse_rustymilk_preset_set(&fixture.source, fixture.id == "milk2-double");
                let preset_summaries = parsed
                    .presets
                    .iter()
                    .map(|preset| {
                        (
                            preset.format.clone(),
                            preset.title.clone(),
                            preset.base_values.keys().cloned().collect::<Vec<_>>(),
                            !preset.equations.frame.trim().is_empty(),
                            !preset.equations.per_pixel.trim().is_empty(),
                            !preset.warp_shader.trim().is_empty(),
                            !preset.comp_shader.trim().is_empty(),
                            preset
                                .shapes
                                .iter()
                                .filter(|entry| rustymilk_entry_has_content(entry))
                                .count(),
                            preset
                                .sprites
                                .iter()
                                .filter(|entry| rustymilk_entry_has_content(entry))
                                .count(),
                            preset
                                .waves
                                .iter()
                                .filter(|entry| rustymilk_entry_has_content(entry))
                                .count(),
                        )
                    })
                    .collect::<Vec<_>>();
                (fixture.id, parsed.format, preset_summaries)
            })
            .collect::<Vec<_>>();

        assert_eq!(summaries[0].0, "classic-primitives");
        assert_eq!(summaries[0].1, "milk");
        assert_eq!(summaries[0].2[0].1, "Fixture Classic Primitives");
        assert_eq!(summaries[0].2[0].7, 1);
        assert_eq!(summaries[0].2[0].8, 1);
        assert_eq!(summaries[0].2[0].9, 1);

        assert_eq!(summaries[2].0, "milk2-double");
        assert_eq!(summaries[2].1, "milk2");
        assert_eq!(summaries[2].2.len(), 2);
        assert_eq!(summaries[2].2[0].1, "Fixture Double Left");
        assert_eq!(summaries[2].2[1].1, "Fixture Double Right");

        assert_eq!(summaries[3].0, "milkdrop3-q-registers");
        assert_eq!(summaries[3].1, "milk2");
        assert_eq!(summaries[3].2.len(), 2);
        assert_eq!(summaries[3].2[0].5, true);
        assert_eq!(summaries[3].2[1].6, true);
        assert_eq!(summaries[3].2[1].8, 1);

        assert_eq!(summaries[4].0, "milkdrop3-dense-primitives");
        assert_eq!(summaries[4].2[0].7, 40);
        assert_eq!(summaries[4].2[0].9, 20);

        assert_eq!(summaries[5].0, "unsupported-control-flow-shader");
        assert_eq!(summaries[5].2[0].6, true);
    }

    #[test]
    fn rustymilk_core_matches_imported_milkdrop_fixture_compatibility() {
        let entries = imported_milkdrop_fixtures()
            .into_iter()
            .map(|fixture| {
                let entry = build_rustymilk_compatibility_entry(
                    fixture.id,
                    "",
                    &fixture.source,
                    fixture.id == "milk2-double",
                );
                assert_eq!(entry.supported, fixture.supported, "{}", fixture.id);
                if fixture.expected_error.is_empty() {
                    assert!(entry
                        .preset_reports
                        .iter()
                        .all(|report| report.error.is_empty()));
                } else {
                    assert!(
                        entry
                            .preset_reports
                            .iter()
                            .any(|report| report.error.contains(fixture.expected_error)),
                        "{} should report {}",
                        fixture.id,
                        fixture.expected_error
                    );
                }
                entry
            })
            .collect::<Vec<_>>();
        let summary = summarize_rustymilk_compatibility_matrix(&entries);

        assert_eq!(summary.total_count, 6);
        assert_eq!(summary.preset_count, 8);
        assert_eq!(summary.supported_count, 5);
        assert_eq!(summary.unsupported_count, 1);
        assert_eq!(summary.max_shape_count, 40);
        assert_eq!(summary.max_sprite_count, 1);
        assert_eq!(summary.max_wave_count, 20);
        assert_eq!(summary.max_q_register_index, 64);
        assert!(summary.q_registers.contains(&"q64".to_string()));
        assert_eq!(
            summary.unsupported_shader_sections,
            vec!["comp_shader".to_string()]
        );
    }
}

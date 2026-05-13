// =============================================================================
// rustymilk-preset: Preset parsing, serialization, compatibility analysis, and
// shader translation for MilkDrop-compatible preset scripts.
// =============================================================================

use std::collections::BTreeMap;
use rustymilk_expr::RustyMilkValue;

#[derive(Clone, Debug, Default, PartialEq)]
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

pub fn clamp_unit(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn clamp_range(value: f64, min: f64, max: f64) -> f64 {
    value.clamp(min, max)
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
            | "assign"
            | "exec2"
            | "exec3"
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
            | "loop"
            | "max"
            | "memcpy"
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
            | "while"
    )
}

fn collect_rustymilk_functions(text: &str, unsupported: &mut Vec<String>) {
    let sanitized = strip_rustymilk_equation_comments(text);
    let chars = sanitized.chars().collect::<Vec<_>>();
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

fn strip_rustymilk_equation_comments(text: &str) -> String {
    text.lines()
        .map(|line| line.split_once("//").map(|(code, _)| code).unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n")
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

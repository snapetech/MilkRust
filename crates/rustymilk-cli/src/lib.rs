use std::{
    fs,
    path::{Path, PathBuf},
};

use rustymilk_core::{
    build_rustymilk_compatibility_entry, parse_rustymilk_preset_set,
    rustymilk_frame_set_from_source_with_audio, rustymilk_preset_name,
    summarize_rustymilk_compatibility_matrix, validate_rustymilk_import,
};
use rustymilk_renderer_core::RustyMilkRenderer;
use rustymilk_renderer_headless::{create_headless_batches, RustyMilkHeadlessRenderer};

#[derive(Clone, Debug, PartialEq)]
pub struct RustyMilkCliResult {
    pub code: i32,
    pub stderr: String,
    pub stdout: String,
}

impl RustyMilkCliResult {
    fn ok(stdout: impl Into<String>) -> Self {
        Self {
            code: 0,
            stderr: String::new(),
            stdout: stdout.into(),
        }
    }

    fn err(code: i32, stderr: impl Into<String>) -> Self {
        Self {
            code,
            stderr: stderr.into(),
            stdout: String::new(),
        }
    }
}

pub fn run_rustymilk_cli(args: &[String]) -> RustyMilkCliResult {
    let Some(command) = args.first().map(String::as_str) else {
        return RustyMilkCliResult::err(2, usage());
    };

    match command {
        "validate" => match read_one_path(args.get(1)) {
            Ok((path, source)) => validate_rustymilk_import(&source)
                .map(|title| {
                    RustyMilkCliResult::ok(format!(
                        "{}\n",
                        serde_json::json!({
                            "path": path.display().to_string(),
                            "status": "valid",
                            "title": title,
                        })
                    ))
                })
                .unwrap_or_else(|error| {
                    RustyMilkCliResult::err(
                        1,
                        format!(
                            "{}\n",
                            serde_json::json!({
                                "path": path.display().to_string(),
                                "status": "invalid",
                                "error": error,
                            })
                        ),
                    )
                }),
            Err(result) => result,
        },
        "inspect" => match read_one_path(args.get(1)) {
            Ok((path, source)) => {
                let parsed = parse_rustymilk_preset_set(
                    &source,
                    source.to_ascii_lowercase().contains("[preset01]"),
                );
                let presets = parsed
                    .presets
                    .iter()
                    .map(|preset| {
                        serde_json::json!({
                            "index": preset.index,
                            "title": preset.title,
                            "baseValueKeys": preset.base_values.keys().cloned().collect::<Vec<_>>(),
                            "shapeCount": preset.shapes.len(),
                            "spriteCount": preset.sprites.len(),
                            "waveCount": preset.waves.len(),
                            "hasWarpShader": !preset.warp_shader.trim().is_empty(),
                            "hasCompShader": !preset.comp_shader.trim().is_empty(),
                        })
                    })
                    .collect::<Vec<_>>();
                RustyMilkCliResult::ok(format!(
                    "{}\n",
                    serde_json::json!({
                        "format": parsed.format,
                        "path": path.display().to_string(),
                        "presetCount": parsed.presets.len(),
                        "presets": presets,
                        "title": rustymilk_preset_name(&source),
                    })
                ))
            }
            Err(result) => result,
        },
        "compat" => compat_command(args.get(1)),
        "render-stats" => match read_one_path(args.get(1)) {
            Ok((path, source)) => {
                let frame_set = rustymilk_frame_set_from_source_with_audio(
                    &source,
                    1.0,
                    0.55,
                    0.35,
                    0.25,
                    &[-1.0, -0.25, 0.0, 0.25, 1.0],
                    &[0.0, 0.2, 0.7, 1.0, 0.4],
                );
                let batches = create_headless_batches(&frame_set);
                let mut renderer = RustyMilkHeadlessRenderer::new();
                let stats = renderer.render_frame_set(&frame_set).unwrap();
                RustyMilkCliResult::ok(format!(
                    "{}\n",
                    serde_json::json!({
                        "path": path.display().to_string(),
                        "frameEntries": stats.frame_entries,
                        "lineVertices": stats.line_vertices,
                        "pointVertices": stats.point_vertices,
                        "texturedVertices": stats.textured_vertices,
                        "triangleVertices": stats.triangle_vertices,
                        "batches": {
                            "filledVertices": batches.filled_vertices.len() / 6,
                            "lineVertices": batches.line_vertices.len() / 6,
                            "pointVertices": batches.point_vertices.len() / 6,
                            "texturedVertices": batches.textured_vertices.len() / 8,
                            "texturedBatches": batches.textured_batches.len(),
                        }
                    })
                ))
            }
            Err(result) => result,
        },
        "--help" | "-h" | "help" => RustyMilkCliResult::ok(usage()),
        _ => RustyMilkCliResult::err(2, usage()),
    }
}

fn compat_command(path: Option<&String>) -> RustyMilkCliResult {
    let Some(path) = path else {
        return RustyMilkCliResult::err(2, usage());
    };
    let path = PathBuf::from(path);
    let Ok(files) = collect_preset_files(&path) else {
        return RustyMilkCliResult::err(1, format!("failed to read {}\n", path.display()));
    };
    if files.is_empty() {
        return RustyMilkCliResult::err(
            1,
            format!("no .milk or .milk2 files in {}\n", path.display()),
        );
    }

    let entries = files
        .iter()
        .filter_map(|file| {
            let source = fs::read_to_string(file).ok()?;
            Some(build_rustymilk_compatibility_entry(
                &file
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("preset")
                    .to_string(),
                &file.display().to_string(),
                &source,
                file.extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("milk2")),
            ))
        })
        .collect::<Vec<_>>();
    let summary = summarize_rustymilk_compatibility_matrix(&entries);
    RustyMilkCliResult::ok(format!(
        "{}\n",
        serde_json::json!({
            "source": path.display().to_string(),
            "totalCount": summary.total_count,
            "presetCount": summary.preset_count,
            "supportedCount": summary.supported_count,
            "unsupportedCount": summary.unsupported_count,
            "webGpuSupportedCount": summary.webgpu_supported_count,
            "webGpuUnsupportedCount": summary.webgpu_unsupported_count,
            "maxShapeCount": summary.max_shape_count,
            "maxSpriteCount": summary.max_sprite_count,
            "maxWaveCount": summary.max_wave_count,
            "maxQRegisterIndex": summary.max_q_register_index,
            "qRegisters": summary.q_registers,
            "unsupportedFunctions": summary.unsupported_functions,
            "unsupportedShaderSections": summary.unsupported_shader_sections,
            "webGpuUnsupportedShaderSections": summary.webgpu_unsupported_shader_sections,
            "entries": entries.iter().map(|entry| serde_json::json!({
                "id": entry.id,
                "fileName": entry.file_name,
                "format": entry.format,
                "presetCount": entry.preset_count,
                "supported": entry.supported,
                "webGpuSupported": entry.webgpu_supported,
                "unsupportedFunctions": entry.unsupported_functions,
                "shaderSections": entry.shader_sections,
                "webGpuShaderSections": entry.webgpu_shader_sections,
            })).collect::<Vec<_>>(),
        })
    ))
}

fn collect_preset_files(path: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    if path.is_file() {
        return Ok(if is_preset_file(path) {
            vec![path.to_path_buf()]
        } else {
            Vec::new()
        });
    }
    let mut files = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry_path.is_dir() {
            files.extend(collect_preset_files(&entry_path)?);
        } else if is_preset_file(&entry_path) {
            files.push(entry_path);
        }
    }
    files.sort();
    Ok(files)
}

fn is_preset_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            extension.eq_ignore_ascii_case("milk") || extension.eq_ignore_ascii_case("milk2")
        })
        .unwrap_or(false)
}

fn read_one_path(path: Option<&String>) -> Result<(PathBuf, String), RustyMilkCliResult> {
    let Some(path) = path else {
        return Err(RustyMilkCliResult::err(2, usage()));
    };
    let path = PathBuf::from(path);
    match fs::read_to_string(&path) {
        Ok(source) => Ok((path, source)),
        Err(error) => Err(RustyMilkCliResult::err(
            1,
            format!("failed to read {}: {error}\n", path.display()),
        )),
    }
}

fn usage() -> String {
    "usage: rustymilk <validate|inspect|compat|render-stats> <file-or-directory>\n".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "rustymilk-cli-test-{}-{name}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    fn write_preset(name: &str, source: &str) -> PathBuf {
        let path = temp_path(name);
        fs::write(&path, source).unwrap();
        path
    }

    fn smoke_source() -> &'static str {
        "name=CLI Smoke\n\
         decay=0.9\n\
         wave_r=0.8\n\
         wave_g=0.4\n\
         wave_b=0.2\n\
         wave_a=0.9\n\
         shape00_enabled=1\n\
         shape00_sides=5\n\
         shape00_rad=0.2\n\
         shape00_a=0.4\n\
         wavecode_0_enabled=1\n\
         wavecode_0_samples=16\n\
         wavecode_0_per_point1=x=i;\n\
         wavecode_0_per_point2=y=0.5+sample*0.25;\n"
    }

    #[test]
    fn validate_reports_valid_preset_as_json() {
        let path = write_preset("valid.milk", smoke_source());
        let result = run_rustymilk_cli(&["validate".to_string(), path.display().to_string()]);
        let _ = fs::remove_file(&path);

        assert_eq!(result.code, 0);
        let value: serde_json::Value = serde_json::from_str(&result.stdout).unwrap();
        assert_eq!(value["status"], "valid");
        assert_eq!(value["title"], "CLI Smoke");
    }

    #[test]
    fn inspect_reports_preset_structure() {
        let path = write_preset("inspect.milk", smoke_source());
        let result = run_rustymilk_cli(&["inspect".to_string(), path.display().to_string()]);
        let _ = fs::remove_file(&path);

        assert_eq!(result.code, 0);
        let value: serde_json::Value = serde_json::from_str(&result.stdout).unwrap();
        assert_eq!(value["presetCount"], 1);
        assert_eq!(value["presets"][0]["shapeCount"], 1);
        assert_eq!(value["presets"][0]["waveCount"], 1);
    }

    #[test]
    fn render_stats_reports_headless_geometry() {
        let path = write_preset("render.milk", smoke_source());
        let result = run_rustymilk_cli(&["render-stats".to_string(), path.display().to_string()]);
        let _ = fs::remove_file(&path);

        assert_eq!(result.code, 0);
        let value: serde_json::Value = serde_json::from_str(&result.stdout).unwrap();
        assert!(value["triangleVertices"].as_u64().unwrap() > 0);
        assert!(value["batches"]["filledVertices"].as_u64().unwrap() > 0);
    }

    #[test]
    fn compat_walks_directory_trees() {
        let dir = temp_path("compat-dir");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("nested.milk");
        fs::write(&path, smoke_source()).unwrap();

        let result = run_rustymilk_cli(&["compat".to_string(), dir.display().to_string()]);
        let _ = fs::remove_dir_all(&dir);

        assert_eq!(result.code, 0);
        let value: serde_json::Value = serde_json::from_str(&result.stdout).unwrap();
        assert_eq!(value["totalCount"], 1);
        assert_eq!(value["entries"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn missing_command_returns_usage_error() {
        let result = run_rustymilk_cli(&[]);
        assert_eq!(result.code, 2);
        assert!(result.stderr.contains("usage: rustymilk"));
    }
}

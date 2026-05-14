use std::{fs, path::PathBuf};

use rustymilk_core::{
    build_rustymilk_compatibility_entry, summarize_rustymilk_compatibility_matrix,
};

use crate::{
    collect_headless_frames, collect_pack_plugins, collect_preset_inputs, parse_non_negative_f64,
    parse_positive_f64, parse_positive_usize, write_pack_plugin_report, DesktopAudioProfile,
    DesktopSessionConfig, PresetInput,
};

#[derive(Clone, Debug)]
struct PlayerReport {
    preset_index: usize,
    source_label: String,
    source_path: String,
    source_title: String,
    frame_total: usize,
    final_time_seconds: f64,
}

pub fn run_desktop_player(args: &[String]) -> i32 {
    let mut args = args.iter();
    let mut preset_inputs: Vec<PathBuf> = Vec::new();
    let mut pack_inputs: Vec<PathBuf> = Vec::new();
    let mut frames = 240usize;
    let mut fps = 60.0f64;
    let mut waveform_size = 64usize;
    let mut spectrum_size = 64usize;
    let mut emit_json = false;
    let mut frame_log_every = 0usize;
    let mut include_frame_records = false;
    let mut profile = DesktopAudioProfile::default();
    let mut plugin_report: Option<PathBuf> = None;

    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--help" | "-h" => {
                print_player_help();
                return 0;
            }
            "--json" => emit_json = true,
            "--preset" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --preset value");
                    print_player_help();
                    return 2;
                };
                preset_inputs.push(PathBuf::from(value));
            }
            "--pack" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --pack value");
                    print_player_help();
                    return 2;
                };
                pack_inputs.push(PathBuf::from(value));
            }
            "--frames" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --frames value");
                    print_player_help();
                    return 2;
                };
                frames = match parse_positive_usize(value) {
                    Ok(value) => value,
                    Err(error) => {
                        eprintln!("invalid --frames value: {error}");
                        return 2;
                    }
                };
            }
            "--fps" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --fps value");
                    print_player_help();
                    return 2;
                };
                fps = match parse_positive_f64(value) {
                    Ok(value) => value,
                    Err(error) => {
                        eprintln!("invalid --fps value: {error}");
                        return 2;
                    }
                };
            }
            "--waveform-size" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --waveform-size value");
                    print_player_help();
                    return 2;
                };
                waveform_size = match parse_positive_usize(value) {
                    Ok(value) => value,
                    Err(error) => {
                        eprintln!("invalid --waveform-size value: {error}");
                        return 2;
                    }
                };
            }
            "--spectrum-size" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --spectrum-size value");
                    print_player_help();
                    return 2;
                };
                spectrum_size = match parse_positive_usize(value) {
                    Ok(value) => value,
                    Err(error) => {
                        eprintln!("invalid --spectrum-size value: {error}");
                        return 2;
                    }
                };
            }
            "--seed" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --seed value");
                    print_player_help();
                    return 2;
                };
                profile.seed = match parse_non_negative_f64(value) {
                    Ok(value) => value,
                    Err(error) => {
                        eprintln!("invalid --seed value: {error}");
                        return 2;
                    }
                };
            }
            "--noise" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --noise value");
                    print_player_help();
                    return 2;
                };
                profile.noise = match parse_non_negative_f64(value) {
                    Ok(value) => value,
                    Err(error) => {
                        eprintln!("invalid --noise value: {error}");
                        return 2;
                    }
                };
            }
            "--bpm" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --bpm value");
                    print_player_help();
                    return 2;
                };
                profile.bpm = match parse_positive_f64(value) {
                    Ok(value) => value,
                    Err(error) => {
                        eprintln!("invalid --bpm value: {error}");
                        return 2;
                    }
                };
            }
            "--bass-gain" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --bass-gain value");
                    print_player_help();
                    return 2;
                };
                profile.bass_gain = match parse_non_negative_f64(value) {
                    Ok(value) => value,
                    Err(error) => {
                        eprintln!("invalid --bass-gain value: {error}");
                        return 2;
                    }
                };
            }
            "--mid-gain" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --mid-gain value");
                    print_player_help();
                    return 2;
                };
                profile.mid_gain = match parse_non_negative_f64(value) {
                    Ok(value) => value,
                    Err(error) => {
                        eprintln!("invalid --mid-gain value: {error}");
                        return 2;
                    }
                };
            }
            "--treble-gain" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --treble-gain value");
                    print_player_help();
                    return 2;
                };
                profile.treble_gain = match parse_non_negative_f64(value) {
                    Ok(value) => value,
                    Err(error) => {
                        eprintln!("invalid --treble-gain value: {error}");
                        return 2;
                    }
                };
            }
            "--frame-log-every" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --frame-log-every value");
                    print_player_help();
                    return 2;
                };
                frame_log_every = match parse_positive_usize(value) {
                    Ok(value) => value,
                    Err(error) => {
                        eprintln!("invalid --frame-log-every value: {error}");
                        return 2;
                    }
                };
            }
            "--include-frame-records" => {
                include_frame_records = true;
            }
            "--plugin-report" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --plugin-report value");
                    print_player_help();
                    return 2;
                };
                plugin_report = Some(PathBuf::from(value));
            }
            unknown => {
                eprintln!("Unknown option: {unknown}");
                print_player_help();
                return 2;
            }
        }
    }

    let selected_presets = match collect_preset_inputs(&preset_inputs, &pack_inputs) {
        Ok(inputs) => inputs,
        Err(error) => {
            eprintln!("Either --preset or --pack is required.");
            eprintln!("Failed to collect source list: {error}");
            print_player_help();
            return 2;
        }
    };

    let (pack_plugins, pack_plugin_error) = match collect_pack_plugins(&pack_inputs) {
        Ok(plugins) => (plugins, None),
        Err(error) => {
            eprintln!("Failed to collect pack plugins: {error}");
            (Vec::new(), Some(error))
        }
    };

    if let Some(report_path) = plugin_report.as_deref() {
        if let Err(error) = write_pack_plugin_report(
            report_path,
            "player",
            &pack_inputs,
            &pack_plugins,
            pack_plugin_error.as_deref(),
        ) {
            eprintln!("Failed to write plugin report '{report_path:?}': {error}");
            return 2;
        }
    }

    let config = DesktopSessionConfig {
        frames,
        fps,
        waveform_size,
        spectrum_size,
        audio_profile: profile,
    };

    let mut playlist = Vec::with_capacity(selected_presets.len());
    let mut presets_summary = Vec::new();
    let mut compatibility_inputs = Vec::with_capacity(selected_presets.len());
    for (preset_index, preset) in selected_presets.iter().enumerate() {
        let source = match fs::read_to_string(&preset.source_path) {
            Ok(value) => value,
            Err(error) => {
                eprintln!(
                    "Failed to load preset '{}': {error}",
                    preset.source_path.display()
                );
                return 1;
            }
        };
        let frame_records = match collect_headless_frames(&source, config.clone()) {
            Ok(value) => value,
            Err(error) => {
                eprintln!(
                    "Session error while reading {}: {error}",
                    preset.source_path.display()
                );
                return 2;
            }
        };
        let file_name = preset.source_path.to_string_lossy().to_string();
        let force_milk2 = preset
            .source_path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("milk2"));
        let entry = build_rustymilk_compatibility_entry(
            &preset.source_label,
            &file_name,
            &source,
            force_milk2,
        );
        compatibility_inputs.push(entry);
        let summary = frame_records
            .last()
            .map(|frame| PlayerReport {
                preset_index,
                source_label: preset.source_label.clone(),
                source_path: preset.source_path.to_string_lossy().to_string(),
                source_title: frame.source_title.clone(),
                frame_total: frame_records.len(),
                final_time_seconds: frame.timing.time_seconds,
            })
            .unwrap_or_else(|| PlayerReport {
                preset_index,
                source_label: preset.source_label.clone(),
                source_path: preset.source_path.to_string_lossy().to_string(),
                source_title: String::from("unknown"),
                frame_total: 0,
                final_time_seconds: 0.0,
            });
        playlist.push(summary);
        presets_summary.push((preset, frame_records));
    }

    if emit_json {
        print_player_json(
            &presets_summary,
            &playlist,
            &compatibility_inputs,
            frames,
            fps,
            frame_log_every,
            include_frame_records,
            &profile,
            &pack_plugins,
        );
    } else {
        print_player_text(
            &playlist,
            frames,
            fps,
            frame_log_every,
            include_frame_records,
            &presets_summary,
            &pack_plugins,
        );
    }

    0
}

fn print_player_help() {
    println!("rustymilk-desktop player - session-oriented desktop runtime probe");
    println!();
    println!("Usage:");
    println!("  cargo run -p rustymilk-desktop --bin player -- [options]");
    println!();
    println!("Options:");
    println!("  --preset <path>         Path to a .milk/.milk2 source file (repeatable)");
    println!("  --pack <path>           Path to a RustyMilk pack folder or manifest");
    println!("  --frames <count>        Number of frames per preset (default: 240)");
    println!("  --fps <value>           Frame rate used for timing (default: 60)");
    println!("  --frame-log-every <n>   Print every N-th frame within each preset (default: 0)");
    println!("  --include-frame-records  Include per-frame payload in JSON output");
    println!(
        "  --waveform-size <n>     Number of waveform samples to generate (default: 64, minimum: 1)"
    );
    println!(
        "  --spectrum-size <n>     Number of spectrum bins to generate (default: 64, minimum: 1)"
    );
    println!("  --bpm <value>           Synthetic BPM for the audio profile (default: 128)");
    println!("  --noise <value>          Profile noise amount (default: 0.08)");
    println!("  --bass-gain <value>      Bass gain (default: 0.95)");
    println!("  --mid-gain <value>       Mid gain (default: 0.74)");
    println!("  --treble-gain <value>    Treble gain (default: 0.62)");
    println!("  --seed <value>           Audio phase seed (default: 0.35)");
    println!("  --plugin-report <path>   Write discovered pack plugin report JSON to path");
    println!("  --json                 Emit a JSON report");
    println!("  --help                 Show this help");
}

fn print_player_text(
    playlist: &[PlayerReport],
    frames: usize,
    fps: f64,
    frame_log_every: usize,
    include_frame_records: bool,
    presets_summary: &[(&PresetInput, Vec<crate::DesktopSessionFrame>)],
    pack_plugins: &[crate::PackPluginInput],
) {
    println!(
        "RustyMilk desktop player: {} presets, {} frames each @ {fps} fps",
        playlist.len(),
        frames
    );
    println!("Compatibility mode: simulated audio only");
    for report in playlist {
        println!(
            "Playlist slot #{:02}: {} ({})",
            report.preset_index, report.source_label, report.source_path
        );
        println!("  Source: {}", report.source_path);
        println!("  Preset title: {}", report.source_title);
        println!("  Frames generated: {}", report.frame_total);
        println!("  Final simulated time: {:.3}s", report.final_time_seconds);
    }

    if include_frame_records && !presets_summary.is_empty() {
        let step = frame_log_every.max(1);
        for (preset, frames) in presets_summary {
            if frames.is_empty() {
                continue;
            }
            println!("Frame log for {}", preset.source_label);
            for frame in frames.iter().step_by(step) {
                println!(
                    "  t={:.3}s index={} vertices={}/{}",
                    frame.timing.time_seconds,
                    frame.timing.frame_index,
                    frame.report.line_vertices + frame.report.point_vertices,
                    frame.report.triangle_vertices,
                );
            }
        }
    }

    println!("Pack plugins discovered: {}", pack_plugins.len());
    for plugin in pack_plugins {
        println!("  - {} ({})", plugin.id, plugin.kind);
        println!("    entry: {}", plugin.entry);
        if let Some(payload) = plugin.payload.as_object() {
            if !payload.is_empty() {
                println!(
                    "    payload keys: {}",
                    payload.keys().cloned().collect::<Vec<_>>().join(", ")
                );
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn print_player_json(
    presets_summary: &[(&PresetInput, Vec<crate::DesktopSessionFrame>)],
    playlist: &[PlayerReport],
    compatibility_inputs: &[rustymilk_core::RustyMilkCompatibilityEntry],
    frames: usize,
    fps: f64,
    frame_log_every: usize,
    include_frame_records: bool,
    profile: &DesktopAudioProfile,
    pack_plugins: &[crate::PackPluginInput],
) {
    let total_vertices: usize = presets_summary
        .iter()
        .map(|(_, frames)| {
            frames
                .iter()
                .map(|frame| {
                    frame.report.line_vertices
                        + frame.report.point_vertices
                        + frame.report.textured_vertices
                        + frame.report.triangle_vertices
                })
                .sum::<usize>()
        })
        .sum();

    let compatibility = summarize_rustymilk_compatibility_matrix(compatibility_inputs);

    let presets = presets_summary
        .iter()
        .map(|(preset, frames)| {
            if include_frame_records {
                serde_json::json!({
                    "label": preset.source_label,
                    "path": preset.source_path.to_string_lossy(),
                    "frameCount": frames.len(),
                    "frames": frames.iter().map(|frame| {
                        serde_json::json!({
                            "index": frame.timing.frame_index,
                            "timeSeconds": frame.timing.time_seconds,
                            "lineVertices": frame.report.line_vertices,
                            "pointVertices": frame.report.point_vertices,
                            "texturedVertices": frame.report.textured_vertices,
                            "triangleVertices": frame.report.triangle_vertices,
                        })
                    })
                    .collect::<Vec<_>>(),
                })
            } else {
                serde_json::json!({
                    "label": preset.source_label,
                    "path": preset.source_path.to_string_lossy(),
                    "frameCount": frames.len(),
                })
            }
        })
        .collect::<Vec<_>>();

    let json = serde_json::json!({
        "mode": "player",
        "presets": playlist.len(),
        "framesPerPreset": frames,
        "fps": fps,
        "frameLogEvery": frame_log_every,
        "includeFrameRecords": include_frame_records,
        "totalVertices": total_vertices,
        "audioProfile": {
            "bpm": profile.bpm,
            "seed": profile.seed,
            "noise": profile.noise,
            "bassGain": profile.bass_gain,
            "midGain": profile.mid_gain,
            "trebleGain": profile.treble_gain,
        },
        "playlist": playlist.iter().map(|report| {
            serde_json::json!({
                "index": report.preset_index,
                "label": report.source_label,
                "path": report.source_path,
                "title": report.source_title,
                "frames": report.frame_total,
                "finalTimeSeconds": report.final_time_seconds,
            })
        }).collect::<Vec<_>>(),
        "compatibility": {
            "supportedCount": compatibility.supported_count,
            "unsupportedCount": compatibility.unsupported_count,
            "presetCount": compatibility.preset_count,
        },
        "sessions": presets,
        "packPlugins": {
            "count": pack_plugins.len(),
            "entries": pack_plugins
                .iter()
                .map(|plugin| {
                    serde_json::json!({
                        "id": plugin.id,
                        "kind": plugin.kind,
                        "entry": plugin.entry,
                        "sourcePath": plugin.source_path.to_string_lossy(),
                        "payloadKeyCount": plugin.payload.as_object().map(|payload| payload.len()).unwrap_or(0),
                    })
                })
                .collect::<Vec<_>>(),
        },
    });
    let json = if include_frame_records {
        json
    } else {
        serde_json::json!({
            "mode": "player",
            "presets": playlist.len(),
            "framesPerPreset": frames,
            "fps": fps,
            "frameLogEvery": frame_log_every,
            "includeFrameRecords": include_frame_records,
            "totalVertices": total_vertices,
            "audioProfile": {
                "bpm": profile.bpm,
                "seed": profile.seed,
                "noise": profile.noise,
                "bassGain": profile.bass_gain,
                "midGain": profile.mid_gain,
                "trebleGain": profile.treble_gain,
            },
            "playlist": playlist.iter().map(|report| {
                serde_json::json!({
                    "index": report.preset_index,
                    "label": report.source_label,
                    "path": report.source_path,
                    "title": report.source_title,
                    "frames": report.frame_total,
                    "finalTimeSeconds": report.final_time_seconds,
                })
            }).collect::<Vec<_>>(),
            "compatibility": {
                "supportedCount": compatibility.supported_count,
                "unsupportedCount": compatibility.unsupported_count,
                "presetCount": compatibility.preset_count,
            },
            "sessions": presets,
            "packPlugins": {
                "count": pack_plugins.len(),
                "entries": pack_plugins
                    .iter()
                    .map(|plugin| {
                        serde_json::json!({
                            "id": plugin.id,
                            "kind": plugin.kind,
                            "entry": plugin.entry,
                            "sourcePath": plugin.source_path.to_string_lossy(),
                            "payloadKeyCount": plugin.payload.as_object().map(|payload| payload.len()).unwrap_or(0),
                        })
                    })
                    .collect::<Vec<_>>(),
            },
        })
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| json.to_string())
    );
}

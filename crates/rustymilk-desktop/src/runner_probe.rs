use std::{fs, path::PathBuf};

use crate::{
    collect_headless_frames, collect_pack_plugins, collect_preset_inputs, parse_non_negative_f64,
    parse_positive_f64, parse_positive_usize, write_pack_plugin_report, DesktopAudioProfile,
    DesktopSessionConfig, PresetInput,
};

pub fn run_desktop_probe(args: &[String]) -> i32 {
    let mut args = args.iter();
    let mut preset_inputs: Vec<PathBuf> = Vec::new();
    let mut pack_inputs: Vec<PathBuf> = Vec::new();
    let mut frames = 240usize;
    let mut fps = 60.0f64;
    let mut waveform_size = 64usize;
    let mut spectrum_size = 64usize;
    let mut emit_json = false;
    let mut profile = DesktopAudioProfile::default();
    let mut plugin_report: Option<PathBuf> = None;

    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--help" | "-h" => {
                print_probe_help();
                return 0;
            }
            "--json" => emit_json = true,
            "--preset" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --preset value");
                    print_probe_help();
                    return 2;
                };
                preset_inputs.push(PathBuf::from(value));
            }
            "--pack" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --pack value");
                    print_probe_help();
                    return 2;
                };
                pack_inputs.push(PathBuf::from(value));
            }
            "--frames" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --frames value");
                    print_probe_help();
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
                    print_probe_help();
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
                    print_probe_help();
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
                    print_probe_help();
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
                    print_probe_help();
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
                    print_probe_help();
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
                    print_probe_help();
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
                    print_probe_help();
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
                    print_probe_help();
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
                    print_probe_help();
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
            "--plugin-report" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --plugin-report value");
                    print_probe_help();
                    return 2;
                };
                plugin_report = Some(PathBuf::from(value));
            }
            unknown => {
                eprintln!("Unknown option: {unknown}");
                print_probe_help();
                return 2;
            }
        }
    }

    let selected_presets = match collect_preset_inputs(&preset_inputs, &pack_inputs) {
        Ok(inputs) => inputs,
        Err(error) => {
            eprintln!("Either --preset or --pack is required.");
            eprintln!("Failed to collect source list: {error}");
            print_probe_help();
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
            "probe",
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

    let mut session_reports = Vec::with_capacity(selected_presets.len());
    for preset in &selected_presets {
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
        let summary = crate::summarize_headless_frames(
            &preset.source_path.to_string_lossy(),
            &preset.source_label,
            frame_records.clone(),
        );
        session_reports.push((preset, frame_records, summary));
    }

    if emit_json {
        print_probe_json(&session_reports, frames, fps, &profile, &pack_plugins);
    } else {
        print_probe_text(&session_reports, fps, &pack_plugins);
    }

    0
}

fn print_probe_help() {
    println!("rustymilk-desktop - prototype RustyMilk headless desktop session runner");
    println!();
    println!("Usage:");
    println!("  cargo run -p rustymilk-desktop --bin rustymilk-desktop -- [options]");
    println!();
    println!("Options:");
    println!("  --preset <path>    Path to a .milk/.milk2 source file (repeatable)");
    println!("  --pack <path>      Path to a RustyMilk pack folder or manifest");
    println!("  --frames <count>   Number of frames to simulate per preset (default: 240)");
    println!("  --fps <value>      Frame rate used for timing (default: 60)");
    println!(
        "  --waveform-size <n>  Number of waveform samples to generate (default: 64, minimum: 1)"
    );
    println!(
        "  --spectrum-size <n>  Number of spectrum bins to generate (default: 64, minimum: 1)"
    );
    println!("  --bpm <value>      Synthetic BPM for the probe audio profile (default: 128)");
    println!("  --noise <value>     Profile noise amount (default: 0.08)");
    println!("  --bass-gain <value> Bass gain (default: 0.95)");
    println!("  --mid-gain <value>  Mid gain (default: 0.74)");
    println!("  --treble-gain <value> Treble gain (default: 0.62)");
    println!("  --seed <value>      Audio phase seed (default: 0.35)");
    println!("  --plugin-report <path> Write discovered pack plugin report JSON to path");
    println!("  --json             Emit a JSON report");
    println!("  --help             Show this help");
    println!();
    println!("Examples:");
    println!(
        "  cargo run -p rustymilk-desktop --bin rustymilk-desktop -- --preset examples/sample-pack/presets/warm-scope.milk --frames 120 --fps 60"
    );
    println!("  cargo run -p rustymilk-desktop --bin rustymilk-desktop -- --pack examples/sample-pack --frames 24 --json");
    println!();
    println!("Subcommands:");
    println!("  cargo run -p rustymilk-desktop --bin player  for player-mode simulation output");
    println!(
        "  cargo run -p rustymilk-desktop --bin studio  for studio-style compatibility inspection"
    );
}

fn print_probe_text(
    session_reports: &[(
        &PresetInput,
        Vec<crate::DesktopSessionFrame>,
        crate::DesktopSessionSummary,
    )],
    fps: f64,
    pack_plugins: &[crate::PackPluginInput],
) {
    println!(
        "RustyMilk desktop probe: {} presets, {} frames each @ {fps} fps",
        session_reports.len(),
        session_reports
            .first()
            .map(|(_, frames, _)| frames.len())
            .unwrap_or(0),
    );
    for (index, (preset, _frame_records, summary)) in session_reports.iter().enumerate() {
        println!(
            "Preset #{index}: {} ({})",
            preset.source_label,
            preset.source_path.display()
        );
        println!("  Source title: {}", summary.source_line);
        println!("  Source report: {}", summary.source);
        println!("  Frames: {}", summary.frames);
        println!(
            "  Average line vertices: {:.2}",
            summary.average_line_vertices
        );
        println!(
            "  Average point vertices: {:.2}",
            summary.average_point_vertices
        );
        println!(
            "  Average textured vertices: {:.2}",
            summary.average_textured_vertices
        );
        println!(
            "  Average triangle vertices: {:.2}",
            summary.average_triangle_vertices
        );
        println!("  Final frame time: {:.2}s", summary.timing.time_seconds);
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

fn print_probe_json(
    session_reports: &[(
        &PresetInput,
        Vec<crate::DesktopSessionFrame>,
        crate::DesktopSessionSummary,
    )],
    frames: usize,
    fps: f64,
    profile: &DesktopAudioProfile,
    pack_plugins: &[crate::PackPluginInput],
) {
    let sessions = session_reports
        .iter()
        .map(|(preset, frame_records, summary)| {
            serde_json::json!({
                "source": preset.source_path.to_string_lossy(),
                "label": preset.source_label,
                "frameCount": summary.frames,
                "sourceTitle": summary.source_line,
                "finalFrame": summary.timing.frame_index,
                "finalTimeSeconds": summary.timing.time_seconds,
                "averageLineVertices": summary.average_line_vertices,
                "averagePointVertices": summary.average_point_vertices,
                "averageTexturedVertices": summary.average_textured_vertices,
                "averageTriangleVertices": summary.average_triangle_vertices,
                "frames": frame_records.iter().map(|frame| {
                    serde_json::json!({
                        "frame": frame.timing.frame_index,
                        "timeSeconds": frame.timing.time_seconds,
                        "presetTitle": frame.source_title,
                        "presetCount": frame.preset_count,
                        "transitionMode": frame.transition_mode,
                        "transitionSeconds": frame.transition_seconds,
                        "lineVertices": frame.report.line_vertices,
                        "pointVertices": frame.report.point_vertices,
                        "texturedVertices": frame.report.textured_vertices,
                        "triangleVertices": frame.report.triangle_vertices,
                    })
                }).collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>();

    let json = serde_json::json!({
        "mode": "probe",
        "presets": session_reports.len(),
        "framesTotal": session_reports.iter().map(|(_, frames, _)| frames.len()).sum::<usize>(),
        "globalFramesPerPreset": frames,
        "fps": fps,
        "audioProfile": {
            "bpm": profile.bpm,
            "seed": profile.seed,
            "noise": profile.noise,
            "bassGain": profile.bass_gain,
            "midGain": profile.mid_gain,
            "trebleGain": profile.treble_gain,
        },
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
        "sessions": sessions,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| json.to_string())
    );
}

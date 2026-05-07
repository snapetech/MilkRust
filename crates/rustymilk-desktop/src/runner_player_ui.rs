use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use minifb::{Key, Window, WindowOptions};
use rustymilk_core::{RustyMilkFrameSet, RustyMilkPrimitiveMode, RustyMilkTexturedPrimitiveMode};

use crate::{
    collect_pack_plugins, parse_non_negative_f64, parse_positive_f64, parse_positive_usize,
    write_pack_plugin_report, DesktopAudioProfile, DesktopPlayerEngine, DesktopPlayerEngineConfig,
    DesktopPlayerError, RustyMilkFrameSetReport,
};

#[cfg(feature = "audio")]
use crate::CpalDesktopAudioProvider;

#[derive(Default, Clone, Debug)]
struct UiInputState {
    left_prev: bool,
    right_prev: bool,
    space_prev: bool,
    r_prev: bool,
}

const DEFAULT_UI_WIDTH: usize = 960;
const DEFAULT_UI_HEIGHT: usize = 540;
const DEFAULT_UI_PRESET_DURATION_SECONDS: f64 = 20.0;

pub fn run_desktop_player_ui(args: &[String]) -> i32 {
    let mut args = args.iter();
    let mut preset_inputs: Vec<PathBuf> = Vec::new();
    let mut pack_inputs: Vec<PathBuf> = Vec::new();
    let mut fps = 60.0f64;
    let mut width = DEFAULT_UI_WIDTH;
    let mut height = DEFAULT_UI_HEIGHT;
    let mut preset_duration_seconds = DEFAULT_UI_PRESET_DURATION_SECONDS;
    let mut waveform_size = 64usize;
    let mut spectrum_size = 64usize;
    let mut profile = DesktopAudioProfile::default();
    let mut auto_loop = true;
    let mut start_paused = false;
    let mut start_preset = None::<String>;
    let mut plugin_report: Option<PathBuf> = None;
    #[cfg(feature = "audio")]
    let mut audio_device: Option<String> = None;
    #[cfg(feature = "audio")]
    let mut list_audio_devices = false;

    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--help" | "-h" => {
                print_player_ui_help();
                return 0;
            }
            "--preset" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --preset value");
                    print_player_ui_help();
                    return 2;
                };
                preset_inputs.push(PathBuf::from(value));
            }
            "--pack" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --pack value");
                    print_player_ui_help();
                    return 2;
                };
                pack_inputs.push(PathBuf::from(value));
            }
            "--fps" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --fps value");
                    print_player_ui_help();
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
            "--preset-duration" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --preset-duration value");
                    print_player_ui_help();
                    return 2;
                };
                preset_duration_seconds = match parse_non_negative_f64(value) {
                    Ok(value) => value,
                    Err(error) => {
                        eprintln!("invalid --preset-duration value: {error}");
                        return 2;
                    }
                };
            }
            "--width" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --width value");
                    print_player_ui_help();
                    return 2;
                };
                width = match parse_positive_usize(value) {
                    Ok(value) => value,
                    Err(error) => {
                        eprintln!("invalid --width value: {error}");
                        return 2;
                    }
                };
            }
            "--plugin-report" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --plugin-report value");
                    print_player_ui_help();
                    return 2;
                };
                plugin_report = Some(PathBuf::from(value));
            }
            "--height" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --height value");
                    print_player_ui_help();
                    return 2;
                };
                height = match parse_positive_usize(value) {
                    Ok(value) => value,
                    Err(error) => {
                        eprintln!("invalid --height value: {error}");
                        return 2;
                    }
                };
            }
            "--waveform-size" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --waveform-size value");
                    print_player_ui_help();
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
                    print_player_ui_help();
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
                    print_player_ui_help();
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
                    print_player_ui_help();
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
                    print_player_ui_help();
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
                    print_player_ui_help();
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
                    print_player_ui_help();
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
                    print_player_ui_help();
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
            "--no-loop" => {
                auto_loop = false;
            }
            "--pause-start" => {
                start_paused = true;
            }
            "--start-preset" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --start-preset value");
                    print_player_ui_help();
                    return 2;
                };
                start_preset = Some(value.to_string());
            }
            #[cfg(feature = "audio")]
            "--audio-device" => {
                let Some(value) = args.next() else {
                    eprintln!("missing --audio-device value");
                    print_player_ui_help();
                    return 2;
                };
                audio_device = Some(value.to_string());
            }
            #[cfg(feature = "audio")]
            "--list-audio-devices" => {
                list_audio_devices = true;
            }
            unknown => {
                eprintln!("Unknown option: {unknown}");
                print_player_ui_help();
                return 2;
            }
        }
    }

    #[cfg(feature = "audio")]
    if list_audio_devices {
        println!("Available input devices:");
        let devices = CpalDesktopAudioProvider::available_device_names();
        for (index, device) in devices.iter().enumerate() {
            println!("  {index:02}: {device}");
        }
        if devices.is_empty() {
            println!("  (no devices discovered)");
        }
        return 0;
    }

    #[cfg(feature = "audio")]
    let audio_provider = match audio_device {
        Some(device) => CpalDesktopAudioProvider::new_with_device_name(&device),
        None => CpalDesktopAudioProvider::new(),
    };

    #[cfg(feature = "audio")]
    let mut player = match audio_provider
        .map_err(|error| format!("Failed to initialize audio provider: {error}"))
        .and_then(|provider| {
            DesktopPlayerEngine::from_inputs_with_audio_provider_and_start_preset(
                &preset_inputs,
                &pack_inputs,
                DesktopPlayerEngineConfig {
                    fps,
                    preset_duration_seconds,
                    waveform_size,
                    spectrum_size,
                    audio_profile: profile,
                    auto_loop,
                    start_paused,
                },
                start_preset.as_deref(),
                provider,
            )
            .map_err(|error| format!("Failed to initialize player: {error}"))
        }) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("{error}");
            print_player_ui_help();
            return 2;
        }
    };

    #[cfg(not(feature = "audio"))]
    let mut player = match DesktopPlayerEngine::from_inputs_with_start_preset(
        &preset_inputs,
        &pack_inputs,
        DesktopPlayerEngineConfig {
            fps,
            preset_duration_seconds,
            waveform_size,
            spectrum_size,
            audio_profile: profile,
            auto_loop,
            start_paused,
        },
        start_preset.as_deref(),
    ) {
        Ok(value) => value,
        Err(error) => {
            let output = match error {
                DesktopPlayerError::NoPresets => {
                    "Either --preset or --pack is required.".to_string()
                }
                _ => format!("Failed to initialize player: {error}"),
            };
            eprintln!("{output}");
            if let DesktopPlayerError::InvalidPresetInput(details) = error {
                eprintln!("Failed to collect source list: {details}");
            }
            print_player_ui_help();
            return 2;
        }
    };

    let (pack_plugins, pack_plugin_error) = match collect_pack_plugins(&pack_inputs) {
        Ok(plugins) => (plugins, None),
        Err(error) => {
            eprintln!("Failed to load pack plugins: {error}");
            (Vec::new(), Some(error))
        }
    };
    if let Some(report_path) = plugin_report.as_deref() {
        if let Err(error) = write_pack_plugin_report(
            report_path,
            "player-ui",
            &pack_inputs,
            &pack_plugins,
            pack_plugin_error.as_deref(),
        ) {
            eprintln!("Failed to write plugin report '{report_path:?}': {error}");
            return 2;
        }
    }
    if !pack_plugins.is_empty() {
        let installed = player.install_pack_plugins(pack_plugins.clone());
        let skipped = pack_plugins.len().saturating_sub(installed);
        println!(
            "Loaded {} pack plugin(s), skipped {} unsupported kind(s).",
            installed, skipped
        );
        if !pack_plugins.is_empty() {
            println!("Pack plugin manifest:");
            for plugin in &pack_plugins {
                println!("  - {} ({})", plugin.id, plugin.kind);
                println!("    entry: {}", plugin.entry);
            }
        }
    }

    let mut window = match Window::new(
        "RustyMilk Desktop Player",
        width,
        height,
        WindowOptions::default(),
    ) {
        Ok(window) => window,
        Err(error) => {
            eprintln!("Unable to create window: {error}");
            return 1;
        }
    };
    window.limit_update_rate(Some(Duration::from_secs_f64(1.0 / fps.max(1.0))));

    let mut buffer = vec![0u32; width * height];
    let mut input_state = UiInputState::default();

    let mut next_frame_time = Instant::now();
    let frame_step = Duration::from_secs_f64(1.0 / fps.max(1.0));

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        let key_left = window.is_key_down(Key::Left);
        let key_right = window.is_key_down(Key::Right);
        let key_space = window.is_key_down(Key::Space);
        let key_r = window.is_key_down(Key::R);

        let next_preset = key_right && !input_state.right_prev;
        let previous_preset = key_left && !input_state.left_prev;
        let toggle_pause = key_space && !input_state.space_prev;
        let reset = key_r && !input_state.r_prev;

        input_state.left_prev = key_left;
        input_state.right_prev = key_right;
        input_state.space_prev = key_space;
        input_state.r_prev = key_r;

        if next_preset {
            player.next_preset();
        }
        if previous_preset {
            player.prev_preset();
        }
        if toggle_pause {
            player.toggle_running();
        }
        if reset {
            player.reset();
        }

        let now = Instant::now();
        if player.is_running() && now < next_frame_time {
            std::thread::sleep(next_frame_time - now);
            continue;
        }

        let frame = match player.frame() {
            Ok(value) => value,
            Err(error) => {
                eprintln!("Player frame error: {error}");
                return 2;
            }
        };
        let frame = match frame {
            Some(value) => value,
            None => {
                std::thread::sleep(Duration::from_millis(16));
                next_frame_time = now + frame_step;
                continue;
            }
        };
        render_frame_set(
            &mut buffer,
            width,
            height,
            &frame.frame_set,
            &frame.report,
            frame.preset_index,
            frame.preset_total,
            frame.time_seconds,
            frame.global_frame_index,
            player.is_running(),
            frame.local_frame_index,
        );

        if window.update_with_buffer(&buffer, width, height).is_err() {
            eprintln!("Window update failed");
            return 1;
        }
        window.set_title(&format!(
            "RustyMilk Desktop Player - {} ({}/{})",
            frame.preset_label,
            frame.preset_index + 1,
            frame.preset_total
        ));

        if player.is_running() {
            next_frame_time += frame_step;
            if next_frame_time < Instant::now() {
                next_frame_time = Instant::now();
            }
        } else {
            next_frame_time = now + frame_step;
        }
    }

    println!(
        "RustyMilk desktop UI playback complete: rendered {} frames across {} preset(s).",
        player.state().global_frame_index,
        player.preset_count()
    );
    0
}

fn render_frame_set(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    frame_set: &RustyMilkFrameSet,
    report: &RustyMilkFrameSetReport,
    preset_index: usize,
    preset_count: usize,
    time_seconds: f64,
    frame_index: usize,
    running: bool,
    local_frame: usize,
) {
    clear_buffer(
        buffer,
        palette_color(
            0.04 + (time_seconds.sin().abs() * 0.06),
            0.05 + (frame_index as f64 * 0.0003).fract() * 0.04,
            0.11,
            1.0,
        ),
    );

    for entry in &frame_set.entries {
        let frame = &entry.frame;
        let fade = frame.background_alpha.clamp(0.0, 0.35);
        if fade > 0.0 {
            fade_buffer(
                buffer,
                width,
                height,
                palette_color(0.0, 0.0, 0.0, fade.min(1.0)),
            );
        }

        for primitive in &frame.primitives {
            let color = palette_color(
                primitive.color[0],
                primitive.color[1],
                primitive.color[2],
                primitive.color[3],
            );
            match primitive.mode {
                RustyMilkPrimitiveMode::LineStrip => {
                    render_line_strip(buffer, width, height, &primitive.vertices, color);
                }
                RustyMilkPrimitiveMode::Lines => {
                    render_lines(buffer, width, height, &primitive.vertices, color);
                }
                RustyMilkPrimitiveMode::Points => {
                    render_points(buffer, width, height, &primitive.vertices, color);
                }
                RustyMilkPrimitiveMode::TriangleFan => {
                    render_triangle_fan_wireframe(
                        buffer,
                        width,
                        height,
                        &primitive.vertices,
                        color,
                    );
                }
                RustyMilkPrimitiveMode::Triangles => {
                    render_triangles_wireframe(buffer, width, height, &primitive.vertices, color);
                }
            }
        }

        for primitive in &frame.textured_primitives {
            let color = palette_color(
                primitive.color[0],
                primitive.color[1],
                primitive.color[2],
                primitive.color[3],
            );
            if primitive.vertices.len() < 2 {
                continue;
            }
            match primitive.mode {
                RustyMilkTexturedPrimitiveMode::Quad => {
                    render_quad_wireframe(buffer, width, height, &primitive.vertices, color);
                }
                RustyMilkTexturedPrimitiveMode::TriangleFan => {
                    render_triangle_fan_wireframe(
                        buffer,
                        width,
                        height,
                        &primitive.vertices,
                        color,
                    );
                }
            }
        }
    }

    let density = ((report.line_vertices
        + report.point_vertices
        + report.textured_vertices
        + report.triangle_vertices) as f64
        / 30000.0)
        .min(1.0);
    let hue = (time_seconds * 0.08).sin().abs();
    let density_fill = (density * 255.0) as u32;
    let marker = palette_color(hue, 0.8 * density + 0.2, 0.7, 0.5);
    overlay_debug(
        buffer,
        width,
        height,
        report,
        preset_index,
        preset_count,
        density_fill,
        marker,
        running,
        local_frame,
    );
}

fn overlay_debug(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    report: &RustyMilkFrameSetReport,
    preset_index: usize,
    preset_count: usize,
    density_fill: u32,
    accent: u32,
    running: bool,
    local_frame: usize,
) {
    if width == 0 || height == 0 {
        return;
    }
    let mut x = 8i32;
    let y_top = 8i32;
    let y_bottom = height as i32 - 9;
    let x_right = width as i32 - 9;
    while x <= x_right {
        draw_pixel(buffer, width, height, x, y_top, accent);
        draw_pixel(buffer, width, height, x, y_bottom, accent);
        x += 1;
    }

    let mut y = 8i32;
    while y <= y_bottom {
        draw_pixel(buffer, width, height, 8, y, accent);
        draw_pixel(buffer, width, height, x_right, y, accent);
        y += 1;
    }

    let density_bar_x = width / 6;
    let density_bar_y = height.saturating_sub(20);
    let density_width = density_fill * (width.saturating_sub(60) as u32) / 255;
    let bar_color = palette_color(0.3, 0.9, 0.65, 0.85);
    let outline = palette_color(0.2, 0.25, 0.3, 0.95);
    for x in density_bar_x..(width - density_bar_x) {
        let x_u32 = (x - density_bar_x) as u32;
        let color = if x_u32 <= density_width {
            bar_color
        } else {
            outline
        };
        draw_pixel(buffer, width, height, x as i32, density_bar_y as i32, color);
        draw_pixel(
            buffer,
            width,
            height,
            x as i32,
            density_bar_y.saturating_sub(1) as i32,
            outline,
        );
        draw_pixel(
            buffer,
            width,
            height,
            x as i32,
            density_bar_y.saturating_add(1) as i32,
            outline,
        );
    }

    let progress_max = (preset_count.max(1) * 5) as i32;
    let progress = ((preset_index as i32 + 1) * 5).min(progress_max).max(0);
    let progress_x = width.saturating_sub(14);
    for i in 0..progress {
        draw_pixel(buffer, width, height, progress_x as i32 - i, 12, bar_color);
    }
    let _ = report.source_title.len();

    let state_text_x = 18usize;
    let state_text_y = 18usize;
    let state_color = if running {
        palette_color(0.15, 0.95, 0.15, 1.0)
    } else {
        palette_color(0.95, 0.2, 0.2, 1.0)
    };
    if running {
        draw_pixel(
            buffer,
            width,
            height,
            state_text_x as i32,
            state_text_y as i32,
            state_color,
        );
        draw_pixel(
            buffer,
            width,
            height,
            state_text_x as i32 + 1,
            state_text_y as i32,
            state_color,
        );
        draw_pixel(
            buffer,
            width,
            height,
            state_text_x as i32 + 2,
            state_text_y as i32,
            state_color,
        );
    } else {
        draw_pixel(
            buffer,
            width,
            height,
            state_text_x as i32,
            state_text_y as i32,
            state_color,
        );
        draw_pixel(
            buffer,
            width,
            height,
            state_text_x as i32,
            state_text_y as i32 + 1,
            state_color,
        );
        draw_pixel(
            buffer,
            width,
            height,
            state_text_x as i32,
            state_text_y as i32 + 2,
            state_color,
        );
    }

    let local_marker = (local_frame % (width.saturating_sub(40))) + 20;
    draw_pixel(
        buffer,
        width,
        height,
        local_marker as i32,
        state_text_y as i32 + 6,
        palette_color(0.2, 0.4, 1.0, 0.8),
    );
}

fn render_line_strip(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    vertices: &[f64],
    color: u32,
) {
    let vertex_count = vertices.len() / 2;
    if vertex_count < 2 {
        return;
    }
    for index in 0..(vertex_count - 1) {
        let start = (index * 2, (index + 1) * 2);
        let a = to_pixel(vertices[start.0], vertices[start.0 + 1], width, height);
        let b = to_pixel(vertices[start.1], vertices[start.1 + 1], width, height);
        draw_segment(buffer, width, height, a.0, a.1, b.0, b.1, color);
    }
}

fn render_lines(buffer: &mut [u32], width: usize, height: usize, vertices: &[f64], color: u32) {
    let vertex_count = vertices.len() / 2;
    if vertex_count < 2 {
        return;
    }
    let mut index = 0usize;
    while index + 1 < vertex_count {
        let a = to_pixel(vertices[index * 2], vertices[index * 2 + 1], width, height);
        let next = index + 1;
        let b = to_pixel(vertices[next * 2], vertices[next * 2 + 1], width, height);
        draw_segment(buffer, width, height, a.0, a.1, b.0, b.1, color);
        index += 2;
    }
}

fn render_points(buffer: &mut [u32], width: usize, height: usize, vertices: &[f64], color: u32) {
    let radius = 1i32;
    for index in (0..vertices.len()).step_by(2) {
        if index + 1 >= vertices.len() {
            break;
        }
        let point = to_pixel(vertices[index], vertices[index + 1], width, height);
        for y in -radius..=radius {
            for x in -radius..=radius {
                draw_pixel(buffer, width, height, point.0 + x, point.1 + y, color);
            }
        }
    }
}

fn render_triangle_fan_wireframe(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    vertices: &[f64],
    color: u32,
) {
    let vertex_count = vertices.len() / 2;
    if vertex_count < 3 {
        return;
    }
    let center = to_pixel(vertices[0], vertices[1], width, height);
    let mut index = 1usize;
    while index < vertex_count {
        let current = to_pixel(vertices[index * 2], vertices[index * 2 + 1], width, height);
        draw_segment(
            buffer, width, height, center.0, center.1, current.0, current.1, color,
        );
        if index + 1 < vertex_count {
            let next = to_pixel(
                vertices[(index + 1) * 2],
                vertices[(index + 1) * 2 + 1],
                width,
                height,
            );
            draw_segment(
                buffer, width, height, current.0, current.1, next.0, next.1, color,
            );
        } else {
            let first = to_pixel(vertices[2], vertices[3], width, height);
            draw_segment(
                buffer, width, height, current.0, current.1, first.0, first.1, color,
            );
        }
        index += 1;
    }
}

fn render_triangles_wireframe(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    vertices: &[f64],
    color: u32,
) {
    let vertex_count = vertices.len() / 2;
    let mut index = 0usize;
    while index + 2 < vertex_count {
        let a = to_pixel(vertices[index * 2], vertices[index * 2 + 1], width, height);
        let b = to_pixel(
            vertices[(index + 1) * 2],
            vertices[(index + 1) * 2 + 1],
            width,
            height,
        );
        let c = to_pixel(
            vertices[(index + 2) * 2],
            vertices[(index + 2) * 2 + 1],
            width,
            height,
        );
        draw_segment(buffer, width, height, a.0, a.1, b.0, b.1, color);
        draw_segment(buffer, width, height, b.0, b.1, c.0, c.1, color);
        draw_segment(buffer, width, height, c.0, c.1, a.0, a.1, color);
        index += 3;
    }
}

fn render_quad_wireframe(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    vertices: &[f64],
    color: u32,
) {
    if vertices.len() < 8 {
        return;
    }
    let p0 = to_pixel(vertices[0], vertices[1], width, height);
    let p1 = to_pixel(vertices[2], vertices[3], width, height);
    let p2 = to_pixel(vertices[4], vertices[5], width, height);
    let p3 = to_pixel(vertices[6], vertices[7], width, height);
    draw_segment(buffer, width, height, p0.0, p0.1, p1.0, p1.1, color);
    draw_segment(buffer, width, height, p1.0, p1.1, p2.0, p2.1, color);
    draw_segment(buffer, width, height, p2.0, p2.1, p3.0, p3.1, color);
    draw_segment(buffer, width, height, p3.0, p3.1, p0.0, p0.1, color);
}

fn clear_buffer(buffer: &mut [u32], color: u32) {
    buffer.fill(color);
}

fn fade_buffer(buffer: &mut [u32], _width: usize, _height: usize, color: u32) {
    for pixel in buffer.iter_mut() {
        *pixel = blend_colors(*pixel, color);
    }
}

fn to_pixel(x: f64, y: f64, width: usize, height: usize) -> (i32, i32) {
    let sx = (width as f64 - 1.0).max(1.0);
    let sy = (height as f64 - 1.0).max(1.0);
    let nx = ((x * 0.5 + 0.5) * sx).round().clamp(0.0, sx);
    let ny = ((1.0 - (y * 0.5 + 0.5)) * sy).round().clamp(0.0, sy);
    (nx as i32, ny as i32)
}

fn draw_segment(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    color: u32,
) {
    let mut x0 = x0;
    let mut y0 = y0;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        draw_pixel(buffer, width, height, x0, y0, color);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn draw_pixel(buffer: &mut [u32], width: usize, height: usize, x: i32, y: i32, color: u32) {
    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
        return;
    }
    let idx = y as usize * width + x as usize;
    if idx < buffer.len() {
        buffer[idx] = blend_colors(buffer[idx], color);
    }
}

fn palette_color(r: f64, g: f64, b: f64, a: f64) -> u32 {
    let r = (r.clamp(0.0, 1.0) * 255.0) as u32;
    let g = (g.clamp(0.0, 1.0) * 255.0) as u32;
    let b = (b.clamp(0.0, 1.0) * 255.0) as u32;
    let a = (a.clamp(0.0, 1.0) * 255.0) as u32;
    (a << 24) | (r << 16) | (g << 8) | b
}

fn blend_colors(dst: u32, src: u32) -> u32 {
    let src_a = ((src >> 24) & 0xFF) as f64 / 255.0;
    if src_a >= 1.0 {
        return src;
    }
    if src_a <= 0.0 {
        return dst;
    }

    let dst_r = ((dst >> 16) & 0xFF) as f64;
    let dst_g = ((dst >> 8) & 0xFF) as f64;
    let dst_b = (dst & 0xFF) as f64;

    let src_r = ((src >> 16) & 0xFF) as f64;
    let src_g = ((src >> 8) & 0xFF) as f64;
    let src_b = (src & 0xFF) as f64;

    let out_r = src_r * src_a + dst_r * (1.0 - src_a);
    let out_g = src_g * src_a + dst_g * (1.0 - src_a);
    let out_b = src_b * src_a + dst_b * (1.0 - src_a);
    (0xFF << 24)
        | ((out_r.round() as u32) << 16)
        | ((out_g.round() as u32) << 8)
        | out_b.round() as u32
}

fn print_player_ui_help() {
    println!("rustymilk-desktop player-ui - lightweight desktop windowed mode");
    println!();
    println!("Usage:");
    println!("  cargo run -p rustymilk-desktop --features ui --bin player-ui -- [options]");
    println!();
    println!("Options:");
    println!("  --preset <path>           Path to a .milk/.milk2 source file (repeatable)");
    println!("  --pack <path>             Path to a RustyMilk pack folder or manifest");
    println!("  --width <pixels>          Window width (default: 960)");
    println!("  --height <pixels>         Window height (default: 540)");
    println!("  --fps <value>             Frame rate (default: 60)");
    println!("  --preset-duration <secs>   Frames per preset before cycling (default: 20)");
    println!("  --waveform-size <n>       Waveform samples (default: 64, minimum: 1)");
    println!("  --spectrum-size <n>       Spectrum bins (default: 64, minimum: 1)");
    println!("  --bpm <value>             Synthetic BPM (default: 128)");
    println!("  --noise <value>           Profile noise (default: 0.08)");
    println!("  --seed <value>            Audio phase seed (default: 0.35)");
    println!("  --bass-gain <value>       Bass gain (default: 0.95)");
    println!("  --mid-gain <value>        Mid gain (default: 0.74)");
    println!("  --treble-gain <value>     Treble gain (default: 0.62)");
    println!("  --plugin-report <path>    Write discovered pack plugin report JSON to path");
    println!("  --no-loop                 Stop playback after last preset");
    println!("  --pause-start             Start playback paused");
    println!("  --start-preset <index>    Start at packed preset index or label");
    #[cfg(feature = "audio")]
    println!("  --audio-device <name>     Use a named input device for live capture");
    #[cfg(feature = "audio")]
    println!("  --list-audio-devices      Show available input devices");
    println!("  --help                    Show this help");
    println!();
    println!("Note: Close the window or press ESC to exit.");
    println!("Controls: Left/Right = prev/next preset, Space = pause/resume, R = reset");
}

#[cfg(test)]
mod tests {
    use crate::player_api::{DesktopPlayerEngine, DesktopPlayerEngineConfig};
    use crate::PresetInput;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn player_ui_start_preset_lookup_delegates_to_engine_api() {
        let process_id = std::process::id();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|value| value.subsec_nanos())
            .unwrap_or(0);
        let base = std::env::temp_dir().join(format!(
            "rustymilk-desktop-ui-api-test-{process_id}-{nanos}"
        ));
        std::fs::create_dir_all(&base).unwrap();
        let first = base.join("a.milk");
        let second = base.join("b.milk");
        let third = base.join("c.milk");
        std::fs::write(&first, "name=first\n").unwrap();
        std::fs::write(&second, "name=second\n").unwrap();
        std::fs::write(&third, "name=third\n").unwrap();

        let presets = vec![
            PresetInput {
                source_path: first,
                source_label: "first".to_string(),
            },
            PresetInput {
                source_path: second,
                source_label: "second".to_string(),
            },
            PresetInput {
                source_path: third,
                source_label: "third".to_string(),
            },
        ];

        let mut engine = DesktopPlayerEngine::from_preset_inputs(
            presets,
            DesktopPlayerEngineConfig {
                fps: 60.0,
                preset_duration_seconds: 2.0,
                waveform_size: 64,
                spectrum_size: 64,
                audio_profile: crate::DesktopAudioProfile::default(),
                auto_loop: true,
                start_paused: false,
            },
        )
        .ok()
        .unwrap();

        engine.seek_preset("1").unwrap();
        assert_eq!(engine.preset_label(), Some("second"));
        assert_eq!(engine.state().preset_index, 1);

        assert!(engine.seek_preset("third").is_ok());
        assert_eq!(engine.state().preset_index, 2);

        assert!(engine.seek_preset("missing").is_err());
    }
}

use std::{
    collections::BTreeSet,
    fs,
    path::Path,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::Value;

use rustymilk_pack::load_rustymilk_pack_manifest;

#[derive(Clone, Debug)]
pub struct PresetInput {
    pub source_path: PathBuf,
    pub source_label: String,
}

#[derive(Clone, Debug)]
pub struct PackPluginInput {
    pub id: String,
    pub kind: String,
    pub entry: String,
    pub source_path: PathBuf,
    pub payload: Value,
}

fn normalize_plugin_kind(value: &str) -> String {
    let value = value.to_lowercase();
    match value.as_str() {
        "javascript" | "module" => "js".to_string(),
        _ => value,
    }
}

impl PackPluginInput {
    fn with_payload(
        id: String,
        kind: String,
        entry: String,
        source_path: PathBuf,
    ) -> Result<Self, String> {
        let normalized_kind = normalize_plugin_kind(&kind);
        let payload = if normalized_kind == "data" {
            let source = fs::read_to_string(&source_path).map_err(|error| {
                format!(
                    "failed to read plugin '{}' from {}: {error}",
                    id,
                    source_path.display()
                )
            })?;
            serde_json::from_str(&source).map_err(|error| {
                format!(
                    "plugin '{}' has invalid JSON payload '{}': {error}",
                    id,
                    source_path.display()
                )
            })?
        } else {
            Value::Null
        };

        Ok(Self {
            id,
            kind: normalized_kind,
            entry,
            source_path,
            payload,
        })
    }
}

pub fn parse_positive_usize(value: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .map_err(|error| format!("invalid integer '{value}': {error}"))
        .and_then(|parsed| {
            if parsed > 0 {
                Ok(parsed)
            } else {
                Err(format!("value must be greater than zero: '{value}'"))
            }
        })
}

pub fn parse_positive_f64(value: &str) -> Result<f64, String> {
    value
        .parse::<f64>()
        .map_err(|error| format!("invalid number '{value}': {error}"))
        .and_then(|parsed| {
            if parsed.is_finite() && parsed > 0.0 {
                Ok(parsed)
            } else {
                Err(format!("value must be a positive finite number: '{value}'"))
            }
        })
}

pub fn parse_non_negative_f64(value: &str) -> Result<f64, String> {
    value
        .parse::<f64>()
        .map_err(|error| format!("invalid number '{value}': {error}"))
        .and_then(|parsed| {
            if parsed.is_finite() && parsed >= 0.0 {
                Ok(parsed)
            } else {
                Err(format!("value must be a finite number: '{value}'"))
            }
        })
}

pub fn gather_preset_path_candidates(path: &Path) -> Result<Vec<PathBuf>, String> {
    if path.is_file() {
        match path.extension().and_then(|value| value.to_str()) {
            Some("milk") | Some("milk2") => return Ok(vec![path.to_path_buf()]),
            _ => {
                return Err(format!(
                    "unsupported preset file '{}': expected .milk or .milk2",
                    path.display()
                ))
            }
        }
    }
    if !path.is_dir() {
        return Err(format!("input path does not exist: {}", path.display()));
    }
    let mut stack = vec![path.to_path_buf()];
    let mut found = Vec::new();
    while let Some(current) = stack.pop() {
        let mut entries = fs::read_dir(&current)
            .map_err(|error| format!("failed to read '{}': {error}", current.display()))?;
        while let Some(entry) = entries.next().transpose().map_err(|error| {
            format!(
                "failed to enumerate directory '{}': {error}",
                current.display()
            )
        })? {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                stack.push(entry_path);
                continue;
            }
            match entry_path.extension().and_then(|value| value.to_str()) {
                Some("milk") | Some("milk2") => found.push(entry_path),
                _ => {}
            }
        }
    }
    if found.is_empty() {
        Err(format!(
            "no .milk/.milk2 sources found in {}",
            path.display()
        ))
    } else {
        found.sort();
        Ok(found)
    }
}

pub fn gather_pack_presets(path: &Path) -> Result<Vec<PresetInput>, String> {
    let (manifest_path, manifest) =
        load_rustymilk_pack_manifest(path).map_err(|error| format!("{error}"))?;
    let base = manifest_path
        .parent()
        .map_or_else(|| Path::new("."), |value| value);
    let mut presets = Vec::with_capacity(manifest.presets.len());
    for preset in manifest.presets {
        let source_path = base.join(preset.file);
        let source_label = if preset.title.trim().is_empty() {
            if preset.id.trim().is_empty() {
                source_path.to_string_lossy().into_owned()
            } else {
                preset.id
            }
        } else {
            preset.title
        };
        if !source_path.is_file() {
            return Err(format!(
                "pack preset '{}' is missing: {}",
                source_label,
                source_path.display()
            ));
        }
        presets.push(PresetInput {
            source_path,
            source_label,
        });
    }
    if presets.is_empty() {
        Err(format!(
            "pack manifest has no presets: {}",
            manifest_path.display()
        ))
    } else {
        Ok(presets)
    }
}

pub fn gather_pack_plugins(path: &Path) -> Result<Vec<PackPluginInput>, String> {
    let (manifest_path, manifest) =
        load_rustymilk_pack_manifest(path).map_err(|error| format!("{error}"))?;
    let base = manifest_path
        .parent()
        .map_or_else(|| Path::new("."), |value| value);
    let mut plugins = Vec::with_capacity(manifest.plugins.len());
    for plugin in manifest.plugins {
        let source_path = base.join(&plugin.entry);
        if !source_path.is_file() {
            return Err(format!(
                "pack plugin '{}' is missing: {}",
                plugin.id,
                source_path.display()
            ));
        }
        let plugin =
            PackPluginInput::with_payload(plugin.id, plugin.kind, plugin.entry, source_path)?;
        plugins.push(plugin);
    }
    Ok(plugins)
}

pub fn collect_pack_plugins(pack_inputs: &[PathBuf]) -> Result<Vec<PackPluginInput>, String> {
    let mut plugins = Vec::new();
    for pack_input in pack_inputs {
        let mut pack_plugins = gather_pack_plugins(pack_input)?;
        plugins.append(&mut pack_plugins);
    }
    if plugins.is_empty() {
        return Ok(Vec::new());
    }
    let mut seen = BTreeSet::new();
    plugins.retain(|plugin| seen.insert(plugin.source_path.clone()));
    plugins.sort_by(|a, b| a.source_path.cmp(&b.source_path));
    Ok(plugins)
}

pub fn collect_preset_inputs(
    preset_inputs: &[PathBuf],
    pack_inputs: &[PathBuf],
) -> Result<Vec<PresetInput>, String> {
    let mut selected_presets = Vec::new();
    for preset_input in preset_inputs {
        let candidates = gather_preset_path_candidates(preset_input)?;
        for path in candidates {
            let label = path
                .file_name()
                .and_then(|value| value.to_str())
                .map(ToString::to_string)
                .unwrap_or_else(|| path.to_string_lossy().into_owned())
                .to_string();
            selected_presets.push(PresetInput {
                source_path: path,
                source_label: label,
            });
        }
    }

    for pack_input in pack_inputs {
        let mut pack_presets = gather_pack_presets(pack_input)?;
        selected_presets.append(&mut pack_presets);
    }

    if selected_presets.is_empty() {
        return Err("no preset sources were selected".to_string());
    }

    let mut seen = BTreeSet::new();
    selected_presets.retain(|preset| seen.insert(preset.source_path.clone()));
    selected_presets.sort_by(|a, b| a.source_path.cmp(&b.source_path));
    Ok(selected_presets)
}

pub fn write_pack_plugin_report(
    destination: &Path,
    mode: &str,
    pack_inputs: &[PathBuf],
    pack_plugins: &[PackPluginInput],
    collection_error: Option<&str>,
) -> Result<(), String> {
    let generated_millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("system clock regression: {error}"))?
        .as_millis();
    let report = serde_json::json!({
        "schemaVersion": 1,
        "mode": mode,
        "generatedUnixMillis": generated_millis,
        "packInputs": pack_inputs
            .iter()
            .map(|path| path.to_string_lossy())
            .collect::<Vec<_>>(),
        "collectionError": collection_error,
        "packPlugins": {
            "count": pack_plugins.len(),
            "entries": pack_plugins
                .iter()
                .map(|plugin| serde_json::json!({
                    "id": plugin.id,
                    "kind": plugin.kind,
                    "entry": plugin.entry,
                    "sourcePath": plugin.source_path.to_string_lossy(),
                    "payloadKeyCount": plugin.payload.as_object().map(|payload| payload.len()).unwrap_or(0),
                    "payload": if plugin.payload.is_object() {
                        Some(&plugin.payload)
                    } else {
                        None
                    },
                }))
                .collect::<Vec<_>>(),
        },
    });

    let json = serde_json::to_string_pretty(&report)
        .map_err(|error| format!("unable to serialize plugin report: {error}"))?;
    fs::write(destination, json).map_err(|error| {
        format!(
            "unable to write plugin report '{}': {error}",
            destination.display()
        )
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_test_root(test_name: &str) -> PathBuf {
        let pid = std::process::id();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|value| value.subsec_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!(
            "rustymilk-desktop-cli-{}-{}-{}",
            pid, test_name, nanos
        ));
        std::fs::remove_dir_all(&root).ok();
        std::fs::create_dir_all(&root).expect("create test root");
        root
    }

    #[test]
    fn parses_positive_numbers_and_floats() {
        assert_eq!(parse_positive_usize("24").unwrap(), 24);
        assert!(parse_positive_usize("0").is_err());
        assert!(parse_positive_usize("-3").is_err());

        assert!((parse_positive_f64("1.5").unwrap() - 1.5).abs() < f64::EPSILON);
        assert!(parse_positive_f64("0").is_err());
        assert!(parse_positive_f64("bad").is_err());

        assert_eq!(parse_non_negative_f64("0").unwrap(), 0.0);
        assert!(parse_non_negative_f64("-0.1").is_err());
    }

    #[test]
    fn collects_paths_from_files_directories_and_pack_manifests() {
        let root = make_test_root("collects_paths");
        let loose = root.join("loose");
        let pack_dir = root.join("pack");

        let manual_a = loose.join("warm.milk");
        let ignored = loose.join("readme.txt");
        let nested = loose.join("nested");
        let nested_milk = nested.join("cold.milk2");

        std::fs::create_dir_all(&nested).unwrap();
        std::fs::create_dir_all(&pack_dir.join("presets")).unwrap();

        std::fs::write(&manual_a, "name=warm\n").unwrap();
        std::fs::write(&ignored, "ignore\n").unwrap();
        std::fs::write(&nested_milk, "name=cold\n").unwrap();

        let manual = collect_preset_inputs(&[manual_a.clone()], &[]).unwrap();
        assert_eq!(manual.len(), 1);
        assert_eq!(manual[0].source_path, manual_a);

        assert!(collect_preset_inputs(&[ignored.clone()], &[]).is_err());

        let walk = collect_preset_inputs(&[loose.clone()], &[]).unwrap();
        assert_eq!(walk.len(), 2);
        assert!(walk.iter().any(|entry| entry.source_path == manual_a));
        assert!(walk.iter().any(|entry| entry.source_path == nested_milk));

        let preset_path = "presets/studio.milk";
        std::fs::write(pack_dir.join(preset_path), "name=studio\n").unwrap();
        let manifest_path = pack_dir.join("manifest.json");
        let manifest = format!(
            "{{\n  \"schemaVersion\": 1,\n  \"id\": \"test-pack\",\n  \"name\": \"Test Pack\",\n  \"version\": \"0.1.0\",\n  \"author\": \"rustymilk\",\n  \"description\": \"\",\n  \"license\": \"CC0-1.0\",\n  \"requiredRustyMilkVersion\": \"0.1.0\",\n  \"presets\": [{{\"id\": \"studio\", \"title\": \"Studio\", \"file\": \"{preset_path}\"}}]\n}}"
        );
        std::fs::write(&manifest_path, manifest).unwrap();

        let from_pack = collect_preset_inputs(&[], &[pack_dir.clone()]).unwrap();
        assert_eq!(from_pack.len(), 1);
        assert_eq!(from_pack[0].source_path.file_name().unwrap(), "studio.milk");

        let mixed = collect_preset_inputs(&[manual_a.clone()], &[pack_dir]).unwrap();
        assert_eq!(mixed.len(), 2);
        let paths: Vec<PathBuf> = mixed.iter().map(|item| item.source_path.clone()).collect();
        assert!(paths.contains(&manual_a));
        assert!(paths.iter().any(|path| path.ends_with(preset_path)));

        let deduped = collect_preset_inputs(&[manual_a.clone(), manual_a], &[]).unwrap();
        assert_eq!(deduped.len(), 1);

        assert!(collect_preset_inputs(&[], &[]).is_err());
    }

    #[test]
    fn parses_pack_plugins_and_collects_pack_artifacts() {
        let root = make_test_root("collects_pack_plugins");
        let pack_dir = root.join("plugin-pack");
        std::fs::create_dir_all(&pack_dir.join("plugins")).unwrap();
        std::fs::create_dir_all(&pack_dir.join("presets")).unwrap();

        let preset_path = "presets/studio.milk";
        let plugin_path = "plugins/playlist.json";
        std::fs::write(pack_dir.join(preset_path), "name=studio\n").unwrap();
        std::fs::write(
            pack_dir.join(plugin_path),
            r#"{ "playlist": ["studio"], "tags": ["test"] }"#,
        )
        .unwrap();
        let manifest_path = pack_dir.join("manifest.json");
        let manifest = format!(
            "{{\n  \"schemaVersion\": 1,\n  \"id\": \"plugin-pack\",\n  \"name\": \"Plugin Pack\",\n  \"version\": \"0.1.0\",\n  \"author\": \"rustymilk\",\n  \"description\": \"\",\n  \"license\": \"CC0-1.0\",\n  \"requiredRustyMilkVersion\": \"0.1.0\",\n  \"presets\": [{{\"id\": \"studio\", \"title\": \"Studio\", \"file\": \"{preset_path}\"}}],\n  \"plugins\": [{{\"id\": \"default-playlist\", \"kind\": \"data\", \"entry\": \"{plugin_path}\"}}]\n}}"
        );
        std::fs::write(&manifest_path, manifest).unwrap();

        let pack_plugins = collect_pack_plugins(&[pack_dir.clone()]).unwrap();
        assert_eq!(pack_plugins.len(), 1);
        assert_eq!(pack_plugins[0].id, "default-playlist");
        assert_eq!(pack_plugins[0].kind, "data");
        assert_eq!(pack_plugins[0].entry, plugin_path);
        assert_eq!(
            pack_plugins[0].payload["playlist"],
            serde_json::json!(["studio"])
        );

        let presets = collect_preset_inputs(&[], &[pack_dir.clone()]).unwrap();
        assert_eq!(presets.len(), 1);
        let presets_again = collect_preset_inputs(&[], &[pack_dir]).unwrap();
        assert_eq!(presets_again.len(), 1);
    }

    #[test]
    fn writes_pack_plugin_report() {
        let root = make_test_root("writes_pack_plugin_report");
        let pack_dir = root.join("plugin-pack");
        let payload_path = pack_dir.join("plugins/playlist.json");
        std::fs::create_dir_all(payload_path.parent().unwrap()).unwrap();
        std::fs::write(&payload_path, r#"{ "items": ["A", "B", "C"] }"#).unwrap();

        let plugin = PackPluginInput::with_payload(
            "playlist".to_string(),
            "data".to_string(),
            "plugins/playlist.json".to_string(),
            payload_path.clone(),
        )
        .expect("build payload plugin");
        let destination = root.join("pack-plugin-report.json");

        write_pack_plugin_report(
            &destination,
            "player",
            std::slice::from_ref(&pack_dir),
            std::slice::from_ref(&plugin),
            None,
        )
        .expect("write report");

        let saved = std::fs::read_to_string(&destination).expect("read report");
        assert!(saved.contains("\"mode\": \"player\""));
        assert!(saved.contains("\"playlist\""));
        assert!(saved.contains("\"count\": 1"));
    }
}

use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use rustymilk_core::{build_rustymilk_compatibility_entry, rustymilk_preset_name};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub struct RustyMilkPackManifest {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub license: String,
    pub required_rustymilk_version: String,
    pub source_urls: Vec<String>,
    pub presets: Vec<RustyMilkPackPreset>,
    pub textures: Vec<RustyMilkPackTexture>,
    pub fragments: Vec<RustyMilkPackFragment>,
    pub plugins: Vec<RustyMilkPackPlugin>,
    pub playlist: Vec<String>,
    pub automation_defaults: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RustyMilkPackPreset {
    pub id: String,
    pub title: String,
    pub file: String,
    pub source_format: String,
    pub tags: Vec<String>,
    pub thumbnail: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RustyMilkPackTexture {
    pub id: String,
    pub file: String,
    pub aliases: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RustyMilkPackFragment {
    pub id: String,
    pub kind: String,
    pub file: String,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RustyMilkPackPlugin {
    pub id: String,
    pub kind: String,
    pub entry: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RustyMilkPackPresetReport {
    pub id: String,
    pub file: String,
    pub title: String,
    pub source_format: String,
    pub format: String,
    pub preset_count: usize,
    pub supported: bool,
    pub webgpu_supported: bool,
    pub missing: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RustyMilkPackValidationReport {
    pub manifest_path: String,
    pub pack_id: String,
    pub pack_name: String,
    pub version: String,
    pub valid: bool,
    pub preset_count: usize,
    pub texture_count: usize,
    pub fragment_count: usize,
    pub plugin_count: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub presets: Vec<RustyMilkPackPresetReport>,
}

impl RustyMilkPackValidationReport {
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "manifestPath": self.manifest_path,
            "packId": self.pack_id,
            "packName": self.pack_name,
            "version": self.version,
            "valid": self.valid,
            "presetCount": self.preset_count,
            "textureCount": self.texture_count,
            "fragmentCount": self.fragment_count,
            "pluginCount": self.plugin_count,
            "errors": self.errors,
            "warnings": self.warnings,
            "presets": self.presets.iter().map(|preset| serde_json::json!({
                "id": preset.id,
                "file": preset.file,
                "title": preset.title,
                "sourceFormat": preset.source_format,
                "format": preset.format,
                "presetCount": preset.preset_count,
                "supported": preset.supported,
                "webGpuSupported": preset.webgpu_supported,
                "missing": preset.missing,
            })).collect::<Vec<_>>(),
        })
    }
}

pub fn parse_rustymilk_pack_manifest(source: &str) -> Result<RustyMilkPackManifest, Vec<String>> {
    let value = serde_json::from_str::<Value>(source)
        .map_err(|error| vec![format!("manifest is not valid JSON: {error}")])?;
    let Some(object) = value.as_object() else {
        return Err(vec!["manifest root must be a JSON object".to_string()]);
    };

    let mut errors = Vec::new();
    let schema_version = optional_u64(object.get("schemaVersion"))
        .or_else(|| optional_u64(object.get("schema_version")))
        .unwrap_or(1) as u32;
    let id = required_string(object.get("id"), "id", &mut errors);
    let name = required_string(object.get("name"), "name", &mut errors);
    let version = required_string(object.get("version"), "version", &mut errors);
    let author = optional_string(object.get("author")).unwrap_or_default();
    let description = optional_string(object.get("description")).unwrap_or_default();
    let license = optional_string(object.get("license")).unwrap_or_default();
    let required_rustymilk_version = optional_string(object.get("requiredRustyMilkVersion"))
        .or_else(|| optional_string(object.get("required_rustymilk_version")))
        .unwrap_or_default();
    let source_urls = string_array(object.get("sourceUrls"))
        .or_else(|| string_array(object.get("source_urls")))
        .unwrap_or_default();

    let presets = pack_array(
        object.get("presets"),
        "presets",
        &mut errors,
        parse_pack_preset,
    );
    let textures = pack_array(
        object.get("textures"),
        "textures",
        &mut errors,
        parse_pack_texture,
    );
    let fragments = pack_array(
        object.get("fragments"),
        "fragments",
        &mut errors,
        parse_pack_fragment,
    );
    let plugins = pack_array(
        object.get("plugins"),
        "plugins",
        &mut errors,
        parse_pack_plugin,
    );
    let playlist = string_array(object.get("playlist")).unwrap_or_default();
    let automation_defaults = object
        .get("automationDefaults")
        .or_else(|| object.get("automation_defaults"))
        .cloned()
        .unwrap_or(Value::Null);

    if errors.is_empty() {
        Ok(RustyMilkPackManifest {
            schema_version,
            id,
            name,
            version,
            author,
            description,
            license,
            required_rustymilk_version,
            source_urls,
            presets,
            textures,
            fragments,
            plugins,
            playlist,
            automation_defaults,
        })
    } else {
        Err(errors)
    }
}

pub fn load_rustymilk_pack_manifest(
    pack_path: impl AsRef<Path>,
) -> Result<(PathBuf, RustyMilkPackManifest), String> {
    let manifest_path = rustymilk_pack_manifest_path(pack_path.as_ref());
    let source = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("failed to read {}: {error}", manifest_path.display()))?;
    parse_rustymilk_pack_manifest(&source)
        .map(|manifest| (manifest_path, manifest))
        .map_err(|errors| errors.join("; "))
}

pub fn validate_rustymilk_pack_dir(pack_path: impl AsRef<Path>) -> RustyMilkPackValidationReport {
    let pack_path = pack_path.as_ref();
    let manifest_path = rustymilk_pack_manifest_path(pack_path);
    let manifest_path_label = manifest_path.display().to_string();
    let source = match fs::read_to_string(&manifest_path) {
        Ok(source) => source,
        Err(error) => {
            return RustyMilkPackValidationReport {
                manifest_path: manifest_path_label,
                pack_id: String::new(),
                pack_name: String::new(),
                version: String::new(),
                valid: false,
                preset_count: 0,
                texture_count: 0,
                fragment_count: 0,
                plugin_count: 0,
                errors: vec![format!("failed to read manifest: {error}")],
                warnings: Vec::new(),
                presets: Vec::new(),
            };
        }
    };
    validate_rustymilk_pack_source(pack_path, &manifest_path_label, &source)
}

pub fn validate_rustymilk_pack_source(
    pack_path: impl AsRef<Path>,
    manifest_path_label: &str,
    source: &str,
) -> RustyMilkPackValidationReport {
    let pack_path = pack_path.as_ref();
    let manifest = match parse_rustymilk_pack_manifest(source) {
        Ok(manifest) => manifest,
        Err(errors) => {
            return RustyMilkPackValidationReport {
                manifest_path: manifest_path_label.to_string(),
                pack_id: String::new(),
                pack_name: String::new(),
                version: String::new(),
                valid: false,
                preset_count: 0,
                texture_count: 0,
                fragment_count: 0,
                plugin_count: 0,
                errors,
                warnings: Vec::new(),
                presets: Vec::new(),
            };
        }
    };

    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    validate_id("id", &manifest.id, &mut errors);
    if manifest.schema_version != 1 {
        errors.push(format!(
            "unsupported schemaVersion {}; expected 1",
            manifest.schema_version
        ));
    }
    if manifest.presets.is_empty() {
        warnings.push("pack contains no presets".to_string());
    }

    let mut preset_reports = Vec::new();
    for preset in &manifest.presets {
        validate_id(
            &format!("presets.{}.id", preset.id),
            &preset.id,
            &mut errors,
        );
        validate_relative_pack_path(&preset.file, &format!("preset {}", preset.id), &mut errors);
        let source_format = normalize_source_format(&preset.source_format, &preset.file);
        if preset.title.trim().is_empty() {
            warnings.push(format!("preset {} has no title", preset.id));
        }
        let preset_path = pack_path.join(&preset.file);
        match fs::read_to_string(&preset_path) {
            Ok(preset_source) => {
                if source_format == "milk" || source_format == "milk2" {
                    let entry = build_rustymilk_compatibility_entry(
                        &preset.id,
                        &preset.file,
                        &preset_source,
                        source_format == "milk2",
                    );
                    preset_reports.push(RustyMilkPackPresetReport {
                        id: preset.id.clone(),
                        file: preset.file.clone(),
                        title: if preset.title.trim().is_empty() {
                            rustymilk_preset_name(&preset_source)
                        } else {
                            preset.title.clone()
                        },
                        source_format: source_format.clone(),
                        format: entry.format,
                        preset_count: entry.preset_count,
                        supported: entry.supported,
                        webgpu_supported: entry.webgpu_supported,
                        missing: false,
                    });
                } else if source_format == "butterchurn-json" {
                    if serde_json::from_str::<Value>(&preset_source).is_err() {
                        errors.push(format!("preset {} is not valid JSON", preset.file));
                    }
                    preset_reports.push(RustyMilkPackPresetReport {
                        id: preset.id.clone(),
                        file: preset.file.clone(),
                        title: preset.title.clone(),
                        source_format: source_format.clone(),
                        format: source_format,
                        preset_count: 1,
                        supported: false,
                        webgpu_supported: false,
                        missing: false,
                    });
                } else {
                    errors.push(format!(
                        "preset {} uses unsupported sourceFormat {}",
                        preset.file, source_format
                    ));
                    preset_reports.push(RustyMilkPackPresetReport {
                        id: preset.id.clone(),
                        file: preset.file.clone(),
                        title: preset.title.clone(),
                        source_format: source_format.clone(),
                        format: source_format,
                        preset_count: 0,
                        supported: false,
                        webgpu_supported: false,
                        missing: false,
                    });
                }
            }
            Err(error) => {
                errors.push(format!("preset {} failed to read: {error}", preset.file));
                preset_reports.push(RustyMilkPackPresetReport {
                    id: preset.id.clone(),
                    file: preset.file.clone(),
                    title: preset.title.clone(),
                    source_format,
                    format: String::new(),
                    preset_count: 0,
                    supported: false,
                    webgpu_supported: false,
                    missing: true,
                });
            }
        }
        if !preset.thumbnail.trim().is_empty() {
            validate_relative_pack_path(
                &preset.thumbnail,
                &format!("thumbnail {}", preset.thumbnail),
                &mut errors,
            );
            if !pack_path.join(&preset.thumbnail).is_file() {
                warnings.push(format!("thumbnail {} is missing", preset.thumbnail));
            }
        }
    }

    for texture in &manifest.textures {
        validate_id(
            &format!("textures.{}.id", texture.id),
            &texture.id,
            &mut errors,
        );
        validate_relative_pack_path(
            &texture.file,
            &format!("texture {}", texture.id),
            &mut errors,
        );
        if !pack_path.join(&texture.file).is_file() {
            warnings.push(format!("texture {} is missing", texture.file));
        }
    }
    for fragment in &manifest.fragments {
        validate_id(
            &format!("fragments.{}.id", fragment.id),
            &fragment.id,
            &mut errors,
        );
        validate_relative_pack_path(
            &fragment.file,
            &format!("fragment {}", fragment.id),
            &mut errors,
        );
        if !pack_path.join(&fragment.file).is_file() {
            warnings.push(format!("fragment {} is missing", fragment.file));
        }
    }
    for plugin in &manifest.plugins {
        validate_id(
            &format!("plugins.{}.id", plugin.id),
            &plugin.id,
            &mut errors,
        );
        validate_relative_pack_path(&plugin.entry, &format!("plugin {}", plugin.id), &mut errors);
        if !pack_path.join(&plugin.entry).is_file() {
            warnings.push(format!("plugin {} is missing", plugin.entry));
        }
    }

    RustyMilkPackValidationReport {
        manifest_path: manifest_path_label.to_string(),
        pack_id: manifest.id,
        pack_name: manifest.name,
        version: manifest.version,
        valid: errors.is_empty(),
        preset_count: preset_reports.len(),
        texture_count: manifest.textures.len(),
        fragment_count: manifest.fragments.len(),
        plugin_count: manifest.plugins.len(),
        errors,
        warnings,
        presets: preset_reports,
    }
}

pub fn rustymilk_pack_manifest_path(pack_path: &Path) -> PathBuf {
    if pack_path.is_dir() {
        pack_path.join("manifest.json")
    } else {
        pack_path.to_path_buf()
    }
}

fn required_string(value: Option<&Value>, field: &str, errors: &mut Vec<String>) -> String {
    match optional_string(value) {
        Some(value) if !value.trim().is_empty() => value,
        _ => {
            errors.push(format!("manifest field {field} must be a non-empty string"));
            String::new()
        }
    }
}

fn optional_string(value: Option<&Value>) -> Option<String> {
    value.and_then(Value::as_str).map(ToString::to_string)
}

fn optional_u64(value: Option<&Value>) -> Option<u64> {
    value.and_then(Value::as_u64)
}

fn string_array(value: Option<&Value>) -> Option<Vec<String>> {
    value.and_then(Value::as_array).map(|values| {
        values
            .iter()
            .filter_map(Value::as_str)
            .map(ToString::to_string)
            .collect()
    })
}

fn pack_array<T>(
    value: Option<&Value>,
    field: &str,
    errors: &mut Vec<String>,
    parse: fn(&Value, usize, &str, &mut Vec<String>) -> T,
) -> Vec<T> {
    let Some(value) = value else {
        return Vec::new();
    };
    let Some(values) = value.as_array() else {
        errors.push(format!("manifest field {field} must be an array"));
        return Vec::new();
    };
    values
        .iter()
        .enumerate()
        .map(|(index, value)| parse(value, index, field, errors))
        .collect()
}

fn parse_pack_preset(
    value: &Value,
    index: usize,
    field: &str,
    errors: &mut Vec<String>,
) -> RustyMilkPackPreset {
    if !value.is_object() {
        errors.push(format!("{field}[{index}] must be an object"));
    }
    RustyMilkPackPreset {
        id: required_string(value.get("id"), &format!("{field}[{index}].id"), errors),
        title: optional_string(value.get("title")).unwrap_or_default(),
        file: required_string(value.get("file"), &format!("{field}[{index}].file"), errors),
        source_format: optional_string(value.get("sourceFormat"))
            .or_else(|| optional_string(value.get("source_format")))
            .or_else(|| optional_string(value.get("format")))
            .unwrap_or_default(),
        tags: string_array(value.get("tags")).unwrap_or_default(),
        thumbnail: optional_string(value.get("thumbnail")).unwrap_or_default(),
    }
}

fn parse_pack_texture(
    value: &Value,
    index: usize,
    field: &str,
    errors: &mut Vec<String>,
) -> RustyMilkPackTexture {
    if !value.is_object() {
        errors.push(format!("{field}[{index}] must be an object"));
    }
    RustyMilkPackTexture {
        id: required_string(value.get("id"), &format!("{field}[{index}].id"), errors),
        file: required_string(value.get("file"), &format!("{field}[{index}].file"), errors),
        aliases: string_array(value.get("aliases")).unwrap_or_default(),
    }
}

fn parse_pack_fragment(
    value: &Value,
    index: usize,
    field: &str,
    errors: &mut Vec<String>,
) -> RustyMilkPackFragment {
    if !value.is_object() {
        errors.push(format!("{field}[{index}] must be an object"));
    }
    RustyMilkPackFragment {
        id: required_string(value.get("id"), &format!("{field}[{index}].id"), errors),
        kind: optional_string(value.get("kind")).unwrap_or_else(|| "preset".to_string()),
        file: required_string(value.get("file"), &format!("{field}[{index}].file"), errors),
        tags: string_array(value.get("tags")).unwrap_or_default(),
    }
}

fn parse_pack_plugin(
    value: &Value,
    index: usize,
    field: &str,
    errors: &mut Vec<String>,
) -> RustyMilkPackPlugin {
    if !value.is_object() {
        errors.push(format!("{field}[{index}] must be an object"));
    }
    RustyMilkPackPlugin {
        id: required_string(value.get("id"), &format!("{field}[{index}].id"), errors),
        kind: optional_string(value.get("kind")).unwrap_or_else(|| "data".to_string()),
        entry: required_string(
            value.get("entry"),
            &format!("{field}[{index}].entry"),
            errors,
        ),
    }
}

fn validate_id(field: &str, id: &str, errors: &mut Vec<String>) {
    if id.trim().is_empty() {
        errors.push(format!("{field} must not be empty"));
        return;
    }
    if !id
        .chars()
        .all(|char| char.is_ascii_alphanumeric() || char == '-' || char == '_' || char == '.')
    {
        errors.push(format!(
            "{field} contains invalid characters; use letters, numbers, '.', '-', or '_'"
        ));
    }
}

fn validate_relative_pack_path(path: &str, label: &str, errors: &mut Vec<String>) {
    if path.trim().is_empty() {
        errors.push(format!("{label} path must not be empty"));
        return;
    }
    let path = Path::new(path);
    if path.is_absolute() {
        errors.push(format!("{label} path must be relative"));
        return;
    }
    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        errors.push(format!("{label} path must stay inside the pack"));
    }
}

fn normalize_source_format(source_format: &str, file: &str) -> String {
    let normalized = source_format.trim().to_ascii_lowercase().replace('_', "-");
    if !normalized.is_empty() {
        return normalized;
    }
    let lower_file = file.to_ascii_lowercase();
    if lower_file.ends_with(".milk2") {
        "milk2".to_string()
    } else if lower_file.ends_with(".json") {
        "butterchurn-json".to_string()
    } else {
        "milk".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "rustymilk-pack-test-{}-{name}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    fn manifest_source() -> &'static str {
        r#"{
          "schemaVersion": 1,
          "id": "sample-pack",
          "name": "Sample Pack",
          "version": "0.1.0",
          "author": "RustyMilk",
          "license": "CC0-1.0",
          "requiredRustyMilkVersion": "0.1.0",
          "sourceUrls": ["https://example.invalid/rustymilk"],
              "presets": [
                {
                  "id": "warm-lines",
                  "title": "Warm Lines",
                  "file": "presets/warm-lines.milk",
                  "sourceFormat": "milk",
                  "tags": ["lines", "fixture"],
                  "thumbnail": "thumbnails/warm-lines.png"
                }
          ],
          "textures": [
            { "id": "noise", "file": "textures/noise.png", "aliases": ["noise_lq"] }
          ],
          "fragments": [
            { "id": "warm-shape", "kind": "shape", "file": "fragments/warm-shape.json" }
          ],
          "plugins": [
            { "id": "playlist-defaults", "kind": "data", "entry": "plugins/playlist.json" }
          ],
          "playlist": ["warm-lines"]
        }"#
    }

    fn preset_source() -> &'static str {
        "name=Warm Lines\n\
         decay=0.9\n\
         wave_r=0.8\n\
         wave_g=0.35\n\
         wave_b=0.2\n\
         wave_a=0.9\n"
    }

    #[test]
    fn parses_manifest_metadata() {
        let manifest = parse_rustymilk_pack_manifest(manifest_source()).unwrap();
        assert_eq!(manifest.id, "sample-pack");
        assert_eq!(manifest.presets[0].file, "presets/warm-lines.milk");
        assert_eq!(manifest.presets[0].source_format, "milk");
        assert_eq!(manifest.textures[0].aliases, vec!["noise_lq"]);
    }

    #[test]
    fn validates_pack_directory_and_presets() {
        let dir = temp_dir("valid");
        fs::create_dir_all(dir.join("presets")).unwrap();
        fs::create_dir_all(dir.join("textures")).unwrap();
        fs::create_dir_all(dir.join("fragments")).unwrap();
        fs::create_dir_all(dir.join("plugins")).unwrap();
        fs::create_dir_all(dir.join("thumbnails")).unwrap();
        fs::write(dir.join("manifest.json"), manifest_source()).unwrap();
        fs::write(dir.join("presets/warm-lines.milk"), preset_source()).unwrap();
        fs::write(dir.join("textures/noise.png"), []).unwrap();
        fs::write(dir.join("fragments/warm-shape.json"), "{}").unwrap();
        fs::write(dir.join("plugins/playlist.json"), "{}").unwrap();

        let report = validate_rustymilk_pack_dir(&dir);
        let _ = fs::remove_dir_all(&dir);

        assert!(report.valid, "{:?}", report.errors);
        assert_eq!(report.preset_count, 1);
        assert_eq!(report.presets[0].title, "Warm Lines");
        assert!(report
            .warnings
            .iter()
            .any(|warning| warning.contains("thumbnail")));
    }

    #[test]
    fn rejects_paths_that_escape_pack() {
        let source = r#"{
          "id": "bad-pack",
          "name": "Bad Pack",
          "version": "0.1.0",
          "presets": [{ "id": "escape", "file": "../escape.milk" }]
        }"#;
        let report = validate_rustymilk_pack_source(".", "manifest.json", source);
        assert!(!report.valid);
        assert!(report
            .errors
            .iter()
            .any(|error| error.contains("must stay inside the pack")));
    }
}

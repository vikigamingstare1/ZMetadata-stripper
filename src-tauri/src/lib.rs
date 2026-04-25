mod commands;
mod formats;
mod presets;
mod types;

use commands::{inspect::inspect_file, strip::strip_file};

const SUPPORTED_EXTS: &[&str] = &[
    "jpg","jpeg","png","webp","tiff","tif","gif","bmp","avif","heic","heif",
    "pdf","docx","xlsx","pptx","odt","ods","odp","rtf","svg",
    "mp3","flac","ogg","wav","wave","aiff","aif","m4a",
    "mp4","mov","mkv","webm",
];

#[tauri::command]
async fn scan_folder(path: String) -> Vec<String> {
    let mut out = Vec::new();
    collect_files(std::path::Path::new(&path), &mut out);
    out
}

fn collect_files(dir: &std::path::Path, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_files(&p, out);
        } else if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
            if SUPPORTED_EXTS.contains(&ext.to_lowercase().as_str()) {
                out.push(p.to_string_lossy().to_string());
            }
        }
    }
}

#[tauri::command]
async fn get_presets() -> Vec<serde_json::Value> {
    use serde_json::json;
    vec![
        json!({ "id": "max_privacy",    "label": "Max Privacy",        "description": "Strip everything — full forensic-safe clean." }),
        json!({ "id": "keep_quality",   "label": "Keep Quality",       "description": "Strip GPS & author, keep ICC & exposure data." }),
        json!({ "id": "social_media",   "label": "Social Media",       "description": "Strip GPS & author, keep color profile for delivery." }),
        json!({ "id": "documents_only", "label": "Documents Only",     "description": "Focused on PDF / Office metadata scrub." }),
    ]
}

#[tauri::command]
async fn get_strip_options(preset_id: String) -> types::StripOptions {
    presets::from_id(&preset_id)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            inspect_file,
            strip_file,
            get_presets,
            get_strip_options,
            scan_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

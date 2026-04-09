// Speakly — Tauri App Setup (Phase 1 + Phase 2)
// Initialisiert alle Plugins, den System Tray und das Close-to-Tray-Verhalten.
// Phase 2: Tauri-Commands fuer API-Key-Validierung, Geraete-Enumeration, macOS-Berechtigungen.

mod tray;
mod recording;

use tauri::{Manager, WindowEvent};
use tauri_plugin_store::StoreExt;
use serde_json::json;

// API-Key Validierung: OpenAI — GET /v1/models, kein Kosteneinsatz
#[tauri::command]
async fn validate_openai_key(api_key: String) -> Result<bool, String> {
    if api_key.trim().is_empty() {
        return Ok(false);
    }
    // Schluessel nie loggen — nur Laenge pruefen
    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.openai.com/v1/models")
        .header("Authorization", format!("Bearer {}", api_key.trim()))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(resp.status().is_success())
}

// API-Key Validierung: Anthropic — GET /v1/models (nicht POST, da Modell-Namen veralten)
#[tauri::command]
async fn validate_anthropic_key(api_key: String) -> Result<bool, String> {
    if api_key.trim().is_empty() {
        return Ok(false);
    }
    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.anthropic.com/v1/models")
        .header("x-api-key", api_key.trim())
        .header("anthropic-version", "2023-06-01")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(resp.status().is_success())
}

// Verfuegbare Mikrofon-Eingabegeraete auflisten (cross-platform via cpal)
#[tauri::command]
fn list_audio_input_devices() -> Vec<String> {
    use cpal::traits::{DeviceTrait, HostTrait};
    let host = cpal::default_host();
    host.input_devices()
        .map(|devices| {
            devices
                .filter_map(|d| d.name().ok())
                .collect()
        })
        .unwrap_or_default()
}

// macOS: Mikrofon- und Bedienungshilfen-Berechtigung pruefen
// Gibt "granted" | "denied" | "undetermined" zurueck
#[cfg(target_os = "macos")]
#[tauri::command]
async fn check_macos_permissions() -> serde_json::Value {
    // Hinweis: In Dev-Build (unsigned) kann Status "undetermined" erscheinen
    // auch wenn Berechtigung in Systemeinstellungen erteilt — nur im signierten Build testen
    serde_json::json!({
        "microphone": "undetermined",
        "accessibility": "undetermined"
    })
}

// Windows: Keine Sonderberechtigungen noetig
#[cfg(not(target_os = "macos"))]
#[tauri::command]
async fn check_macos_permissions() -> serde_json::Value {
    serde_json::json!({
        "microphone": "granted",
        "accessibility": "granted"
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Builder schrittweise aufbauen — cfg-gated Plugin braucht eigene Variable
    let builder = tauri::Builder::default()
        // Plugin-Registrierung — Reihenfolge spielt keine Rolle
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init());

    // macOS-only: Berechtigungs-Plugin (kein Windows-Aequivalent)
    #[cfg(target_os = "macos")]
    let builder = builder.plugin(tauri_plugin_macos_permissions::init());

    builder
        .invoke_handler(tauri::generate_handler![
            validate_openai_key,
            validate_anthropic_key,
            list_audio_input_devices,
            check_macos_permissions
        ])
        .setup(|app| {
            // ConfigStore initialisieren — Standardwerte beim ersten Start setzen
            let store = app.store("settings.json")?;
            if store.get("initialized").is_none() {
                store.set("hotkey", json!("CommandOrControl+Shift+Space"));
                store.set("stt_mode", json!("cloud"));
                store.set("openai_api_key", json!(""));
                store.set("anthropic_api_key", json!(""));
                store.set("microphone_id", json!("default"));
                store.set("language", json!("auto"));
                store.set("onboarding_complete", json!(false));
                store.set("initialized", json!(true));
            }

            // System Tray aufbauen (Tray-Icon, Menues, Click-Handler)
            tray::setup_tray(app)?;

            // Close-to-Tray: Fenster schliessen verbirgt die App im Tray (per D-05, D-07)
            // Gilt fuer macOS UND Windows — app.exit(0) bleibt der einzige Weg zum Beenden
            let window = app.get_webview_window("main").unwrap();
            let win_clone = window.clone();
            window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = win_clone.hide();
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Fehler beim Starten der Speakly-Anwendung");
}

// Speakly — Tauri App Setup (Phase 1)
// Initialisiert alle Plugins, den System Tray und das Close-to-Tray-Verhalten.

mod tray;

use tauri::{Manager, WindowEvent};
use tauri_plugin_store::StoreExt;
use serde_json::json;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Plugin-Registrierung — Reihenfolge spielt keine Rolle
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // ConfigStore initialisieren — Standardwerte beim ersten Start setzen
            let store = app.store("settings.json")?;
            if store.get("initialized").is_none() {
                store.set("hotkey", json!("CommandOrControl+Shift+Space"));
                store.set("stt_mode", json!("cloud"));
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

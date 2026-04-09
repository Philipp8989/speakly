// Speakly — Whisper-Modell Download (Phase 6, D-10 bis D-14)
// Laedt ggml-base.bin von HuggingFace mit Streaming-Progress-Events.
// Abbruch: Datei loeschen + stt_mode auf "cloud" setzen.
// Korruption: Dateigroesse pruefen, bei Abweichung loeschen und Fehler.

use tauri::Emitter;
use tauri_plugin_store::StoreExt;

/// URL und erwartete Groesse des ggml-base.bin Modells (per D-06, D-11)
pub const WHISPER_BASE_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin";
/// Erwartete Dateigroesse in Bytes: ~142 MB (per D-06)
pub const WHISPER_BASE_SIZE: u64 = 147_964_211;

/// Prueft ob das Modell bereits vollstaendig heruntergeladen ist (per D-10, D-14)
pub fn model_exists(app: &tauri::AppHandle) -> bool {
    if let Ok(path) = super::local::model_path(app) {
        if let Ok(meta) = std::fs::metadata(&path) {
            return meta.len() == WHISPER_BASE_SIZE;
        }
    }
    false
}

/// Laedt das ggml-base.bin Modell herunter (per D-11, D-12, D-13, D-14).
/// Emittiert "whisper_download_progress" Events: { percent: f32, downloaded_mb: f32, total_mb: f32 }
/// Emittiert "whisper_download_complete" bei Erfolg.
/// Emittiert "whisper_download_error" bei Fehler: { message: String }
/// Bei Abbruch (teilweiser Download) → Datei loeschen, stt_mode="cloud".
#[tauri::command]
pub async fn download_whisper_model(app: tauri::AppHandle) -> Result<(), String> {
    use reqwest::Client;
    use std::io::Write;
    use futures_util::StreamExt;

    let path = super::local::model_path(&app)?;

    // Zielverzeichnis anlegen falls nicht vorhanden
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    // Datei oeffnen fuer Streaming-Schreiben
    let mut file = std::fs::File::create(&path).map_err(|e| e.to_string())?;

    let client = Client::new();
    let resp = client.get(WHISPER_BASE_URL)
        .timeout(std::time::Duration::from_secs(300)) // 5 Min Timeout fuer ~142MB
        .send()
        .await
        .map_err(|e| {
            let _ = std::fs::remove_file(&path);
            format!("Netzwerkfehler beim Download: {e}")
        })?;

    if !resp.status().is_success() {
        let _ = std::fs::remove_file(&path);
        return Err(format!("Download fehlgeschlagen: HTTP {}", resp.status()));
    }

    let total = resp.content_length().unwrap_or(WHISPER_BASE_SIZE);
    let mut downloaded: u64 = 0;

    // Streaming: Chunks lesen und in Datei schreiben (D-12: Progress-Events)
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| {
            let _ = std::fs::remove_file(&path);
            format!("Fehler beim Lesen des Downloads: {e}")
        })?;

        file.write_all(&chunk).map_err(|e| {
            let _ = std::fs::remove_file(&path);
            format!("Fehler beim Schreiben: {e}")
        })?;

        downloaded += chunk.len() as u64;
        let percent = (downloaded as f32 / total as f32 * 100.0).min(100.0);
        let downloaded_mb = downloaded as f32 / 1_048_576.0;
        let total_mb = total as f32 / 1_048_576.0;

        // Progress-Event alle ~1MB oder immer (Frontend drosselt selbst)
        let _ = app.emit("whisper_download_progress", serde_json::json!({
            "percent": percent,
            "downloaded_mb": downloaded_mb,
            "total_mb": total_mb
        }));
    }

    drop(file);

    // Dateigroesse pruefen (D-14: Korruption erkennen / T-06-01)
    let actual_size = std::fs::metadata(&path)
        .map(|m| m.len())
        .unwrap_or(0);

    if actual_size != WHISPER_BASE_SIZE {
        let _ = std::fs::remove_file(&path);
        let _ = app.emit("whisper_download_error", serde_json::json!({
            "message": format!("Modell-Datei beschaedigt (Groesse: {} statt {}). Bitte erneut versuchen.",
                actual_size, WHISPER_BASE_SIZE)
        }));
        return Err("Modell-Datei beschaedigt — Download unvollstaendig".into());
    }

    let _ = app.emit("whisper_download_complete", serde_json::json!({ "success": true }));
    Ok(())
}

/// Bricht den laufenden Download ab: Datei loeschen + stt_mode="cloud" setzen (per D-13).
/// Wird vom Frontend aufgerufen wenn User "Abbrechen" klickt.
#[tauri::command]
pub async fn cancel_whisper_download(app: tauri::AppHandle) -> Result<(), String> {
    // Datei loeschen falls vorhanden (teilweiser Download)
    if let Ok(path) = super::local::model_path(&app) {
        let _ = std::fs::remove_file(&path);
    }

    // stt_mode auf "cloud" zuruecksetzen (D-13)
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set("stt_mode", serde_json::json!("cloud"));

    let _ = app.emit("whisper_download_cancelled", serde_json::json!({}));
    Ok(())
}

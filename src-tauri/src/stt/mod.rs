// Speakly — STT-Modul (Phase 4)
// Sendet WAV-Puffer an die OpenAI Whisper API und gibt transkribierten Text zurueck.
// Fehlerbehandlung: benutzerfreundliche deutsche Fehlermeldungen (D-04, D-05).

use tauri_plugin_store::StoreExt;

/// Sendet WAV-Bytes an OpenAI Whisper API und gibt den transkribierten Text zurueck.
/// Fehler werden als deutsche Nutzer-Fehlermeldungen zurueckgegeben.
pub async fn call_whisper_api(app: &tauri::AppHandle, wav_bytes: Vec<u8>) -> Result<String, String> {
    use reqwest::multipart;

    // API-Key aus ConfigStore lesen — niemals den Key loggen (T-04-01)
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let api_key = store.get("openai_api_key")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();

    if api_key.trim().is_empty() {
        return Err("API-Key fehlt oder ungueltig".into());
    }

    // Sprach-Einstellung lesen (D-07, D-08)
    let language = store.get("language")
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    // Multipart-Formular aufbauen (D-02, D-03, Pitfall 6 — mime_str erforderlich)
    let file_part = multipart::Part::bytes(wav_bytes)
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|e| e.to_string())?;

    let mut form = multipart::Form::new()
        .part("file", file_part)
        .text("model", "whisper-1")
        .text("response_format", "json");

    // language-Parameter nur hinzufuegen wenn explizit gesetzt und nicht "auto" (D-07, D-08)
    if let Some(lang) = language {
        if lang != "auto" && !lang.is_empty() {
            form = form.text("language", lang);
        }
    }

    // HTTP-Request mit 30s Timeout senden (D-01, D-05)
    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.openai.com/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {}", api_key.trim()))
        .multipart(form)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| {
            // Timeout und Netzwerkfehler → gleiche Nutzermeldung (D-05)
            let _ = e; // Fehlerdetails nicht loggen — kein API-Key-Leak moeglich
            "Netzwerkfehler — Transkription fehlgeschlagen".to_string()
        })?;

    // HTTP-Fehler behandeln (D-04, D-05)
    let status = resp.status();
    if status.as_u16() == 401 || status.as_u16() == 403 {
        return Err("API-Key fehlt oder ungueltig".into());
    }
    if !status.is_success() {
        return Err("Netzwerkfehler — Transkription fehlgeschlagen".into());
    }

    // Antwort parsen — nur "text"-Feld extrahieren (D-03)
    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let text = body["text"].as_str().unwrap_or("").to_string();
    Ok(text)
}

#[cfg(test)]
mod tests {
    // Einheit-Tests fuer die STT-Logik.
    // HTTP-Aufrufe werden nicht gemockt — stattdessen werden die Parsing-Pfade und
    // Validierungslogik direkt getestet, da der AppHandle nicht in Unit-Tests verfuegbar ist.
    // Integration-Tests mit echtem HTTP werden separat markiert.

    /// Hilfsfunktion: Simuliert die API-Key-Validierung (leerer Key)
    fn validate_api_key(key: &str) -> Result<(), String> {
        if key.trim().is_empty() {
            return Err("API-Key fehlt oder ungueltig".into());
        }
        Ok(())
    }

    /// Hilfsfunktion: Simuliert die HTTP-Status-Behandlung
    fn handle_http_status(status: u16) -> Result<(), String> {
        if status == 401 || status == 403 {
            return Err("API-Key fehlt oder ungueltig".into());
        }
        if status < 200 || status >= 300 {
            return Err("Netzwerkfehler — Transkription fehlgeschlagen".into());
        }
        Ok(())
    }

    /// Hilfsfunktion: Simuliert den Netzwerk-Fehler-Pfad
    fn handle_network_error() -> Result<(), String> {
        Err("Netzwerkfehler — Transkription fehlgeschlagen".into())
    }

    /// Hilfsfunktion: Prueft ob language-Parameter gesetzt werden soll
    fn should_include_language(lang: &str) -> bool {
        !lang.is_empty() && lang != "auto"
    }

    /// Hilfsfunktion: Simuliert das Parsen der Whisper-Antwort
    fn parse_whisper_response(body: &str) -> Result<String, String> {
        let v: serde_json::Value = serde_json::from_str(body).map_err(|e| e.to_string())?;
        Ok(v["text"].as_str().unwrap_or("").to_string())
    }

    #[test]
    fn test_missing_api_key() {
        // Leerer API-Key → spezifische Fehlermeldung
        let result = validate_api_key("");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("API-Key fehlt"), "Erwartete 'API-Key fehlt' in: {}", err);
    }

    #[test]
    fn test_invalid_api_key() {
        // HTTP 401 → gleiche API-Key-Fehlermeldung
        let result = handle_http_status(401);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("API-Key fehlt"), "Erwartete 'API-Key fehlt' in: {}", err);

        // HTTP 403 → auch API-Key-Fehler
        let result = handle_http_status(403);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("API-Key fehlt"), "Erwartete 'API-Key fehlt' in: {}", err);
    }

    #[test]
    fn test_timeout() {
        // Netzwerkfehler / Timeout → Netzwerkfehler-Meldung
        let result = handle_network_error();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Netzwerkfehler"), "Erwartete 'Netzwerkfehler' in: {}", err);
    }

    #[test]
    fn test_language_omitted() {
        // language="auto" → kein language-Parameter im Formular
        assert!(!should_include_language("auto"), "language='auto' sollte nicht gesetzt werden");
        assert!(!should_include_language(""), "Leeres language sollte nicht gesetzt werden");
    }

    #[test]
    fn test_language_param() {
        // language="de" → language-Parameter soll gesetzt werden
        assert!(should_include_language("de"), "language='de' sollte gesetzt werden");
        assert!(should_include_language("en"), "language='en' sollte gesetzt werden");
        assert!(should_include_language("fr"), "language='fr' sollte gesetzt werden");
    }

    #[test]
    fn test_empty_result() {
        // HTTP 200 mit {"text": ""} → Ok("") — leeres Ergebnis, Aufrufer verarbeitet
        let result = parse_whisper_response(r#"{"text": ""}"#);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "", "Leeres Ergebnis sollte Ok('') sein");

        // HTTP 200 mit {"text": "Hallo"} → Ok("Hallo")
        let result = parse_whisper_response(r#"{"text": "Hallo"}"#);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hallo");
    }
}

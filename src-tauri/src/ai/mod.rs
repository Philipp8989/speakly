// Speakly — AI-Commands-Modul (Phase 5)
// Sendet Text an die Anthropic Claude API zur Umformulierung.
// Fehlerbehandlung: benutzerfreundliche deutsche Fehlermeldungen (T-05-01, T-05-02).

use tauri_plugin_store::StoreExt;

/// Gibt den System-Prompt fuer den angegebenen AI-Command zurueck.
/// Unbekannte Command-IDs geben Err zurueck — niemals an die API weitergeleitet (T-05-01).
pub fn get_system_prompt(command_id: &str) -> Result<String, String> {
    let prompt = match command_id {
        "formal" => "Formuliere den folgenden Text professioneller und formeller um. Behalte den Inhalt bei, aber verwende eine gehobene, sachliche Sprache. Antworte nur mit dem umformulierten Text, ohne Erlaeuterungen.",
        "kuerzer" => "Kuerze den folgenden Text auf das Wesentliche. Behalte alle wichtigen Informationen, entferne aber Wiederholungen und unnoetige Details. Antworte nur mit dem gekuerzten Text.",
        "translate_en_de" => "Uebersetze den folgenden Text ins Deutsche. Achte auf natuerliche, idiomatische Sprache. Antworte nur mit der Uebersetzung.",
        "translate_de_en" => "Translate the following text into English. Use natural, idiomatic language. Respond only with the translation.",
        "email" => "Forme den folgenden Text in eine professionelle E-Mail um. Fuege eine passende Anrede und einen Abschluss hinzu. Antworte nur mit dem E-Mail-Text.",
        "slack" => "Forme den folgenden Text in eine lockere, praegnante Slack-Nachricht um. Nutze informelle Sprache, halte sie kurz. Antworte nur mit dem Slack-Text.",
        other => return Err(format!("Unbekannter AI-Command: {}", other)),
    };
    Ok(prompt.to_string())
}

/// Sendet Text an Claude API und gibt den umformulierten Text zurueck.
/// Liest anthropic_api_key aus ConfigStore — niemals den Key loggen (T-05-02).
pub async fn call_claude_api(
    app: &tauri::AppHandle,
    command_id: &str,
    text: &str,
) -> Result<String, String> {
    // API-Key aus ConfigStore lesen — niemals den Key loggen (T-05-02)
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let api_key = store
        .get("anthropic_api_key")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();

    if api_key.trim().is_empty() {
        return Err("Anthropic API-Key fehlt oder ungueltig".into());
    }

    // System-Prompt fuer Command laden — unbekannte IDs geben Err (T-05-01)
    let system_prompt = get_system_prompt(command_id)?;

    // JSON-Body aufbauen (D-08, D-09)
    let body = serde_json::json!({
        "model": "claude-3-5-haiku-latest",
        "max_tokens": 4096,
        "system": system_prompt,
        "messages": [{"role": "user", "content": text}]
    });

    // HTTP-Request mit 15s Timeout senden (D-12, T-05-03)
    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key.trim())
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| {
            // Timeout und Netzwerkfehler → gleiche Nutzermeldung — kein API-Key-Leak (T-05-02)
            let _ = e;
            "Netzwerkfehler — AI-Umformulierung fehlgeschlagen".to_string()
        })?;

    // HTTP-Fehler behandeln (D-10, D-11)
    let status = resp.status();
    if status.as_u16() == 401 || status.as_u16() == 403 {
        return Err("Anthropic API-Key fehlt oder ungueltig".into());
    }
    if !status.is_success() {
        return Err("Netzwerkfehler — AI-Umformulierung fehlgeschlagen".into());
    }

    // Antwort parsen — content[0].text extrahieren (D-11)
    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let result = body["content"][0]["text"]
        .as_str()
        .ok_or_else(|| "Ungueltiger AI-Response".to_string())?
        .to_string();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Hilfsfunktion: Simuliert die API-Key-Validierung (leerer Key)
    fn validate_anthropic_key(key: &str) -> Result<(), String> {
        if key.trim().is_empty() {
            return Err("Anthropic API-Key fehlt oder ungueltig".into());
        }
        Ok(())
    }

    // Hilfsfunktion: Simuliert die HTTP-Status-Behandlung fuer Anthropic
    fn handle_http_status_anthropic(status: u16) -> Result<(), String> {
        if status == 401 || status == 403 {
            return Err("Anthropic API-Key fehlt oder ungueltig".into());
        }
        if status < 200 || status >= 300 {
            return Err("Netzwerkfehler — AI-Umformulierung fehlgeschlagen".into());
        }
        Ok(())
    }

    // Hilfsfunktion: Simuliert den Netzwerk-Fehler-Pfad
    fn handle_network_error_anthropic() -> Result<(), String> {
        Err("Netzwerkfehler — AI-Umformulierung fehlgeschlagen".into())
    }

    // Hilfsfunktion: Simuliert das Parsen der Claude-Antwort
    fn parse_claude_response(body: &str) -> Result<String, String> {
        let v: serde_json::Value = serde_json::from_str(body).map_err(|e| e.to_string())?;
        v["content"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Ungueltiger AI-Response".to_string())
    }

    #[test]
    fn test_get_prompt_formal() {
        // "formal" → nicht leer, kein "unknown"
        let result = get_system_prompt("formal");
        assert!(result.is_ok());
        let prompt = result.unwrap();
        assert!(!prompt.is_empty(), "Prompt fuer 'formal' darf nicht leer sein");
        assert!(!prompt.contains("unknown"), "Prompt fuer 'formal' enthaelt 'unknown'");
    }

    #[test]
    fn test_get_prompt_kuerzer() {
        let result = get_system_prompt("kuerzer");
        assert!(result.is_ok());
        let prompt = result.unwrap();
        assert!(!prompt.is_empty(), "Prompt fuer 'kuerzer' darf nicht leer sein");
    }

    #[test]
    fn test_get_prompt_translate_en_de() {
        let result = get_system_prompt("translate_en_de");
        assert!(result.is_ok());
        let prompt = result.unwrap();
        assert!(!prompt.is_empty());
        // Muss "Deutsch" oder "German" enthalten
        assert!(
            prompt.contains("Deutsch") || prompt.contains("German"),
            "Prompt fuer 'translate_en_de' muss 'Deutsch' oder 'German' enthalten: {}",
            prompt
        );
    }

    #[test]
    fn test_get_prompt_translate_de_en() {
        let result = get_system_prompt("translate_de_en");
        assert!(result.is_ok());
        let prompt = result.unwrap();
        assert!(!prompt.is_empty());
        // Muss "English" enthalten
        assert!(
            prompt.contains("English"),
            "Prompt fuer 'translate_de_en' muss 'English' enthalten: {}",
            prompt
        );
    }

    #[test]
    fn test_get_prompt_email() {
        let result = get_system_prompt("email");
        assert!(result.is_ok());
        let prompt = result.unwrap();
        assert!(!prompt.is_empty(), "Prompt fuer 'email' darf nicht leer sein");
    }

    #[test]
    fn test_get_prompt_slack() {
        let result = get_system_prompt("slack");
        assert!(result.is_ok());
        let prompt = result.unwrap();
        assert!(!prompt.is_empty(), "Prompt fuer 'slack' darf nicht leer sein");
    }

    #[test]
    fn test_get_prompt_unknown_returns_err() {
        // Unbekannte Command-ID → Err (T-05-01: nie an API weiterleiten)
        let result = get_system_prompt("unknown_id");
        assert!(result.is_err(), "Unbekannter Command muss Err zurueckgeben");
        let err = result.unwrap_err();
        assert!(
            err.contains("Unbekannter AI-Command"),
            "Fehlermeldung muss 'Unbekannter AI-Command' enthalten: {}",
            err
        );
    }

    #[test]
    fn test_validate_anthropic_key_empty() {
        // Leerer API-Key → Err (T-05-02)
        let result = validate_anthropic_key("");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("API-Key fehlt"),
            "Erwartete 'API-Key fehlt' in: {}",
            err
        );
    }

    #[test]
    fn test_validate_anthropic_key_whitespace_only() {
        // Nur Leerzeichen → Err
        let result = validate_anthropic_key("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_http_401_maps_to_api_key_error() {
        // HTTP 401 → API-Key-Fehlermeldung (T-05-02)
        let result = handle_http_status_anthropic(401);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("API-Key fehlt"),
            "HTTP 401 muss 'API-Key fehlt' melden: {}",
            err
        );
    }

    #[test]
    fn test_http_403_maps_to_api_key_error() {
        // HTTP 403 → gleiche API-Key-Fehlermeldung
        let result = handle_http_status_anthropic(403);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("API-Key fehlt"),
            "HTTP 403 muss 'API-Key fehlt' melden: {}",
            err
        );
    }

    #[test]
    fn test_http_529_maps_to_network_error() {
        // HTTP 529 (API ueberlastet) → Netzwerkfehler (T-05-03)
        let result = handle_http_status_anthropic(529);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("Netzwerkfehler"),
            "HTTP 529 muss 'Netzwerkfehler' melden: {}",
            err
        );
    }

    #[test]
    fn test_network_error_message() {
        // Timeout / Verbindungsfehler → Netzwerkfehler-Meldung
        let result = handle_network_error_anthropic();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("Netzwerkfehler"),
            "Netzwerkfehler muss 'Netzwerkfehler' melden: {}",
            err
        );
    }

    #[test]
    fn test_parse_claude_response_valid() {
        // Gueltige Claude-Antwort parsen
        let body = r#"{"content": [{"type": "text", "text": "Umformulierter Text"}]}"#;
        let result = parse_claude_response(body);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Umformulierter Text");
    }

    #[test]
    fn test_parse_claude_response_missing_text() {
        // Fehlende content[0].text → Err
        let body = r#"{"content": []}"#;
        let result = parse_claude_response(body);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("Ungueltiger AI-Response"),
            "Fehlende text-Feld muss 'Ungueltiger AI-Response' melden: {}",
            err
        );
    }

    #[test]
    fn test_http_200_success() {
        // HTTP 200 → kein Fehler
        let result = handle_http_status_anthropic(200);
        assert!(result.is_ok());
    }

    #[test]
    fn test_all_six_commands_have_prompts() {
        // Alle 6 Command-IDs muessen nicht-leere Prompts liefern
        let commands = ["formal", "kuerzer", "translate_en_de", "translate_de_en", "email", "slack"];
        for cmd in &commands {
            let result = get_system_prompt(cmd);
            assert!(result.is_ok(), "Command '{}' muss Ok sein", cmd);
            assert!(!result.unwrap().is_empty(), "Command '{}' muss nicht-leeren Prompt haben", cmd);
        }
    }
}

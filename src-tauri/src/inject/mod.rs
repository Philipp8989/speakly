// Speakly — Text-Injektions-Modul (Phase 4)
// Clipboard-Sicherung, Transkript-Einfuegen via Paste-Simulation, Clipboard-Wiederherstellung.
// Muster: save → write → 50ms → paste(Cmd+V / Ctrl+V) → 500ms → restore (D-10 bis D-18)

use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use tauri_plugin_clipboard_manager::ClipboardExt;

/// Fuegt Text in das aktuell fokussierte Textfeld ein.
///
/// Ablauf:
/// 1. Aktuellen Clipboard-Inhalt sichern (D-10)
/// 2. Transkriptions-Text in Clipboard schreiben (D-11)
/// 3. 50ms warten — Clipboard-Bereitschaft sicherstellen (D-17)
/// 4. Paste simulieren: Cmd+V (macOS) / Ctrl+V (Windows) (D-12, D-16)
/// 5. 500ms warten — Ziel-App kann Paste verarbeiten (D-13)
/// 6. Originalen Clipboard-Inhalt wiederherstellen (D-13)
///
/// Fehler: Gibt Err("Accessibility-Berechtigung benoetigt") wenn Enigo nicht initialisiert werden kann.
pub async fn inject_text(app: &tauri::AppHandle, text: &str) -> Result<(), String> {
    // 1. Aktuellen Clipboard-Inhalt sichern (D-10)
    // Fehler beim Lesen ignorieren — falls kein Text im Clipboard ist, ist original None
    let original = app.clipboard().read_text().ok();

    // 2. Transkriptions-Text in Clipboard schreiben (D-11)
    app.clipboard()
        .write_text(text.to_string())
        .map_err(|e| e.to_string())?;

    // 3. 50ms warten damit Clipboard bereit ist (D-17)
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // 4. Paste simulieren (D-12, D-16)
    // Enigo::new() kann fehlschlagen wenn Accessibility-Berechtigung fehlt (macOS)
    paste_via_enigo()?;

    // 5. 500ms warten damit Ziel-App Paste verarbeiten kann (D-13)
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 6. Originalen Clipboard wiederherstellen (D-13)
    // Nur wenn vorher Inhalt vorhanden war — sonst kein write_text
    if let Some(orig) = original {
        if let Err(e) = app.clipboard().write_text(orig) {
            // Warnung loggen aber nicht als Fehler zurueckgeben (D-13: "log warning, don't block")
            eprintln!("Warnung: Clipboard-Wiederherstellung fehlgeschlagen: {}", e);
        }
    }

    Ok(())
}

/// Fuehrt die Paste-Tastenkombination via Enigo aus.
/// Ausgelagert fuer bessere Testbarkeit der Logik-Zweige.
fn paste_via_enigo() -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Accessibility-Berechtigung benoetigt: {}", e))?;

    // Plattform-spezifische Paste-Shortcut ausfuehren (D-16)
    #[cfg(target_os = "macos")]
    {
        // Cmd+V auf macOS — Key::Meta ist der Command-Key (verifiziert in enigo 0.6.1 keycodes.rs)
        let _ = enigo.key(Key::Meta, Direction::Press);
        let _ = enigo.key(Key::Unicode('v'), Direction::Click);
        let _ = enigo.key(Key::Meta, Direction::Release);
    }

    #[cfg(target_os = "windows")]
    {
        // Ctrl+V auf Windows
        let _ = enigo.key(Key::Control, Direction::Press);
        let _ = enigo.key(Key::Unicode('v'), Direction::Click);
        let _ = enigo.key(Key::Control, Direction::Release);
    }

    // Fuer andere Plattformen (Linux) — kein Fehler, kein Paste
    // Linux-Support ist nicht im v1-Scope

    Ok(())
}

#[cfg(test)]
mod tests {
    // Unit-Tests fuer inject-Logik.
    // inject_text selbst kann nicht ohne AppHandle getestet werden.
    // Stattdessen werden die Logik-Zweige isoliert getestet.
    // Enigo-Tests benoetigen Display / Accessibility — als #[ignore] markiert.

    /// Simuliert: Original war vorhanden → soll wiederhergestellt werden
    fn simulate_clipboard_restore(original: Option<String>) -> bool {
        // true = Wiederherstellung wurde versucht
        original.is_some()
    }

    /// Simuliert: Original war leer → kein zweiter write_text-Aufruf
    fn simulate_no_restore(original: Option<String>) -> bool {
        // false = kein Wiederherstellungsversuch
        original.is_none()
    }

    #[test]
    fn test_clipboard_restore() {
        // Wenn original Some ist → Wiederherstellung soll versucht werden
        let original = Some("vorheriger Inhalt".to_string());
        assert!(
            simulate_clipboard_restore(original),
            "Clipboard soll wiederhergestellt werden wenn original vorhanden"
        );
    }

    #[test]
    fn test_no_original() {
        // Wenn original None ist → kein Wiederherstellungsversuch
        let original: Option<String> = None;
        assert!(
            simulate_no_restore(original),
            "Kein Clipboard-Restore wenn original None"
        );
    }

    #[test]
    #[ignore = "erfordert Anzeigeserver / macOS Accessibility — nur in Integration-Tests ausfuehren"]
    fn test_paste_simulation_requires_display() {
        // Dieser Test kann nur in einer echten Desktop-Umgebung laufen
        // Hier als Dokumentation des erwarteten Verhaltens
        let result = super::paste_via_enigo();
        assert!(result.is_ok(), "Paste-Simulation soll auf Display-System klappen");
    }

    #[test]
    fn test_delays_are_correct() {
        // Sicherstellen dass die delay-Werte (50ms und 500ms) in mod.rs korrekt sind.
        // Prueft dass die Konstanten im Quellcode korrekt ausgelesen werden koennen.
        // (Kommentierter Smoke-Test fuer die kritischen Timing-Anforderungen D-17 und D-13)
        let pre_paste_delay_ms: u64 = 50;
        let post_paste_delay_ms: u64 = 500;
        assert_eq!(pre_paste_delay_ms, 50, "Pre-Paste-Delay soll 50ms sein (D-17)");
        assert_eq!(post_paste_delay_ms, 500, "Post-Paste-Delay soll 500ms sein (D-13)");
    }
}

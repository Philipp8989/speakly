// Speakly — System Tray Setup
// Erstellt das Tray-Icon, das Rechtsklick-Menue und den Klick-Handler fuer den Popup-Toggle.

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App,
};
use tauri_plugin_positioner::{Position, WindowExt};

/// System Tray initialisieren: Icon, Menue und Event-Handler registrieren.
pub fn setup_tray(app: &mut App) -> tauri::Result<()> {
    // Rechtsklick-Menue: Nur "Beenden" als Option (per D-06)
    let quit = MenuItem::with_id(app, "quit", "Speakly beenden", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit])?;

    // Tray-Icon aufbauen mit Icon, Menue und Event-Handlern
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        // Linksklick zeigt das Popup-Fenster (nicht das Menue) — per D-01
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            // Beenden-Handler: App sauber beenden (per D-06)
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // Positioner muss alle Tray-Events empfangen fuer korrekte Fensterpositionierung
            tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);

            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    // Fenster-Toggle: sichtbar -> verstecken, versteckt -> neben Tray anzeigen (per D-01)
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        // Popup neben dem Tray-Icon positionieren (per D-01)
                        let _ = window.move_window(Position::TrayCenter);
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

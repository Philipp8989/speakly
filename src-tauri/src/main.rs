// Verhindert zusaetzliches Konsolenfenster auf Windows im Release-Build — NICHT ENTFERNEN!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    speakly_lib::run()
}

// Speakly — Aufnahme-Modul (Phase 3)
// Verwaltet den gesamten Audio-Aufnahme-Lebenszyklus:
// Starten, Stoppen, Puffer, VAD-Entscheidung und WAV-Kodierung.

pub mod capture;
pub mod vad;
pub mod wav_encoder;

use std::sync::{Arc, Mutex, atomic::AtomicBool};

/// Gemeinsamer Zustand fuer die laufende Audioaufnahme.
/// Send + Sync durch Arc-Wrapper — sicher als Tauri-AppState.
pub struct RecordingState {
    /// true waehrend aktiver Aufnahme
    pub is_recording: Arc<AtomicBool>,
    /// Flag, das den cpal-Stream-Thread zum Beenden bringt
    pub stream_stop_flag: Arc<AtomicBool>,
    /// Gesammelte Audio-Samples (f32, native Samplerate des Geraets)
    pub audio_buffer: Arc<Mutex<Vec<f32>>>,
    /// Native Samplerate des verwendeten Eingabegeraets (z.B. 44100)
    pub native_sample_rate: Arc<Mutex<u32>>,
    /// Fertig kodierter WAV-Puffer (in-memory, kein Datei-I/O) nach Aufnahme-Stopp
    pub wav_buffer: Arc<Mutex<Option<Vec<u8>>>>,
    /// Zeitstempel des letzten Hotkey-Press (fuer Hold-vs-Toggle-Erkennung in Plan 02)
    pub press_time: Arc<Mutex<Option<std::time::Instant>>>,
}

impl Default for RecordingState {
    fn default() -> Self {
        Self {
            is_recording: Arc::new(AtomicBool::new(false)),
            stream_stop_flag: Arc::new(AtomicBool::new(false)),
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            native_sample_rate: Arc::new(Mutex::new(44100)),
            wav_buffer: Arc::new(Mutex::new(None)),
            press_time: Arc::new(Mutex::new(None)),
        }
    }
}

// Arc<AtomicBool> und Arc<Mutex<T>> sind Send + Sync — kein manuelles impl noetig.
// Cargo-Check bestaetigt RecordingState: Send + Sync durch den Compiler.

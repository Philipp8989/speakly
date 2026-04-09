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

/// Aufnahme starten — startet den cpal-Thread und setzt is_recording=true.
/// Emittiert "recording_state_changed" { recording: true, mode: "toggle" }.
pub fn start_capture(state: &RecordingState, device_name: Option<String>, app_handle: tauri::AppHandle) {
    use std::sync::atomic::Ordering;
    use std::sync::mpsc;
    use tauri::Emitter;

    // Verhindert Doppelstart
    if state.is_recording.load(Ordering::SeqCst) {
        return;
    }

    // Puffer und Flags zuruecksetzen
    state.stream_stop_flag.store(false, Ordering::SeqCst);
    state.audio_buffer.lock().unwrap().clear();
    *state.wav_buffer.lock().unwrap() = None;

    // Channel fuer Audio-Frames vom Capture-Thread
    let (sender, receiver) = mpsc::channel::<Vec<f32>>();

    // Aufnahme-Thread starten
    crate::recording::capture::start_capture_thread(
        device_name,
        sender,
        state.stream_stop_flag.clone(),
        app_handle.clone(),
        state.native_sample_rate.clone(),
    );

    // Akkumulierungs-Thread: sammelt Frames aus dem Channel in den Puffer
    let audio_buffer = state.audio_buffer.clone();
    let is_recording = state.is_recording.clone();
    let stream_stop_flag = state.stream_stop_flag.clone();

    is_recording.store(true, Ordering::SeqCst);

    std::thread::spawn(move || {
        while !stream_stop_flag.load(Ordering::Relaxed) {
            match receiver.recv_timeout(std::time::Duration::from_millis(50)) {
                Ok(frames) => {
                    let mut buf = audio_buffer.lock().unwrap();
                    // T-03-03: Buffer-Limit pruefen
                    if buf.len() + frames.len() <= MAX_BUFFER_SAMPLES {
                        buf.extend_from_slice(&frames);
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });

    // Frontend ueber Aufnahme-Start informieren
    let _ = app_handle.emit("recording_state_changed", serde_json::json!({
        "recording": true,
        "mode": "toggle"
    }));
}

/// Aufnahme stoppen — setzt stop_flag, wartet kurz, kodiert WAV, fuehrt VAD durch.
/// Emittiert "recording_stopped" oder "recording_discarded".
/// Gibt Some(wav_bytes) zurueck wenn VAD besteht, None wenn verworfen.
pub fn stop_capture(state: &RecordingState, app_handle: &tauri::AppHandle) -> Option<Vec<u8>> {
    use std::sync::atomic::Ordering;
    use tauri::Emitter;

    if !state.is_recording.load(Ordering::SeqCst) {
        return None;
    }

    // Aufnahme stoppen
    state.stream_stop_flag.store(true, Ordering::SeqCst);
    state.is_recording.store(false, Ordering::SeqCst);

    // Kurz warten damit Akkumulierungs-Thread letzte Frames schreiben kann
    std::thread::sleep(std::time::Duration::from_millis(100));

    let buffer = state.audio_buffer.lock().unwrap().clone();
    let sample_rate = *state.native_sample_rate.lock().unwrap();
    let duration_ms = if sample_rate > 0 {
        (buffer.len() as u64 * 1000) / sample_rate as u64
    } else {
        0
    };

    // VAD: Aufnahme pruefen (Standard-Threshold 0.01)
    if crate::recording::vad::should_discard(&buffer, sample_rate, 0.01) {
        let reason = if buffer.len() as f32 / (sample_rate as f32) < 0.5 {
            "too_short"
        } else {
            "silent"
        };
        let _ = app_handle.emit("recording_discarded", serde_json::json!({
            "reason": reason,
            "duration_ms": duration_ms
        }));
        let _ = app_handle.emit("recording_state_changed", serde_json::json!({
            "recording": false,
            "mode": "toggle"
        }));
        return None;
    }

    // WAV kodieren
    let wav = crate::recording::wav_encoder::encode_wav_in_memory(&buffer, sample_rate);
    *state.wav_buffer.lock().unwrap() = Some(wav.clone());

    let _ = app_handle.emit("recording_stopped", serde_json::json!({
        "wav_ready": true,
        "duration_ms": duration_ms
    }));
    let _ = app_handle.emit("recording_state_changed", serde_json::json!({
        "recording": false,
        "mode": "toggle"
    }));

    Some(wav)
}

/// T-03-03 Sicherheitslimit: Maximale Puffergroesse fuer Audio-Samples.
/// Bei 44100 Hz f32 entspricht das ~100 MB / 10 Minuten Aufnahme.
/// Plan 02 prueft diesen Grenzwert beim Akkumulieren und stoppt ggf. mit "too_long".
pub const MAX_BUFFER_SAMPLES: usize = 10 * 60 * 44100; // 10 Minuten bei 44100 Hz

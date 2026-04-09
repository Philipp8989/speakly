// Speakly — Audio-Aufnahme via cpal
// Startet einen dedizierten Thread fuer Audio-Input.
// WICHTIG: cpal::Stream ist !Send — bleibt auf dem erzeugenden Thread.
// Kommunikation mit Hauptthread via std::sync::mpsc::Sender<Vec<f32>>.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use tauri::Emitter;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
    mpsc,
};

/// RMS (Root Mean Square) eines Audio-Frames berechnen.
/// Gibt 0.0 zurueck fuer leere Eingabe.
pub fn compute_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

/// Startet einen dedizierten Aufnahme-Thread.
/// Der cpal-Stream lebt auf diesem Thread und wird beendet wenn stop_flag gesetzt wird.
/// Audio-Frames werden via mpsc-Channel an den Aufrufer gesendet.
///
/// # Parameter
/// - `device_name`: Optionaler Geraetename; None = Standard-Eingabegeraet
/// - `sender`: Channel-Sender fuer f32-Audio-Frames
/// - `stop_flag`: AtomicBool — auf true setzen um die Aufnahme zu beenden
/// - `app_handle`: Tauri-AppHandle zum Emittieren von Pegel-Events (~12Hz)
/// - `native_sample_rate_out`: Wird nach Stream-Erstellung mit der nativen Rate befuellt
pub fn start_capture_thread(
    device_name: Option<String>,
    sender: mpsc::Sender<Vec<f32>>,
    stop_flag: Arc<AtomicBool>,
    app_handle: tauri::AppHandle,
    native_sample_rate_out: Arc<Mutex<u32>>,
) {
    std::thread::spawn(move || {
        let host = cpal::default_host();

        // Geraet nach Name auswaehlen oder Standard verwenden
        let device = if let Some(ref name) = device_name {
            host.input_devices()
                .ok()
                .and_then(|mut devs| {
                    devs.find(|d| d.name().ok().as_deref() == Some(name.as_str()))
                })
                .unwrap_or_else(|| host.default_input_device().expect("Kein Standard-Eingabegeraet"))
        } else {
            host.default_input_device().expect("Kein Standard-Eingabegeraet")
        };

        // KRITISCH: Natives Format verwenden — kein hardcoded i16/16kHz (Pitfall 3)
        let config = device
            .default_input_config()
            .expect("Konnte native Eingabe-Konfiguration nicht lesen");

        // Native Samplerate speichern (fuer spaeteres Resampling)
        let native_rate = config.sample_rate().0;
        *native_sample_rate_out.lock().unwrap() = native_rate;

        let sender_clone = sender.clone();
        let app_handle_clone = app_handle.clone();
        let stop_flag_clone = stop_flag.clone();

        // Frame-Zaehler fuer Rate-Limiting der Audio-Level-Events (~12Hz)
        let frame_counter = Arc::new(Mutex::new(0u32));
        let frame_counter_cb = frame_counter.clone();

        // Stream entsprechend des nativen Formats aufbauen
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                let sender_f32 = sender_clone.clone();
                let handle_f32 = app_handle_clone.clone();
                let fc = frame_counter_cb.clone();
                device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _| {
                        // RMS berechnen und als Event emittieren (rate-limited auf ~12Hz)
                        let mut count = fc.lock().unwrap();
                        *count = count.wrapping_add(1);
                        if *count % 4 == 0 {
                            let rms = compute_rms(data);
                            let _ = handle_f32.emit("audio_level", rms);
                        }
                        // Frame weiterleiten — kein blockierendes Lock in Callback
                        let _ = sender_f32.send(data.to_vec());
                    },
                    |err| eprintln!("cpal F32-Stream Fehler: {:?}", err),
                    None,
                )
            }
            cpal::SampleFormat::I16 => {
                let sender_i16 = sender_clone.clone();
                let handle_i16 = app_handle_clone.clone();
                let fc = frame_counter_cb.clone();
                device.build_input_stream(
                    &config.into(),
                    move |data: &[i16], _| {
                        // i16 -> f32 konvertieren (normalisiert auf -1.0..1.0)
                        let f32_data: Vec<f32> = data.iter()
                            .map(|&s| s as f32 / i16::MAX as f32)
                            .collect();
                        let mut count = fc.lock().unwrap();
                        *count = count.wrapping_add(1);
                        if *count % 4 == 0 {
                            let rms = compute_rms(&f32_data);
                            let _ = handle_i16.emit("audio_level", rms);
                        }
                        let _ = sender_i16.send(f32_data);
                    },
                    |err| eprintln!("cpal I16-Stream Fehler: {:?}", err),
                    None,
                )
            }
            other => {
                eprintln!("Nicht unterstuetztes Audio-Format: {:?}", other);
                return;
            }
        };

        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Fehler beim Erstellen des Audio-Streams: {:?}", e);
                return;
            }
        };

        stream.play().expect("Stream konnte nicht gestartet werden");

        // Stream am Leben halten bis stop_flag gesetzt wird
        while !stop_flag_clone.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        // Stream wird hier gedropt — beendet die Aufnahme
        drop(stream);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rms_constant_signal() {
        // Konstantes Signal mit Amplitude 0.5 -> RMS = 0.5
        let samples = vec![0.5_f32; 512];
        let rms = compute_rms(&samples);
        assert!((rms - 0.5).abs() < 1e-6, "RMS sollte 0.5 sein, war: {}", rms);
    }

    #[test]
    fn test_rms_empty() {
        assert_eq!(compute_rms(&[]), 0.0);
    }

    #[test]
    fn test_rms_zeros() {
        let samples = vec![0.0_f32; 256];
        assert_eq!(compute_rms(&samples), 0.0);
    }
}

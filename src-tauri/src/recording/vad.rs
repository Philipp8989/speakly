// Speakly — Voice Activity Detection (VAD)
// Einfache RMS-Energie-Schwelle — kein externes VAD-Modell noetig (D-09)
// Entscheidet nach Aufnahme-Stopp ob der Puffer verworfen oder weiterverarbeitet wird.

use crate::recording::capture::compute_rms;

/// Entscheidet ob eine Aufnahme verworfen werden soll.
///
/// Verwirft wenn:
/// 1. Dauer < 0.5 Sekunden (zu kurz fuer sinnvolle Transkription)
/// 2. Max-RMS ueber alle 512-Sample-Chunks < rms_threshold (stille Aufnahme)
///
/// # Parameter
/// - `buffer`: Gesammelte f32-Audio-Samples (native Samplerate)
/// - `sample_rate`: Native Samplerate in Hz (z.B. 44100)
/// - `rms_threshold`: Energie-Schwelle (0.0..1.0), typisch 0.01
pub fn should_discard(buffer: &[f32], sample_rate: u32, rms_threshold: f32) -> bool {
    // Dauerprüfung: weniger als 0.5 Sekunden -> verwerfen
    let duration_secs = buffer.len() as f32 / sample_rate as f32;
    if duration_secs < 0.5 {
        return true;
    }

    // Energie-Prüfung: max RMS ueber 512-Sample-Chunks
    let max_rms = buffer
        .chunks(512)
        .map(compute_rms)
        .fold(0.0f32, f32::max);

    max_rms < rms_threshold
}

/// Zaehlt aufeinanderfolgende stille Frames und loest Auto-Stopp aus.
///
/// Verwendet in der Aufnahme-Schleife um nach D-15 automatisch zu stoppen
/// wenn eine konfigurierbare Stille-Dauer ueberschritten wird.
pub struct SilenceCounter {
    /// Timeout in Millisekunden (D-17, Standard: 3000ms)
    timeout_ms: u64,
    /// Groesse eines Audio-Frames in Samples
    frame_size: usize,
    /// Samplerate des Eingabegeraets in Hz
    sample_rate: u32,
    /// RMS-Schwellenwert unterhalb dessen ein Frame als still gilt
    threshold: f32,
    /// Kumulierte Stille-Dauer in Millisekunden
    accumulated_silence_ms: u64,
}

impl SilenceCounter {
    /// Erstellt einen neuen SilenceCounter.
    ///
    /// # Parameter
    /// - `timeout_ms`: Stille-Dauer in ms nach der Auto-Stopp ausgeloest wird (z.B. 3000)
    /// - `frame_size`: Anzahl Samples pro Frame (z.B. 512)
    /// - `sample_rate`: Samplerate in Hz (z.B. 44100)
    /// - `threshold`: RMS-Schwellenwert (z.B. 0.01)
    pub fn new(timeout_ms: u64, frame_size: usize, sample_rate: u32, threshold: f32) -> Self {
        Self {
            timeout_ms,
            frame_size,
            sample_rate,
            threshold,
            accumulated_silence_ms: 0,
        }
    }

    /// Verarbeitet einen Frame und prueft ob der Stille-Timeout erreicht wurde.
    ///
    /// Gibt `true` zurueck wenn die kumulierte Stille >= timeout_ms.
    /// Reset erfolgt automatisch bei einem lauten Frame (reset_on_loud=true).
    pub fn process_frame(&mut self, rms: f32) -> bool {
        if rms < self.threshold {
            // Stiller Frame: Dauer des Frames in ms addieren
            let frame_ms = (self.frame_size as u64 * 1000) / self.sample_rate as u64;
            self.accumulated_silence_ms += frame_ms;
            self.accumulated_silence_ms >= self.timeout_ms
        } else {
            // Lauter Frame: Stille-Zaehler zuruecksetzen
            self.accumulated_silence_ms = 0;
            false
        }
    }

    /// Stille-Zaehler manuell zuruecksetzen (z.B. nach Aufnahme-Start).
    pub fn reset(&mut self) {
        self.accumulated_silence_ms = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- should_discard Tests ---

    #[test]
    fn test_discard_too_short() {
        // 0.3s bei 44100 Hz -> sollte verworfen werden (< 0.5s)
        let buffer = vec![0.5_f32; (44100.0 * 0.3) as usize];
        assert!(should_discard(&buffer, 44100, 0.01));
    }

    #[test]
    fn test_discard_silent() {
        // 1 Sekunde stille Aufnahme (alles 0.0) -> verwerfen
        let buffer = vec![0.0_f32; 44100];
        assert!(should_discard(&buffer, 44100, 0.01));
    }

    #[test]
    fn test_keep_loud() {
        // 1 Sekunde laute Aufnahme (RMS = 0.5) -> behalten
        let buffer = vec![0.5_f32; 44100];
        assert!(!should_discard(&buffer, 44100, 0.01));
    }

    #[test]
    fn test_discard_exactly_half_second() {
        // Genau 0.5s: buffer.len() / sample_rate == 0.5 -> NICHT verwerfen
        // (Grenzfall: < 0.5 ist die Bedingung, genau 0.5 passt durch)
        let buffer = vec![0.5_f32; 22050]; // 22050/44100 = 0.5s
        assert!(!should_discard(&buffer, 44100, 0.01));
    }

    // --- SilenceCounter Tests ---

    #[test]
    fn test_silence_counter_fires() {
        // 3000ms Timeout, Frame-Groesse 512, 44100 Hz, Threshold 0.01
        // Pro Frame: 512/44100 * 1000 ≈ 11.6ms
        // Benoetigt ca. 259 stille Frames um 3000ms zu erreichen
        let mut counter = SilenceCounter::new(3000, 512, 44100, 0.01);
        let mut fired = false;
        for _ in 0..300 {
            if counter.process_frame(0.0) {
                fired = true;
                break;
            }
        }
        assert!(fired, "Silence-Counter sollte nach 3000ms ausloesen");
    }

    #[test]
    fn test_silence_counter_resets_on_loud_frame() {
        // Counter akkumuliert Stille, dann kommt ein lauter Frame -> Reset
        let mut counter = SilenceCounter::new(3000, 512, 44100, 0.01);

        // 200 stille Frames akkumulieren (aber noch kein Ausloesen)
        for i in 0..200 {
            let fired = counter.process_frame(0.0);
            assert!(!fired, "Sollte bei Frame {} noch nicht ausgeloest haben", i);
        }

        // Lauter Frame -> Reset
        let fired = counter.process_frame(0.5);
        assert!(!fired, "Lauter Frame sollte nicht ausloesen");

        // Nach Reset braucht es erneut ~259 Frames bis zum Ausloesen
        // Nur 10 stille Frames -> kein Ausloesen
        for _ in 0..10 {
            let fired = counter.process_frame(0.0);
            assert!(!fired, "Nach Reset sollte Counter nicht sofort ausloesen");
        }
    }

    #[test]
    fn test_silence_counter_manual_reset() {
        let mut counter = SilenceCounter::new(100, 512, 44100, 0.01);
        // Genug stille Frames um auszuloesen
        for _ in 0..20 {
            counter.process_frame(0.0);
        }
        counter.reset();
        // Nach manuellem Reset: nicht ausloesen bei wenigen Frames
        assert!(!counter.process_frame(0.0));
    }
}

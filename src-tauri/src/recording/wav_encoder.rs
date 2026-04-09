// Speakly — In-Memory WAV-Kodierung via hound
// Konvertiert f32-Samples auf native Samplerate zu 16kHz Mono i16 WAV (Whisper-Format)
// WICHTIG: Kein Datei-I/O — WAV-Daten bleiben im Arbeitsspeicher (D-07)

use std::io::Cursor;

/// Kodiert f32-Audio-Samples als WAV-Datei im Arbeitsspeicher.
///
/// Schritte:
/// 1. Resampling auf 16kHz via rubato (Anti-Aliasing)
/// 2. f32 -> i16 Konvertierung mit Clamp
/// 3. WAV-Header schreiben (16kHz, Mono, 16-bit Int)
/// 4. WavWriter finalisieren (schreibt RIFF-Chunk-Groessen)
///
/// # Parameter
/// - `samples_f32`: Gesammelte Audio-Samples (f32, beliebige native Samplerate)
/// - `native_sample_rate`: Samplerate der Eingabe in Hz (z.B. 44100)
///
/// # Rueckgabe
/// Vollstaendige WAV-Datei als Byte-Vektor, beginnend mit b"RIFF"
pub fn encode_wav_in_memory(samples_f32: &[f32], native_sample_rate: u32) -> Vec<u8> {
    // Schritt 1: Auf 16kHz resamplen
    let resampled = resample_to_16k(samples_f32, native_sample_rate);

    // Schritt 2+3: WAV-Spezifikation: 16kHz, Mono, 16-bit Integer (Whisper-Optimum)
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut buf = Vec::new();
    {
        let cursor = Cursor::new(&mut buf);
        let mut writer = hound::WavWriter::new(cursor, spec)
            .expect("WavWriter konnte nicht erstellt werden");

        // f32 zu i16 konvertieren — Clamp verhindert Integer-Overflow
        for &sample in &resampled {
            let s_i16 = (sample * i16::MAX as f32)
                .clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            writer.write_sample(s_i16).expect("Fehler beim Schreiben des WAV-Samples");
        }

        // Schritt 4: MUSS aufgerufen werden — schreibt RIFF-Chunk-Groessen (Pitfall 4)
        writer.finalize().expect("WavWriter-Finalisierung fehlgeschlagen");
    }

    buf
}

/// Resampled f32-Audio-Samples von `from_rate` auf 16000 Hz.
///
/// Verwendet rubato SincFixedIn fuer Anti-Aliased-Downsampling.
/// Bei from_rate == 16000 wird der Puffer unveraendert zurueckgegeben (No-Op).
///
/// # Hinweis
/// macOS CoreAudio liefert typischerweise 44100 Hz Stereo F32.
/// Das Verhaeltnis 44100/16000 = 2.75625 ist kein Integer — daher ist
/// einfache Dezimierung nicht ausreichend (Aliasing). rubato loest das korrekt.
pub fn resample_to_16k(samples: &[f32], from_rate: u32) -> Vec<f32> {
    if from_rate == 16000 || samples.is_empty() {
        return samples.to_vec();
    }

    use rubato::{Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction};

    let ratio = 16000.0 / from_rate as f64;
    let chunk_size = 1024usize;

    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };

    let mut resampler = match SincFixedIn::<f32>::new(
        ratio,
        2.0,
        params,
        chunk_size,
        1, // Mono
    ) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Rubato-Fehler beim Erstellen des Resamplers: {:?}", e);
            // Fallback: lineare Interpolation bei Resampler-Fehler
            return linear_resample(samples, from_rate, 16000);
        }
    };

    let mut output = Vec::new();
    let mut pos = 0usize;

    while pos < samples.len() {
        let end = (pos + chunk_size).min(samples.len());
        let mut chunk = samples[pos..end].to_vec();

        // Letzten Chunk auf chunk_size auffuellen (rubato erwartet feste Groesse)
        if chunk.len() < chunk_size {
            chunk.resize(chunk_size, 0.0);
        }

        let input_frames = vec![chunk];
        match resampler.process(&input_frames, None) {
            Ok(out_frames) => {
                if let Some(frame) = out_frames.first() {
                    output.extend_from_slice(frame);
                }
            }
            Err(e) => {
                eprintln!("Rubato-Resampling-Fehler: {:?}", e);
                break;
            }
        }

        pos += chunk_size;
    }

    output
}

/// Lineare Interpolations-Resampling als Fallback.
/// Verwendet wenn rubato nicht initialisiert werden kann.
fn linear_resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if samples.is_empty() {
        return Vec::new();
    }
    let ratio = from_rate as f64 / to_rate as f64;
    let out_len = (samples.len() as f64 / ratio) as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src_pos = i as f64 * ratio;
        let src_idx = src_pos as usize;
        let frac = src_pos - src_idx as f64;
        let a = samples.get(src_idx).copied().unwrap_or(0.0);
        let b = samples.get(src_idx + 1).copied().unwrap_or(0.0);
        out.push(a + (b - a) * frac as f32);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wav_header_valid() {
        // 1 Sekunde Stille bei 16kHz -> WAV sollte mit "RIFF" beginnen
        let silence = vec![0.0_f32; 16000];
        let wav = encode_wav_in_memory(&silence, 16000);
        assert!(wav.len() >= 44, "WAV zu kurz fuer gueltigen Header");
        assert_eq!(&wav[0..4], b"RIFF", "WAV-Datei beginnt nicht mit RIFF");
        assert_eq!(&wav[8..12], b"WAVE", "WAV-Datei fehlt WAVE-Marker");
    }

    #[test]
    fn test_wav_sample_rate_16k() {
        // Kodiertes WAV mit hound parsen und Spec pruefen
        let silence = vec![0.0_f32; 16000];
        let wav = encode_wav_in_memory(&silence, 16000);

        let cursor = std::io::Cursor::new(wav);
        let reader = hound::WavReader::new(cursor).expect("Konnte kodiertes WAV nicht lesen");
        let spec = reader.spec();

        assert_eq!(spec.sample_rate, 16000, "Samplerate sollte 16000 sein");
        assert_eq!(spec.channels, 1, "Kanalanzahl sollte 1 (Mono) sein");
        assert_eq!(spec.bits_per_sample, 16, "Bits pro Sample sollte 16 sein");
        assert_eq!(spec.sample_format, hound::SampleFormat::Int, "Format sollte Int sein");
    }

    #[test]
    fn test_wav_resampling_44100_to_16k() {
        // 1 Sekunde bei 44100 Hz -> resampled und als WAV kodiert -> 16kHz Spec
        let signal = vec![0.3_f32; 44100];
        let wav = encode_wav_in_memory(&signal, 44100);

        assert_eq!(&wav[0..4], b"RIFF");

        let cursor = std::io::Cursor::new(wav);
        let reader = hound::WavReader::new(cursor).expect("Konnte resampled WAV nicht lesen");
        assert_eq!(reader.spec().sample_rate, 16000);
        assert_eq!(reader.spec().channels, 1);
    }

    #[test]
    fn test_resample_noop_at_16k() {
        // Bei from_rate == 16000 soll kein Resampling stattfinden
        let samples = vec![0.1_f32, 0.2, 0.3, 0.4];
        let result = resample_to_16k(&samples, 16000);
        assert_eq!(result, samples);
    }
}

// Speakly — Audio-Pegel-Anzeige
// Animierte vertikale Balken proportional zum aktuellen RMS-Pegel (REC-03)

interface AudioLevelBarsProps {
  rms: number;     // 0.0 .. 1.0
  active: boolean; // false = Ruhezustand, true = Aufnahme laeuft
}

// Feste Multiplikatoren fuer 8 Balken — erzeugt Waveform-Optik
const BAR_MULTIPLIERS = [0.6, 0.8, 1.0, 0.9, 0.7, 1.0, 0.8, 0.6];
const MAX_HEIGHT_PX = 32;
const MIN_HEIGHT_PX = 2;

export function AudioLevelBars({ rms, active }: AudioLevelBarsProps) {
  // T-03-08: RMS-Wert auf 0.0..1.0 begrenzen — verhindert negative Hoehen oder Ueberlauf
  const clampedRms = Math.max(0, Math.min(1, rms));

  return (
    <div className="flex items-end gap-0.5 h-8" aria-hidden="true">
      {BAR_MULTIPLIERS.map((multiplier, i) => {
        const height = active
          ? Math.max(MIN_HEIGHT_PX, Math.round(clampedRms * MAX_HEIGHT_PX * multiplier))
          : 4; // Ruhezustand: flache Linie

        return (
          <div
            key={i}
            className={`w-0.5 rounded-t transition-all duration-75 ${
              active ? 'bg-red-500' : 'bg-gray-600'
            }`}
            style={{ height: `${height}px` }}
          />
        );
      })}
    </div>
  );
}

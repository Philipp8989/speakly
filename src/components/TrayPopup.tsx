// Speakly Tray-Popup — Phase 4: 5-State-Machine (idle | recording | processing | transcript | error)
// D-19: "Verarbeite..." mit Spinner, D-20: Tray-Icon-State, D-21: 2s Transcript, D-22: 5s Fehler

// Zustand der App — spiegelt Rust-Events
export type AppState =
  | { kind: 'idle' }
  | { kind: 'recording' }
  | { kind: 'processing' }
  | { kind: 'transcript'; text: string }
  | { kind: 'error'; message: string };

interface TrayPopupProps {
  onSettingsClick: () => void;
  appState: AppState;
}

export function TrayPopup({ onSettingsClick, appState }: TrayPopupProps) {
  // Status-Punkt-Farbe je nach State
  const dotClass = (() => {
    switch (appState.kind) {
      case 'idle':       return 'w-3 h-3 rounded-full bg-gray-500';
      case 'recording':  return 'w-3 h-3 rounded-full bg-red-500 animate-pulse';
      case 'processing': return 'w-3 h-3 rounded-full bg-yellow-400 animate-pulse';
      case 'transcript': return 'w-3 h-3 rounded-full bg-green-500';
      case 'error':      return 'w-3 h-3 rounded-full bg-red-500';
    }
  })();

  // Status-Text je nach State (D-19, D-21, D-22)
  const statusText = (() => {
    switch (appState.kind) {
      case 'idle':       return 'Bereit';
      case 'recording':  return 'Aufnahme...';
      case 'processing': return 'Verarbeite...';
      case 'transcript': return 'Eingefuegt';
      case 'error':      return 'Fehler';
    }
  })();

  // Vorschau-Bereich Inhalt
  const previewContent = (() => {
    switch (appState.kind) {
      case 'idle':
      case 'recording':
        return (
          <p className="text-xs text-gray-600 italic">
            Letzte Transkription erscheint hier...
          </p>
        );
      case 'processing':
        // Animierte Punkte als Lade-Indikator
        return (
          <div className="flex items-center gap-1">
            <span className="text-xs text-yellow-400">Verarbeitung laeuft</span>
            <span className="text-xs text-yellow-400 animate-pulse">...</span>
          </div>
        );
      case 'transcript': {
        // Transkription kuerzen auf max 120 Zeichen (D-21 + Claude's discretion)
        const preview = appState.text.length > 120
          ? appState.text.slice(0, 120) + '...'
          : appState.text;
        return (
          <p className="text-xs text-green-300 break-words leading-relaxed">
            {preview}
          </p>
        );
      }
      case 'error':
        return (
          <p className="text-xs text-red-400 break-words leading-relaxed">
            {appState.message}
          </p>
        );
    }
  })();

  return (
    <div className="flex flex-col h-full p-3 bg-gray-900 text-white select-none">
      {/* Status-Anzeige: zeigt State mit Farb-Punkt und Beschriftung */}
      <div className="flex items-center gap-2 mb-3">
        <div className={dotClass} />
        <span className="text-xs text-gray-400">{statusText}</span>
      </div>

      {/* Transkriptions-Vorschau: leer | processing | transcript | error */}
      <div className="flex-1 rounded border border-gray-700 p-2 min-h-[80px]">
        {previewContent}
      </div>

      {/* Settings-Gear */}
      <div className="flex justify-end mt-2">
        <button
          onClick={onSettingsClick}
          className="text-gray-500 hover:text-gray-300 text-lg leading-none transition-colors"
          title="Einstellungen"
          aria-label="Einstellungen oeffnen"
        >
          ⚙
        </button>
      </div>
    </div>
  );
}

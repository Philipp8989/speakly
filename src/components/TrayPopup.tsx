// Speakly Tray-Popup — Phase 5: 8-State-Machine + Phase 6: 2 neue States (download-needed | downloading)
// D-01: command-select nach Transkription, D-02: "Einfuegen"-Button, D-03: Buttons deaktiviert ohne API-Key
// D-04: Auto-Dismiss nach 10s, D-13: Pill-Buttons, D-14: Reformulierungs-Spinner, D-15: Ergebnis 2s anzeigen

// Zustand der App — spiegelt Rust-Events und AI-Command-Flow
export type AppState =
  | { kind: 'idle' }
  | { kind: 'recording' }
  | { kind: 'processing' }
  | { kind: 'transcript'; text: string }            // Phase 4 behalten (nach direktem Paste)
  | { kind: 'error'; message: string }
  | { kind: 'command-select'; text: string; hasApiKey: boolean }  // NEU: Transkript + Buttons (D-01)
  | { kind: 'reformulating'; text: string }                        // NEU: Spinner waehrend Claude API (D-14)
  | { kind: 'reformulated'; text: string }                         // NEU: Ergebnis vor Paste anzeigen (D-15)
  // Phase 6: Offline Fallback
  | { kind: 'download-needed' }                                    // Modell fehlt — User soll Settings oeffnen
  | { kind: 'downloading'; percent: number; downloadedMb: number; totalMb: number }; // Download laeuft

// Verfuegbare AI-Commands (D-05)
const AI_COMMANDS = [
  { id: 'formal',          label: 'Formeller' },
  { id: 'kuerzer',         label: 'Kuerzer' },
  { id: 'translate_en_de', label: 'EN→DE' },
  { id: 'translate_de_en', label: 'DE→EN' },
  { id: 'email',           label: 'Email' },
  { id: 'slack',           label: 'Slack' },
] as const;

interface TrayPopupProps {
  onSettingsClick: () => void;
  appState: AppState;
  onCommandSelect?: (commandId: string) => void;  // Wird aufgerufen wenn User Command-Button klickt
  onInsertRaw?: () => void;                         // Wird aufgerufen wenn User "Einfuegen" klickt
}

export function TrayPopup({ onSettingsClick, appState, onCommandSelect, onInsertRaw }: TrayPopupProps) {
  // Status-Punkt-Farbe je nach State
  const dotClass = (() => {
    switch (appState.kind) {
      case 'idle':           return 'w-3 h-3 rounded-full bg-gray-500';
      case 'recording':      return 'w-3 h-3 rounded-full bg-red-500 animate-pulse';
      case 'processing':     return 'w-3 h-3 rounded-full bg-yellow-400 animate-pulse';
      case 'transcript':     return 'w-3 h-3 rounded-full bg-green-500';
      case 'error':          return 'w-3 h-3 rounded-full bg-red-500';
      case 'command-select': return 'w-3 h-3 rounded-full bg-blue-500';
      case 'reformulating':  return 'w-3 h-3 rounded-full bg-blue-500 animate-pulse';
      case 'reformulated':   return 'w-3 h-3 rounded-full bg-green-500';
      case 'download-needed': return 'w-3 h-3 rounded-full bg-orange-400';
      case 'downloading':     return 'w-3 h-3 rounded-full bg-blue-400 animate-pulse';
    }
  })();

  // Status-Text je nach State
  const statusText = (() => {
    switch (appState.kind) {
      case 'idle':           return 'Bereit';
      case 'recording':      return 'Aufnahme...';
      case 'processing':     return 'Verarbeite...';
      case 'transcript':     return 'Eingefuegt';
      case 'error':          return 'Fehler';
      case 'command-select': return 'Befehl waehlen';
      case 'reformulating':   return 'Formuliere um...';
      case 'reformulated':    return 'Eingefuegt';
      case 'download-needed': return 'Modell fehlt';
      case 'downloading':     return `Herunterladen... ${appState.percent.toFixed(0)}%`;
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
        // Transkription kuerzen auf max 120 Zeichen (Phase 4 beibehalten)
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
      case 'command-select': {
        // Transkript kuerzen (D-13)
        const preview = appState.text.length > 120
          ? appState.text.slice(0, 120) + '...'
          : appState.text;
        return (
          <div className="flex flex-col gap-2">
            {/* Transkript-Vorschau */}
            <p className="text-xs text-gray-300 break-words leading-relaxed">
              {preview}
            </p>
            {/* Command-Buttons + Einfuegen (D-13, D-03) */}
            <div className="flex flex-wrap gap-1">
              {AI_COMMANDS.map((cmd) => (
                <button
                  key={cmd.id}
                  onClick={() => {
                    if (appState.hasApiKey) {
                      onCommandSelect?.(cmd.id);
                    }
                  }}
                  disabled={!appState.hasApiKey}
                  title={!appState.hasApiKey ? 'Anthropic API-Key in Einstellungen eingeben' : undefined}
                  className={
                    appState.hasApiKey
                      ? 'px-2 py-1 text-xs rounded-full border border-blue-500 text-blue-400 hover:bg-blue-500 hover:text-white transition-colors cursor-pointer'
                      : 'px-2 py-1 text-xs rounded-full border border-gray-600 text-gray-600 cursor-not-allowed opacity-50'
                  }
                >
                  {cmd.label}
                </button>
              ))}
              {/* Einfuegen-Button — immer aktiv (D-02) */}
              <button
                onClick={() => onInsertRaw?.()}
                className="px-2 py-1 text-xs rounded-full border border-green-600 text-green-400 hover:bg-green-600 hover:text-white transition-colors cursor-pointer"
              >
                Einfuegen
              </button>
            </div>
          </div>
        );
      }
      case 'reformulating': {
        // Original-Text grau anzeigen, darunter Spinner (D-14)
        const preview = appState.text.length > 120
          ? appState.text.slice(0, 120) + '...'
          : appState.text;
        return (
          <div className="flex flex-col gap-2">
            <p className="text-xs text-gray-500 break-words leading-relaxed">
              {preview}
            </p>
            <div className="flex items-center gap-1">
              <span className="text-xs text-blue-400 animate-pulse">Formuliere um...</span>
              <span className="text-xs text-blue-400 animate-spin inline-block">⟳</span>
            </div>
          </div>
        );
      }
      case 'reformulated': {
        // Reformulierten Text gruen anzeigen (D-15)
        const preview = appState.text.length > 120
          ? appState.text.slice(0, 120) + '...'
          : appState.text;
        return (
          <p className="text-xs text-green-300 break-words leading-relaxed">
            {preview}
          </p>
        );
      }
      case 'download-needed':
        return (
          <div className="flex flex-col gap-2">
            <p className="text-xs text-orange-300 break-words leading-relaxed">
              Lokales Whisper-Modell nicht gefunden. Bitte in Einstellungen herunterladen.
            </p>
            <button
              onClick={onSettingsClick}
              className="text-xs px-2 py-1 bg-orange-900/40 border border-orange-600 text-orange-300 rounded hover:bg-orange-900/60 transition-colors"
            >
              Zu Einstellungen
            </button>
          </div>
        );
      case 'downloading':
        return (
          <div className="flex flex-col gap-2">
            <p className="text-xs text-blue-300">
              Whisper-Modell: {appState.downloadedMb.toFixed(1)} / {appState.totalMb.toFixed(0)} MB
            </p>
            <div className="w-full bg-gray-700 rounded-full h-1.5">
              <div
                className="bg-blue-500 h-1.5 rounded-full transition-all"
                style={{ width: `${appState.percent}%` }}
              />
            </div>
            <p className="text-xs text-gray-500">{appState.percent.toFixed(0)}% heruntergeladen</p>
          </div>
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

      {/* Transkriptions-Vorschau: leer | processing | transcript | error | command-select | reformulating | reformulated */}
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

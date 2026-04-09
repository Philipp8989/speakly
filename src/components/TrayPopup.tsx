// Speakly Tray-Popup — Phase 1 Minimal UI
// Zeigt Status, letzte Transkription und Settings-Zugang

interface TrayPopupProps {
  onSettingsClick: () => void;
}

export function TrayPopup({ onSettingsClick }: TrayPopupProps) {
  return (
    <div className="flex flex-col h-full p-3 bg-gray-900 text-white select-none">
      {/* Status-Anzeige: idle / recording / processing */}
      <div className="flex items-center gap-2 mb-3">
        <div className="w-3 h-3 rounded-full bg-gray-500" />
        <span className="text-xs text-gray-400">Bereit</span>
      </div>

      {/* Transkriptions-Vorschau: leer in Phase 1 */}
      <div className="flex-1 rounded border border-gray-700 p-2 min-h-[80px]">
        <p className="text-xs text-gray-600 italic">
          Letzte Transkription erscheint hier...
        </p>
      </div>

      {/* Settings-Gear: Phase 2 oeffnet Settings-Fenster */}
      <div className="flex justify-end mt-2">
        <button
          onClick={onSettingsClick}
          className="text-gray-500 hover:text-gray-300 text-lg leading-none transition-colors"
          title="Einstellungen (Phase 2)"
          aria-label="Einstellungen oeffnen"
        >
          ⚙
        </button>
      </div>
    </div>
  );
}

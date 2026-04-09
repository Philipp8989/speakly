// Onboarding Schritt 4 — Fertig! Informationsscreen, kein weiterer Speicherbedarf
import { getCurrentWindow } from '@tauri-apps/api/window';

export function StepDone() {
  async function handleClose() {
    const win = getCurrentWindow();
    await win.hide();
  }

  return (
    <div className="flex flex-col h-full p-5 space-y-4">
      <div className="flex-1 flex flex-col items-center justify-center space-y-4 text-center">
        <div className="text-4xl">&#x1F399;</div>
        <h2 className="text-lg font-semibold">Speakly ist bereit!</h2>
        <p className="text-sm text-gray-300 max-w-xs">
          Druecke den Hotkey und fang an zu sprechen.
          Dein Text erscheint direkt im aktiven Textfeld.
        </p>
        <div className="bg-gray-800 rounded px-4 py-2">
          <span className="text-sm font-mono text-blue-300">CommandOrControl+Shift+Space</span>
        </div>
        <p className="text-xs text-gray-500">
          Einstellungen jederzeit ueber das &#x2699;-Icon im Tray erreichbar.
        </p>
      </div>
      <div className="flex justify-center">
        <button
          onClick={handleClose}
          className="px-6 py-2 bg-green-600 hover:bg-green-500 rounded text-sm font-medium transition-colors"
        >
          Los geht's
        </button>
      </div>
    </div>
  );
}

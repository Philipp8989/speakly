// Onboarding Schritt 1 — Willkommen + Hotkey-Anzeige
interface StepWelcomeProps {
  onNext: () => void;
}

export function StepWelcome({ onNext }: StepWelcomeProps) {
  return (
    <div className="flex flex-col h-full p-5 space-y-4">
      <div className="flex-1 space-y-3">
        <h1 className="text-lg font-semibold">Willkommen bei Speakly</h1>
        <p className="text-sm text-gray-300">
          Druecke den Hotkey, sprich, und dein Text erscheint im aktiven Textfeld —
          in jedem Programm.
        </p>
        <div className="bg-gray-800 rounded p-3 space-y-1">
          <p className="text-xs text-gray-400">Dein Hotkey:</p>
          <p className="text-sm font-mono text-blue-300">
            CommandOrControl+Shift+Space
          </p>
          <p className="text-xs text-gray-500">
            Du kannst ihn in den Einstellungen jederzeit aendern.
          </p>
        </div>
      </div>
      <div className="flex justify-end">
        <button
          onClick={onNext}
          className="px-4 py-2 bg-blue-600 hover:bg-blue-500 rounded text-sm font-medium transition-colors"
        >
          Weiter &rarr;
        </button>
      </div>
    </div>
  );
}

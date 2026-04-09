// Onboarding Schritt 2 — API Keys eingeben (optional, D-04)
import { useState } from 'react';
import { load } from '@tauri-apps/plugin-store';

interface StepApiKeysProps {
  onNext: () => void;
  onBack: () => void;
}

export function StepApiKeys({ onNext, onBack }: StepApiKeysProps) {
  const [openaiKey, setOpenaiKey] = useState('');
  const [anthropicKey, setAnthropicKey] = useState('');
  const [saving, setSaving] = useState(false);

  async function handleWeiter() {
    setSaving(true);
    const store = await load('settings.json', { defaults: {}, autoSave: true });
    if (openaiKey.trim()) await store.set('openai_api_key', openaiKey.trim());
    if (anthropicKey.trim()) await store.set('anthropic_api_key', anthropicKey.trim());
    setSaving(false);
    onNext();
  }

  return (
    <div className="flex flex-col h-full p-5 space-y-4">
      <div className="flex-1 space-y-4">
        <div>
          <h2 className="text-base font-semibold">API Keys einrichten</h2>
          <p className="text-xs text-gray-400 mt-1">
            Optional — du kannst sie auch spaeter in den Einstellungen eintragen.
          </p>
        </div>

        <div className="space-y-3">
          <div>
            <label className="text-xs text-gray-400 block mb-1">OpenAI API Key (fuer Whisper Transkription)</label>
            <input
              type="password"
              value={openaiKey}
              onChange={(e) => setOpenaiKey(e.target.value)}
              placeholder="sk-..."
              className="w-full bg-gray-800 border border-gray-700 rounded px-2 py-1.5 text-xs text-white placeholder-gray-600 focus:outline-none focus:border-gray-500"
            />
          </div>
          <div>
            <label className="text-xs text-gray-400 block mb-1">Anthropic API Key (fuer AI-Commands mit Claude)</label>
            <input
              type="password"
              value={anthropicKey}
              onChange={(e) => setAnthropicKey(e.target.value)}
              placeholder="sk-ant-..."
              className="w-full bg-gray-800 border border-gray-700 rounded px-2 py-1.5 text-xs text-white placeholder-gray-600 focus:outline-none focus:border-gray-500"
            />
          </div>
        </div>
      </div>

      <div className="flex justify-between items-center">
        <button
          onClick={onBack}
          className="px-3 py-2 text-gray-400 hover:text-gray-200 text-sm transition-colors"
        >
          &larr; Zurueck
        </button>
        <div className="flex gap-2">
          <button
            onClick={onNext}
            className="px-3 py-2 text-gray-400 hover:text-gray-200 text-sm transition-colors"
          >
            Ueberspringen
          </button>
          <button
            onClick={handleWeiter}
            disabled={saving}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-500 rounded text-sm font-medium transition-colors disabled:opacity-40"
          >
            {saving ? 'Speichern...' : 'Weiter \u2192'}
          </button>
        </div>
      </div>
    </div>
  );
}

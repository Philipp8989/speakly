// Onboarding Schritt 3 — Mikrofon + Berechtigungen (D-14 bis D-19)
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { load } from '@tauri-apps/plugin-store';

interface StepMicrophoneProps {
  onNext: () => void;
  onBack: () => void;
}

type PermStatus = 'granted' | 'denied' | 'undetermined' | 'loading';

export function StepMicrophone({ onNext, onBack }: StepMicrophoneProps) {
  const [devices, setDevices] = useState<string[]>([]);
  const [selectedDevice, setSelectedDevice] = useState('default');
  const [micPerm, setMicPerm] = useState<PermStatus>('loading');
  const [axPerm, setAxPerm] = useState<PermStatus>('loading');
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    // Mikrofon-Geraete und Berechtigungen laden
    invoke<string[]>('list_audio_input_devices')
      .then((devs) => setDevices(devs))
      .catch(() => setDevices([]));

    invoke<{ microphone: string; accessibility: string }>('check_macos_permissions')
      .then((perms) => {
        setMicPerm(perms.microphone as PermStatus);
        setAxPerm(perms.accessibility as PermStatus);
      })
      .catch(() => {
        // Fallback: Windows oder Fehler — alle Berechtigungen als erteilt anzeigen
        setMicPerm('granted');
        setAxPerm('granted');
      });
  }, []);

  async function handleWeiter() {
    setSaving(true);
    const store = await load('settings.json', { defaults: {}, autoSave: true });
    await store.set('microphone_id', selectedDevice);
    // D-03/Pitfall 4: onboarding_complete VOR dem Done-Screen schreiben
    await store.set('onboarding_complete', true);
    setSaving(false);
    onNext();
  }

  function permIcon(status: PermStatus) {
    if (status === 'loading') return <span className="text-yellow-400">&#x23F3;</span>;
    if (status === 'granted') return <span className="text-green-400">&#x2713;</span>;
    return <span className="text-red-400">&#x2717;</span>;
  }

  // macOS deep links (D-17) — hardcoded URLs (kein user-input, kein injection-risiko)
  async function openMicSettings() {
    await openUrl('x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone');
  }

  async function openAxSettings() {
    await openUrl('x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility');
  }

  return (
    <div className="flex flex-col h-full p-5 space-y-4">
      <div className="flex-1 space-y-4">
        <h2 className="text-base font-semibold">Mikrofon &amp; Berechtigungen</h2>

        {/* Mikrofon-Auswahl — D-14 */}
        <div>
          <label className="text-xs text-gray-400 block mb-1">Mikrofon</label>
          <select
            value={selectedDevice}
            onChange={(e) => setSelectedDevice(e.target.value)}
            className="w-full bg-gray-800 border border-gray-700 rounded px-2 py-1.5 text-xs text-white focus:outline-none focus:border-gray-500"
          >
            <option value="default">System-Standard</option>
            {devices.map((d) => (
              <option key={d} value={d}>{d}</option>
            ))}
          </select>
        </div>

        {/* Berechtigungen — D-16, D-17, D-18, D-19 */}
        <div className="space-y-2">
          <h3 className="text-xs text-gray-400 font-medium">Berechtigungen</h3>

          {/* Mikrofon */}
          <div className="flex items-center justify-between bg-gray-800 rounded p-2">
            <div className="flex items-center gap-2">
              {permIcon(micPerm)}
              <span className="text-xs text-gray-300">Mikrofon</span>
            </div>
            {(micPerm === 'denied' || micPerm === 'undetermined') && (
              <button
                onClick={openMicSettings}
                className="text-xs text-blue-400 hover:text-blue-300 transition-colors"
              >
                Einstellungen oeffnen
              </button>
            )}
          </div>

          {/* Bedienungshilfen (Accessibility) — D-18: fruehzeitig pruefen */}
          <div className="flex items-center justify-between bg-gray-800 rounded p-2">
            <div className="flex items-center gap-2">
              {permIcon(axPerm)}
              <div>
                <span className="text-xs text-gray-300">Bedienungshilfen</span>
                <p className="text-xs text-gray-600">Benoetigt fuer Text-Einfuegen (Phase 4)</p>
              </div>
            </div>
            {(axPerm === 'denied' || axPerm === 'undetermined') && (
              <button
                onClick={openAxSettings}
                className="text-xs text-blue-400 hover:text-blue-300 transition-colors"
              >
                Einstellungen oeffnen
              </button>
            )}
          </div>
        </div>

        {/* D-19: Windows-Hinweis — bei Windows zeigen Berechtigungen immer "granted" */}
        {micPerm === 'granted' && axPerm === 'granted' && (
          <p className="text-xs text-green-400">Alle Berechtigungen erteilt.</p>
        )}
      </div>

      <div className="flex justify-between items-center">
        <button
          onClick={onBack}
          className="px-3 py-2 text-gray-400 hover:text-gray-200 text-sm transition-colors"
        >
          &larr; Zurueck
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
  );
}

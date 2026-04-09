// Speakly SettingsPage — Einstellungen in einer scrollbaren Seite (D-06)
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { register, unregister, isRegistered } from '@tauri-apps/plugin-global-shortcut';
import { useSettings } from '../hooks/useSettings';

type ValidationState = 'idle' | 'loading' | 'ok' | 'error';

export function SettingsPage() {
  const { settings, setSetting, loaded } = useSettings();

  // Lokale Zwischenwerte fuer API-Key-Eingabe (nicht jeder Tastendruck schreibt in Store)
  const [openaiInput, setOpenaiInput] = useState('');
  const [anthropicInput, setAnthropicInput] = useState('');
  const [openaiValid, setOpenaiValid] = useState<ValidationState>('idle');
  const [anthropicValid, setAnthropicValid] = useState<ValidationState>('idle');

  // Hotkey-Capture-State (D-08)
  const [isCapturingHotkey, setIsCapturingHotkey] = useState(false);
  const [hotkeyCaptured, setHotkeyCaptured] = useState('');
  const [hotkeyConflict, setHotkeyConflict] = useState(false);
  const [hotkeySaving, setHotkeySaving] = useState(false);

  // Download-State fuer STT-Sektion (D-10, D-12, D-13)
  const [downloadState, setDownloadState] = useState<
    | { status: 'idle' }
    | { status: 'downloading'; percent: number; downloadedMb: number; totalMb: number }
    | { status: 'done' }
    | { status: 'error'; message: string }
  >({ status: 'idle' });

  // Windows-Warning: einmal pro Session anzeigen (D-16)
  const [windowsWarningSeen, setWindowsWarningSeen] = useState(false);
  const [showWindowsWarning, setShowWindowsWarning] = useState(false);
  const [isWindows, setIsWindows] = useState(false);

  // Plattform-Erkennung per userAgent (plugin-os nicht installiert)
  useEffect(() => {
    const isWin = navigator.userAgent.toLowerCase().includes('windows');
    setIsWindows(isWin);
  }, []);

  // Fenster verbirgt sich beim Schliessen (D-07) — kein App-Quit
  useEffect(() => {
    const win = getCurrentWindow();
    let unlisten: (() => void) | null = null;
    win.onCloseRequested(async (event) => {
      event.preventDefault();
      await win.hide();
    }).then((fn) => { unlisten = fn; });
    return () => { if (unlisten) unlisten(); };
  }, []);

  // Sobald Store geladen — Felder mit gespeicherten Werten befuellen
  useEffect(() => {
    if (loaded) {
      setOpenaiInput(settings.openai_api_key);
      setAnthropicInput(settings.anthropic_api_key);
    }
  }, [loaded]);

  async function handleSttModeChange(newMode: string) {
    // Windows-Warning beim ersten Wechsel zu 'local' (D-15, D-16)
    if (isWindows && newMode === 'local' && !windowsWarningSeen) {
      setShowWindowsWarning(true);
      setWindowsWarningSeen(true);
    }

    await setSetting('stt_mode', newMode);

    // Modell-Check: invoke check_whisper_model_exists
    // Wenn kein Modell und newMode = 'local' oder 'auto' → Download starten
    if (newMode === 'local' || newMode === 'auto') {
      try {
        const exists = await invoke<boolean>('check_whisper_model_exists');
        if (!exists) {
          await startModelDownload();
        }
      } catch {
        // check_whisper_model_exists nicht vorhanden (Plan 02 registriert es) → kein Download
      }
    }
  }

  async function startModelDownload() {
    setDownloadState({ status: 'downloading', percent: 0, downloadedMb: 0, totalMb: 142 });

    // Download-Progress hoeren
    const unlistenProgress = await listen<{ percent: number; downloaded_mb: number; total_mb: number }>(
      'whisper_download_progress',
      (event) => {
        setDownloadState({
          status: 'downloading',
          percent: event.payload.percent,
          downloadedMb: event.payload.downloaded_mb,
          totalMb: event.payload.total_mb,
        });
      }
    );

    const unlistenComplete = await listen('whisper_download_complete', () => {
      setDownloadState({ status: 'done' });
      unlistenProgress();
      unlistenComplete();
    });

    const unlistenError = await listen<{ message: string }>('whisper_download_error', (e) => {
      setDownloadState({ status: 'error', message: e.payload.message });
      unlistenProgress();
      unlistenComplete();
    });

    // Download starten (feuert Events asynchron)
    invoke('download_whisper_model').catch((e: unknown) => {
      const msg = typeof e === 'string' ? e : 'Download fehlgeschlagen';
      setDownloadState({ status: 'error', message: msg });
      unlistenProgress();
      unlistenComplete();
      unlistenError();
    });
  }

  async function handleCancelDownload() {
    await invoke('cancel_whisper_download');
    setDownloadState({ status: 'idle' });
  }

  async function handleSaveOpenaiKey() {
    await setSetting('openai_api_key', openaiInput.trim());
  }

  async function handleSaveAnthropicKey() {
    await setSetting('anthropic_api_key', anthropicInput.trim());
  }

  async function handleTestOpenai() {
    setOpenaiValid('loading');
    await handleSaveOpenaiKey();
    try {
      // Netzwerkaufruf ueber Rust-Command (kein direktes fetch — CSP/CORS in Tauri)
      const ok = await invoke<boolean>('validate_openai_key', { apiKey: openaiInput.trim() });
      setOpenaiValid(ok ? 'ok' : 'error');
    } catch {
      setOpenaiValid('error');
    }
  }

  async function handleTestAnthropic() {
    setAnthropicValid('loading');
    await handleSaveAnthropicKey();
    try {
      const ok = await invoke<boolean>('validate_anthropic_key', { apiKey: anthropicInput.trim() });
      setAnthropicValid(ok ? 'ok' : 'error');
    } catch {
      setAnthropicValid('error');
    }
  }

  function validationLabel(state: ValidationState) {
    if (state === 'loading') return <span className="text-xs text-yellow-400">Prüfe...</span>;
    if (state === 'ok') return <span className="text-xs text-green-400">✓ Gültig</span>;
    if (state === 'error') return <span className="text-xs text-red-400">✗ Ungültig oder kein Netz</span>;
    return null;
  }

  // Konvertiert KeyboardEvent zu Tauri-Shortcut-String (T-02-05: Modifier-Pflicht)
  function keyEventToShortcut(e: React.KeyboardEvent<HTMLInputElement>): string | null {
    // Nur Modifier alleine ignorieren
    const ignoredKeys = ['Control', 'Shift', 'Alt', 'Meta', 'CapsLock', 'Tab'];
    if (ignoredKeys.includes(e.key)) return null;

    const parts: string[] = [];
    if (e.metaKey || e.ctrlKey) parts.push('CommandOrControl');
    if (e.shiftKey) parts.push('Shift');
    if (e.altKey) parts.push('Alt');

    // Leerzeichen → "Space", F-Tasten direkt, Buchstaben grossgeschrieben
    const key = e.key === ' ' ? 'Space' : e.key.length === 1 ? e.key.toUpperCase() : e.key;
    parts.push(key);

    // Muss mindestens einen Modifier enthalten (kein einzelner Buchstabe als globaler Hotkey)
    if (parts.length < 2) return null;
    return parts.join('+');
  }

  // Alten Hotkey freigeben, neuen registrieren — D-09: Konflikt warnen aber trotzdem speichern
  async function handleRemapHotkey(newHotkey: string) {
    setHotkeySaving(true);
    setHotkeyConflict(false);
    try {
      const oldHotkey = settings.hotkey;
      // Alten Hotkey freigeben, falls registriert
      if (await isRegistered(oldHotkey)) {
        await unregister(oldHotkey);
      }
      // Neuen Hotkey registrieren — Phase 3 ersetzt diesen leeren Handler
      await register(newHotkey, (shortcut) => {
        console.log('Speakly Hotkey (Phase 2 Stub):', shortcut);
      });
      await setSetting('hotkey', newHotkey);
    } catch (err) {
      // Konflikt — ein anderes Programm hat diesen Shortcut belegt
      // D-09: Warnung zeigen, aber trotzdem in Store schreiben
      console.warn('Speakly: Hotkey-Konflikt:', err);
      setHotkeyConflict(true);
      await setSetting('hotkey', newHotkey); // Trotzdem speichern (D-09)
    } finally {
      setHotkeySaving(false);
      setIsCapturingHotkey(false);
      setHotkeyCaptured('');
    }
  }

  if (!loaded) {
    return <div className="p-4 text-gray-400 text-sm">Lade Einstellungen...</div>;
  }

  return (
    <div className="h-full overflow-y-auto bg-gray-900 text-white p-4 space-y-6">
      <h1 className="text-base font-semibold text-white">Einstellungen</h1>

      {/* Hotkey-Sektion — D-08: Anzeige + Klick zum Aufnehmen, D-10: sofort wirksam */}
      <section id="hotkey-section">
        <h2 className="text-sm font-medium text-gray-300 mb-2">Tastenkuerzel</h2>
        <div className="space-y-2">
          <label className="text-xs text-gray-400 block">Globaler Hotkey (Aufnahme starten/stoppen)</label>
          {!isCapturingHotkey ? (
            <div className="flex items-center gap-2">
              <div className="flex-1 bg-gray-800 border border-gray-700 rounded px-2 py-1.5 text-xs text-white font-mono">
                {settings.hotkey}
              </div>
              <button
                onClick={() => { setIsCapturingHotkey(true); setHotkeyCaptured(''); setHotkeyConflict(false); }}
                className="text-xs px-2 py-1.5 bg-gray-700 hover:bg-gray-600 rounded transition-colors"
              >
                Aendern
              </button>
            </div>
          ) : (
            <div className="space-y-1">
              <input
                autoFocus
                readOnly
                value={hotkeyCaptured || 'Tastenkombination druecken...'}
                onKeyDown={(e) => {
                  e.preventDefault();
                  const shortcut = keyEventToShortcut(e);
                  if (shortcut) setHotkeyCaptured(shortcut);
                }}
                onKeyUp={async (e) => {
                  e.preventDefault();
                  if (hotkeyCaptured) {
                    await handleRemapHotkey(hotkeyCaptured);
                  }
                }}
                onBlur={() => { setIsCapturingHotkey(false); setHotkeyCaptured(''); }}
                className="w-full bg-gray-800 border border-blue-500 rounded px-2 py-1.5 text-xs text-blue-300 font-mono cursor-pointer focus:outline-none"
                placeholder="Tastenkombination druecken..."
              />
              <p className="text-xs text-gray-500">Druecke die neue Tastenkombination. ESC zum Abbrechen.</p>
              {hotkeySaving && <span className="text-xs text-yellow-400">Wird gespeichert...</span>}
            </div>
          )}
          {hotkeyConflict && (
            <p className="text-xs text-yellow-400">
              Hinweis: Dieser Hotkey ist moeglicherweise von einer anderen App belegt — er wurde trotzdem gespeichert. Falls er nicht funktioniert, waehle eine andere Kombination.
            </p>
          )}
        </div>
      </section>

      {/* API Keys — D-11: Passwort-maskiert, D-12: Testen-Button */}
      <section id="api-keys-section">
        <h2 className="text-sm font-medium text-gray-300 mb-3">API Keys</h2>

        <div className="space-y-4">
          {/* OpenAI Key */}
          <div>
            <label className="text-xs text-gray-400 block mb-1">OpenAI API Key (Whisper)</label>
            <div className="flex gap-2 items-center">
              <input
                type="password"
                value={openaiInput}
                onChange={(e) => { setOpenaiInput(e.target.value); setOpenaiValid('idle'); }}
                onBlur={handleSaveOpenaiKey}
                placeholder="sk-..."
                className="flex-1 bg-gray-800 border border-gray-700 rounded px-2 py-1.5 text-xs text-white placeholder-gray-600 focus:outline-none focus:border-gray-500"
              />
              <button
                onClick={handleTestOpenai}
                disabled={openaiValid === 'loading' || !openaiInput.trim()}
                className="text-xs px-2 py-1.5 bg-gray-700 hover:bg-gray-600 rounded disabled:opacity-40 transition-colors"
              >
                Testen
              </button>
            </div>
            <div className="mt-1 h-4">{validationLabel(openaiValid)}</div>
          </div>

          {/* Anthropic Key */}
          <div>
            <label className="text-xs text-gray-400 block mb-1">Anthropic API Key (Claude)</label>
            <div className="flex gap-2 items-center">
              <input
                type="password"
                value={anthropicInput}
                onChange={(e) => { setAnthropicInput(e.target.value); setAnthropicValid('idle'); }}
                onBlur={handleSaveAnthropicKey}
                placeholder="sk-ant-..."
                className="flex-1 bg-gray-800 border border-gray-700 rounded px-2 py-1.5 text-xs text-white placeholder-gray-600 focus:outline-none focus:border-gray-500"
              />
              <button
                onClick={handleTestAnthropic}
                disabled={anthropicValid === 'loading' || !anthropicInput.trim()}
                className="text-xs px-2 py-1.5 bg-gray-700 hover:bg-gray-600 rounded disabled:opacity-40 transition-colors"
              >
                Testen
              </button>
            </div>
            <div className="mt-1 h-4">{validationLabel(anthropicValid)}</div>
          </div>
        </div>
      </section>

      {/* Sprache-Sektion — D-09: language Dropdown mit "auto" als Standard (APP-03) */}
      <section id="language-section">
        <h2 className="text-sm font-medium text-gray-300 mb-2">Sprache</h2>
        <div className="space-y-2">
          <label className="text-xs text-gray-400 block">Transkriptionssprache (Whisper)</label>
          <select
            value={settings.language}
            onChange={async (e) => { await setSetting('language', e.target.value); }}
            className="w-full bg-gray-800 border border-gray-700 rounded px-2 py-1.5 text-xs text-white focus:outline-none focus:border-gray-500"
          >
            <option value="auto">Automatisch erkennen</option>
            <option value="de">Deutsch</option>
            <option value="en">Englisch</option>
            <option value="fr">Franzoesisch</option>
            <option value="es">Spanisch</option>
            <option value="it">Italienisch</option>
            <option value="pt">Portugiesisch</option>
            <option value="nl">Niederlaendisch</option>
            <option value="pl">Polnisch</option>
            <option value="ru">Russisch</option>
            <option value="zh">Chinesisch</option>
            <option value="ja">Japanisch</option>
            <option value="ko">Koreanisch</option>
            <option value="ar">Arabisch</option>
            <option value="tr">Tuerkisch</option>
          </select>
          <p className="text-xs text-gray-500">
            "Automatisch" erkennt die Sprache selbst — empfohlen fuer mehrsprachige Nutzung.
          </p>
        </div>
      </section>

      {/* STT-Engine-Sektion — D-01, D-02, D-05 */}
      <section id="stt-engine-section">
        <h2 className="text-sm font-medium text-gray-300 mb-2">Transkriptions-Engine</h2>
        <div className="space-y-3">
          <div>
            <label className="text-xs text-gray-400 block mb-1">STT-Engine</label>
            <select
              value={settings.stt_mode ?? 'cloud'}
              onChange={async (e) => { await handleSttModeChange(e.target.value); }}
              className="w-full bg-gray-800 border border-gray-700 rounded px-2 py-1.5 text-xs text-white focus:outline-none focus:border-gray-500"
            >
              <option value="cloud">Cloud (Whisper API) — schnell, benoetigt Internet</option>
              <option value="local">Lokal (whisper.cpp) — offline, langsamer</option>
              <option value="auto">Auto (lokal wenn offline)</option>
            </select>
          </div>

          {/* Windows Speed Warning (D-15, D-16) */}
          {showWindowsWarning && (
            <div className="bg-yellow-900/30 border border-yellow-600 rounded p-2">
              <p className="text-xs text-yellow-300">
                Lokale Transkription kann auf diesem Geraet langsam sein (10-30x langsamer als Cloud).
              </p>
              <button
                onClick={() => setShowWindowsWarning(false)}
                className="mt-1 text-xs text-yellow-400 hover:text-yellow-200"
              >
                Verstanden
              </button>
            </div>
          )}

          {/* Download-Progress (D-12) */}
          {downloadState.status === 'downloading' && (
            <div className="space-y-1">
              <div className="flex justify-between text-xs text-gray-400">
                <span>Whisper-Modell herunterladen...</span>
                <span>{downloadState.downloadedMb.toFixed(1)} / {downloadState.totalMb.toFixed(0)} MB</span>
              </div>
              <div className="w-full bg-gray-700 rounded-full h-1.5">
                <div
                  className="bg-blue-500 h-1.5 rounded-full transition-all"
                  style={{ width: `${downloadState.percent}%` }}
                />
              </div>
              <div className="flex justify-between items-center">
                <span className="text-xs text-gray-500">{downloadState.percent.toFixed(0)}%</span>
                <button
                  onClick={handleCancelDownload}
                  className="text-xs text-red-400 hover:text-red-300"
                >
                  Abbrechen
                </button>
              </div>
            </div>
          )}

          {downloadState.status === 'done' && (
            <p className="text-xs text-green-400">Modell erfolgreich heruntergeladen.</p>
          )}

          {downloadState.status === 'error' && (
            <div>
              <p className="text-xs text-red-400">{downloadState.message}</p>
              <button
                onClick={startModelDownload}
                className="mt-1 text-xs text-blue-400 hover:text-blue-300"
              >
                Erneut versuchen
              </button>
            </div>
          )}
        </div>
      </section>

      {/* Mikrofon-Sektion — ausgefuellt in Plan 04 */}
      <section id="microphone-section">
        <h2 className="text-sm font-medium text-gray-300 mb-2">Mikrofon</h2>
        <p className="text-xs text-gray-500">Wird in Plan 04 implementiert.</p>
      </section>

      {/* Berechtigungen-Sektion — ausgefuellt in Plan 04 */}
      <section id="permissions-section">
        <h2 className="text-sm font-medium text-gray-300 mb-2">Berechtigungen</h2>
        <p className="text-xs text-gray-500">Wird in Plan 04 implementiert.</p>
      </section>
    </div>
  );
}

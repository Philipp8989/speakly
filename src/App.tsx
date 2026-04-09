// Speakly App Root — Phase 1
// Initialisiert ConfigStore und registriert den Standard-Hotkey.

import { useEffect } from 'react';
import { load } from '@tauri-apps/plugin-store';
import { register } from '@tauri-apps/plugin-global-shortcut';
import { TrayPopup } from './components/TrayPopup';

const DEFAULT_HOTKEY = 'CommandOrControl+Shift+Space';

async function initStore() {
  // Einstellungen laden — autoSave: true schreibt bei jedem set() automatisch
  const store = await load('settings.json', { defaults: {}, autoSave: true });

  // Standardwerte beim ersten Start setzen (Rust-Seite setzt sie auch, Frontend ist idempotent)
  const initialized = await store.get<boolean>('initialized');
  if (!initialized) {
    await store.set('hotkey', DEFAULT_HOTKEY);
    await store.set('stt_mode', 'cloud');
    await store.set('initialized', true);
    console.log('Speakly: Erststart — Standardeinstellungen gesetzt');
  }
}

async function registerHotkey() {
  try {
    // Hotkey registrieren — Phase 1: Nur Stub, kein Handler
    // Phase 3 ersetzt diesen Handler durch die Aufnahme-Logik
    await register(DEFAULT_HOTKEY, (shortcut) => {
      console.log('Speakly Hotkey ausgeloest (Phase 1 Stub):', shortcut);
    });
    console.log('Speakly: Hotkey registriert:', DEFAULT_HOTKEY);
  } catch (err) {
    // Hotkey-Konflikt: Ein anderes Programm hat diesen Shortcut bereits belegt
    console.warn('Speakly: Hotkey-Registrierung fehlgeschlagen (Konflikt?):', err);
  }
}

function App() {
  useEffect(() => {
    // Einmalige Initialisierung beim App-Start
    initStore().catch(console.error);
    registerHotkey().catch(console.error);
  }, []);

  function handleSettingsClick() {
    // Phase 2: Oeffnet das Settings-Fenster
    // In Phase 1 nur ein Platzhalter
    console.log('Speakly: Settings-Fenster (Phase 2)');
  }

  return (
    <div className="w-full h-screen overflow-hidden">
      <TrayPopup onSettingsClick={handleSettingsClick} />
    </div>
  );
}

export default App;

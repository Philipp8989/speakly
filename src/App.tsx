// Speakly App Root — Phase 2 + Phase 3
// Routet zwischen Tray-Popup und Settings/Onboarding-Fenster.
// Phase 1: ConfigStore-Init, TrayPopup
// Phase 2: Settings-Fenster, Onboarding-Pruefung, URL-Routing
// Phase 3: Hotkey vollstaendig in Rust (kein JS-Stub), Aufnahme-Event-Listener

import { useEffect } from 'react';
import { load } from '@tauri-apps/plugin-store';
import { listen } from '@tauri-apps/api/event';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { TrayPopup } from './components/TrayPopup';
import { SettingsPage } from './components/SettingsPage';
import { OnboardingWizard } from './components/onboarding/OnboardingWizard';

const DEFAULT_HOTKEY = 'CommandOrControl+Shift+Space';

// Verhindert Doppel-Oeffnen wenn Klick schnell doppelt ausgeloest wird (Pitfall 1)
let isOpeningSettings = false;

async function initStore() {
  // Einstellungen laden — autoSave: true schreibt bei jedem set() automatisch
  const store = await load('settings.json', { defaults: {}, autoSave: true });

  // Standardwerte beim ersten Start setzen (Rust-Seite setzt sie auch, Frontend ist idempotent)
  const initialized = await store.get<boolean>('initialized');
  if (!initialized) {
    await store.set('hotkey', DEFAULT_HOTKEY);
    await store.set('stt_mode', 'cloud');
    await store.set('openai_api_key', '');
    await store.set('anthropic_api_key', '');
    await store.set('microphone_id', 'default');
    await store.set('language', 'auto');
    await store.set('onboarding_complete', false);
    await store.set('initialized', true);
    console.log('Speakly: Erststart — Standardeinstellungen gesetzt');
  }
}

// Settings-Fenster oeffnen — 500x600, zentriert, nur einmal (D-05, D-07)
async function openSettingsWindow(mode: 'settings' | 'onboarding' = 'settings') {
  // Doppel-Klick-Schutz: kein zweites Fenster waehrend erstes noch laedt
  if (isOpeningSettings) return;

  // Vorhandenes Fenster focussieren statt neues oeffnen (Pitfall 1)
  const existing = await WebviewWindow.getByLabel('settings');
  if (existing) {
    await existing.show();
    await existing.setFocus();
    return;
  }

  isOpeningSettings = true;

  const win = new WebviewWindow('settings', {
    title: 'Speakly Einstellungen',
    url: `index.html?window=settings&mode=${mode}`,
    width: 500,
    height: 600,
    center: true,
    resizable: false,
    decorations: true,
    closable: true,
  });

  win.once('tauri://created', () => {
    isOpeningSettings = false;
    console.log('Speakly: Settings-Fenster erstellt, Modus:', mode);
  });
  win.once('tauri://error', (e) => {
    isOpeningSettings = false;
    console.error('Speakly: Settings-Fenster Fehler:', e);
  });
}

// Onboarding pruefen — oeffnet Wizard wenn Erststart (D-03)
async function checkOnboarding() {
  const store = await load('settings.json', { defaults: {}, autoSave: true });
  const done = await store.get<boolean>('onboarding_complete');
  if (!done) {
    // Kurze Verzoegerung damit Tray-Fenster zuerst erscheint
    setTimeout(() => openSettingsWindow('onboarding'), 500);
  }
}

// URL-Routing: Welches Fenster/Modus rendert diese Instanz?
// main-Fenster: kein ?window-Parameter → TrayPopup
// settings-Fenster: ?window=settings&mode=onboarding → OnboardingWizard
// settings-Fenster: ?window=settings&mode=settings → SettingsPage
function getWindowMode(): { isSettings: boolean; mode: string } {
  const params = new URLSearchParams(window.location.search);
  const windowParam = params.get('window');
  const mode = params.get('mode') ?? 'settings';
  return { isSettings: windowParam === 'settings', mode };
}

function App() {
  const { isSettings, mode } = getWindowMode();

  useEffect(() => {
    if (!isSettings) {
      // Haupt-Fenster: Store initialisieren, Onboarding pruefen
      // Hotkey wird vollstaendig in Rust verwaltet (Phase 3) — kein JS-Stub noetig
      initStore()
        .then(() => checkOnboarding())
        .catch(console.error);

      // Phase 3: Aufnahme-Status Events empfangen
      const unlisteners: Array<() => void> = [];

      listen<{ recording: boolean; mode: string }>('recording_state_changed', (event) => {
        console.log('Speakly: Aufnahme-Status', event.payload);
        // Plan 03-03 verbindet diesen Event mit der UI
      }).then(ul => unlisteners.push(ul));

      listen<{ reason: string; duration_ms: number }>('recording_discarded', (event) => {
        console.log('Speakly: Aufnahme verworfen —', event.payload.reason);
      }).then(ul => unlisteners.push(ul));

      // Cleanup bei Unmount
      return () => { unlisteners.forEach(ul => ul()); };
    }
  }, [isSettings]);

  // Settings-Fenster: Onboarding oder Settings rendern
  if (isSettings) {
    return (
      <div className="w-full h-screen overflow-hidden">
        {mode === 'onboarding' ? <OnboardingWizard /> : <SettingsPage />}
      </div>
    );
  }

  // Haupt-Fenster: TrayPopup
  return (
    <div className="w-full h-screen overflow-hidden">
      <TrayPopup onSettingsClick={() => openSettingsWindow('settings')} />
    </div>
  );
}

export default App;

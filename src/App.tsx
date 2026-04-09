// Speakly App Root — Phase 2 + Phase 3 + Phase 4
// Routet zwischen Tray-Popup und Settings/Onboarding-Fenster.
// Phase 1: ConfigStore-Init, TrayPopup
// Phase 2: Settings-Fenster, Onboarding-Pruefung, URL-Routing
// Phase 3: Hotkey vollstaendig in Rust (kein JS-Stub), Aufnahme-Event-Listener
// Phase 4: transcribe_and_inject invoke, transcription_state_changed Listener, AppState 5-State-Machine

import { useEffect, useState } from 'react';
import { load } from '@tauri-apps/plugin-store';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { TrayPopup, AppState } from './components/TrayPopup';
import { SettingsPage } from './components/SettingsPage';
import { OnboardingWizard } from './components/onboarding/OnboardingWizard';

const DEFAULT_HOTKEY = 'CommandOrControl+Shift+Space';

// Verhindert Doppel-Oeffnen wenn Klick schnell doppelt ausgeloest wird (Pitfall 1)
let isOpeningSettings = false;

// Payload-Typ fuer transcription_state_changed Event
type TranscriptionStatePayload =
  | { state: 'processing' }
  | { state: 'done'; text: string }
  | { state: 'error'; message: string };

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

  // AppState: 5-State-Machine — idle | recording | processing | transcript | error (Phase 4)
  const [appState, setAppState] = useState<AppState>({ kind: 'idle' });

  useEffect(() => {
    if (!isSettings) {
      // Haupt-Fenster: Store initialisieren, Onboarding pruefen
      // Hotkey wird vollstaendig in Rust verwaltet (Phase 3) — kein JS-Stub noetig
      initStore()
        .then(() => checkOnboarding())
        .catch(console.error);

      // Phase 3 + Phase 4: Aufnahme- und Transkriptions-Events empfangen
      const unlisteners: Array<() => void> = [];

      // Aufnahme-Status → AppState aktualisieren (Phase 4: vollstaendige State-Machine)
      listen<{ recording: boolean; mode: string }>('recording_state_changed', (event) => {
        console.log('Speakly: Aufnahme-Status', event.payload);
        setAppState(event.payload.recording ? { kind: 'recording' } : { kind: 'idle' });
      }).then(ul => unlisteners.push(ul));

      // Aufnahme verworfen (VAD) — kein STT-Aufruf (D-24)
      listen<{ reason: string; duration_ms: number }>('recording_discarded', (event) => {
        console.log('Speakly: Aufnahme verworfen —', event.payload.reason);
        // Kein transcribe_and_inject — State bleibt idle
      }).then(ul => unlisteners.push(ul));

      // Aufnahme gestoppt mit WAV-Buffer — transcribe_and_inject aufrufen wenn wav_ready=true (D-24)
      listen<{ wav_ready: boolean; duration_ms: number }>('recording_stopped', (event) => {
        console.log('Speakly: Aufnahme gestoppt, wav_ready:', event.payload.wav_ready);
        if (event.payload.wav_ready) {
          invoke('transcribe_and_inject').catch((err) => {
            console.error('Speakly: transcribe_and_inject Fehler:', err);
          });
        }
        // wav_ready=false bedeutet VAD hat verworfen — kein API-Aufruf (D-24)
      }).then(ul => unlisteners.push(ul));

      // Transkriptions-Status-Events vom Backend empfangen (D-19, D-21, D-22)
      listen<TranscriptionStatePayload>('transcription_state_changed', (event) => {
        const p = event.payload;
        if (p.state === 'processing') {
          // D-19: "Verarbeite..." waehrend Whisper API-Aufruf
          setAppState({ kind: 'processing' });
        } else if (p.state === 'done') {
          // D-21: Transkription kurz anzeigen, dann nach 2 Sekunden zurueck zu idle
          setAppState({ kind: 'transcript', text: p.text });
          setTimeout(() => setAppState({ kind: 'idle' }), 2000);
        } else if (p.state === 'error') {
          // D-22: Fehler 5 Sekunden anzeigen, dann zurueck zu idle
          setAppState({ kind: 'error', message: p.message });
          setTimeout(() => setAppState({ kind: 'idle' }), 5000);
        }
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

  // Haupt-Fenster: TrayPopup mit AppState
  return (
    <div className="w-full h-screen overflow-hidden">
      <TrayPopup
        onSettingsClick={() => openSettingsWindow('settings')}
        appState={appState}
      />
    </div>
  );
}

export default App;

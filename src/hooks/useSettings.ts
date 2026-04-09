// Speakly Settings Hook — liest und schreibt alle Einstellungen via plugin-store
import { useState, useEffect } from 'react';
import { load, Store } from '@tauri-apps/plugin-store';

export interface Settings {
  hotkey: string;
  stt_mode: string;
  openai_api_key: string;
  anthropic_api_key: string;
  microphone_id: string;
  language: string;
  onboarding_complete: boolean;
}

const DEFAULTS: Settings = {
  hotkey: 'CommandOrControl+Shift+Space',
  stt_mode: 'cloud',
  openai_api_key: '',
  anthropic_api_key: '',
  microphone_id: 'default',
  language: 'auto',
  onboarding_complete: false,
};

export function useSettings() {
  const [settings, setSettings] = useState<Settings>(DEFAULTS);
  const [store, setStore] = useState<Store | null>(null);
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    // Einstellungen beim ersten Render laden
    load('settings.json', { autoSave: true }).then(async (s) => {
      const result: Settings = { ...DEFAULTS };
      for (const key of Object.keys(DEFAULTS) as (keyof Settings)[]) {
        const val = await s.get<Settings[typeof key]>(key);
        if (val !== null && val !== undefined) {
          (result[key] as Settings[typeof key]) = val;
        }
      }
      setStore(s);
      setSettings(result);
      setLoaded(true);
    }).catch(console.error);
  }, []);

  // Einzelnen Einstellungswert setzen und sofort in Store schreiben
  async function setSetting<K extends keyof Settings>(key: K, value: Settings[K]) {
    if (!store) return;
    await store.set(key, value);
    setSettings(prev => ({ ...prev, [key]: value }));
  }

  return { settings, setSetting, loaded };
}

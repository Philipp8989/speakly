// Speakly Onboarding Wizard — 4-Schritte Einrichtungsassistent (D-01, D-02)
import { useState } from 'react';
import { StepWelcome } from './StepWelcome';
import { StepApiKeys } from './StepApiKeys';
import { StepMicrophone } from './StepMicrophone';
import { StepDone } from './StepDone';
import { getCurrentWindow } from '@tauri-apps/api/window';

const TOTAL_STEPS = 4;

export function OnboardingWizard() {
  const [step, setStep] = useState(0);

  // Fenster beim Schliessen verbergen (nicht beenden) — D-07
  // Gleiche Logik wie in SettingsPage
  useState(() => {
    const win = getCurrentWindow();
    win.onCloseRequested(async (event) => {
      event.preventDefault();
      await win.hide();
    });
  });

  const goNext = () => setStep((s) => Math.min(s + 1, TOTAL_STEPS - 1));
  const goBack = () => setStep((s) => Math.max(s - 1, 0));

  return (
    <div className="flex flex-col h-full bg-gray-900 text-white">
      {/* Fortschrittsanzeige */}
      <div className="flex gap-1 p-3 pb-0">
        {Array.from({ length: TOTAL_STEPS }).map((_, i) => (
          <div
            key={i}
            className={`h-1 flex-1 rounded-full transition-colors ${i <= step ? 'bg-blue-500' : 'bg-gray-700'}`}
          />
        ))}
      </div>

      {/* Aktueller Schritt */}
      <div className="flex-1 overflow-y-auto">
        {step === 0 && <StepWelcome onNext={goNext} />}
        {step === 1 && <StepApiKeys onNext={goNext} onBack={goBack} />}
        {step === 2 && <StepMicrophone onNext={goNext} onBack={goBack} />}
        {step === 3 && <StepDone />}
      </div>
    </div>
  );
}

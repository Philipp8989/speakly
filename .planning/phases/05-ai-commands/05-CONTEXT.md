# Phase 5: AI Commands - Context

**Gathered:** 2026-04-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Add AI text reformulation commands using Claude API. After transcription, user can choose to reformulate text (formal, shorter, translate, etc.) before it gets pasted. Uses the user's own Anthropic API key.

</domain>

<decisions>
## Implementation Decisions

### Command Flow
- **D-01:** After transcription completes, popup shows the transcript text with a row of AI command buttons below it.
- **D-02:** User can either click an AI command button (reformulate then paste) OR click "Einfuegen" to paste raw transcript directly.
- **D-03:** If no Anthropic API key is set, AI command buttons are disabled/greyed out with tooltip "API-Key in Einstellungen eingeben".
- **D-04:** Auto-dismiss: if user doesn't interact within 10 seconds, raw transcript is pasted automatically (no reformulation).

### Available Commands
- **D-05:** Preset commands: "Formeller", "Kuerzer", "Uebersetzen EN→DE", "Uebersetzen DE→EN", "Email", "Slack"
- **D-06:** Each command maps to a system prompt sent to Claude API along with the transcript text.
- **D-07:** Command prompts are hardcoded in v1 (no custom commands yet). Stored as a const map in Rust.

### Claude API Integration
- **D-08:** Use Anthropic Messages API (POST /v1/messages) with model claude-3-5-haiku-latest (fast + cheap for short reformulation).
- **D-09:** Send from Rust via reqwest. API key from ConfigStore `anthropic_api_key`.
- **D-10:** System prompt per command, user message = raw transcript. Max tokens: 4096.
- **D-11:** On API error, show error in popup and paste raw transcript as fallback.
- **D-12:** Timeout: 15 seconds. On timeout, paste raw transcript.

### UI/UX
- **D-13:** Command buttons are compact pill-shaped buttons in a horizontal row below the transcript.
- **D-14:** While reformulating, show "Formuliere um..." with spinner, buttons disabled.
- **D-15:** After reformulation, show the new text briefly (2s), then auto-paste and return to idle.
- **D-16:** User can cancel reformulation by pressing hotkey again (returns to idle without pasting).

### Claude's Discretion
- Exact system prompts for each command
- Button styling and layout details
- Whether to show both original and reformulated text side-by-side

</decisions>

<canonical_refs>
## Canonical References

### Existing Code
- `src-tauri/src/stt/mod.rs` — Whisper API pattern (similar reqwest multipart pattern for Claude)
- `src-tauri/src/lib.rs` — Tauri commands, managed state
- `src/components/TrayPopup.tsx` — 5-state machine to extend with command selection state
- `src/App.tsx` — Event listeners

### Research
- `.planning/research/STACK.md` — Claude API (Haiku) for reformulation

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `reqwest` already in Cargo.toml — reuse for Claude API calls
- TrayPopup has transcript preview state — extend with command buttons
- ConfigStore has `anthropic_api_key` — already stored from Phase 2
- `inject_text()` in inject/mod.rs — reuse for pasting reformulated text

### Integration Points
- After STT returns transcript → show in popup WITH command buttons (new state)
- On command click → call Claude API → replace transcript → paste
- On "Einfuegen" click or timeout → paste raw transcript

</code_context>

<specifics>
## Specific Ideas

No specific requirements beyond the decisions above.

</specifics>

<deferred>
## Deferred Ideas

- Custom user-defined commands — v2 feature
- Command history — not needed in v1

</deferred>

---

*Phase: 05-ai-commands*
*Context gathered: 2026-04-09*

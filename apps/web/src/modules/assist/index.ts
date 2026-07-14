// V7 Assist (AI) module (SPEC §14, plan §2.6 / §3 e6). SCAFFOLD stub (e0):
// inert, lazily importable, typecheck-green, NOT routed. e6 fills the chat panel,
// composer grammar/rewrite/tone/translate, dictation, semantic-search toggle,
// auto-tag suggest/audit, and the "what left the device" disclosure; e14 wires it
// to /api/assist/*. This module does NOT touch the router or Settings.tsx.
//
// HARD RULE (§14, R4): the whole Assist UI is HIDDEN when the gateway is Disabled,
// and NO Assist path ever transmits/deletes/accepts — send is always human-gated.

/** The Assist capabilities the UI can surface (mirrors `mw_assist::AssistCapability`). */
export type AssistCapability =
  | 'summarize'
  | 'draft'
  | 'grammar'
  | 'dictation'
  | 'search-semantic'
  | 'auto-tag'
  | 'recap'
  | 'assistant';

/** Gateway availability. When 'disabled', the web hides ALL Assist UI (§14). */
export type AssistAvailability = 'disabled' | 'enabled';

/**
 * The honest "what left the device" disclosure (SPEC §14, plan §1.5 / R4). Shown
 * per-message so the user always knows what, if anything, was sent to an endpoint.
 * E2EE-decrypted content and attachments are EXCLUDED by default.
 */
export const WHAT_LEFT_THE_DEVICE: readonly string[] = [
  'the selected message text (subject + body) for the chosen capability',
  'the endpoint host it was sent to',
  'never: end-to-end-encrypted content (excluded by default)',
  'never: attachments (excluded by default)',
  'never: your credentials or other accounts',
];

export { AssistPanel } from './AssistPanel.tsx';

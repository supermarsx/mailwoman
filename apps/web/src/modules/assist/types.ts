// V7 Assist (AI) UI types (SPEC §14, plan §2.6 / §3 e6). The UI model mirrors the
// frozen `mw_assist` surface (§2.4): capabilities, data-class scope, the
// content-free audit, and the `Disabled`-when-unconfigured contract.
//
// TWO HARD, SAFETY-CRITICAL RULES bake into these types (R4):
//   1. NO Assist path transmits / deletes / accepts. There is NO `send` capability
//      and no service method that sends — proposed actions are ALWAYS routed back
//      to the human (Outbox / composer), never executed here.
//   2. When the gateway is `disabled`, the web hides ALL Assist UI.

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

/** Every capability, in display order. NOTE: no `send`/`delete`/`accept` exists. */
export const ASSIST_CAPABILITIES: readonly AssistCapability[] = [
  'summarize',
  'draft',
  'grammar',
  'dictation',
  'search-semantic',
  'auto-tag',
  'recap',
  'assistant',
];

/** Gateway availability. When 'disabled', the web hides ALL Assist UI (§14). */
export type AssistAvailability = 'disabled' | 'enabled';

/**
 * The gateway config the web reads at boot (`GET /api/assist/config`). When
 * `availability === 'disabled'` (gateway unconfigured or admin kill switch), the
 * capability list is empty and NO Assist UI renders.
 */
export interface AssistConfig {
  readonly availability: AssistAvailability;
  /** Which capabilities this user is actually granted (admin-scoped). */
  readonly capabilities: readonly AssistCapability[];
  /** The endpoint host content would be proxied to (for the disclosure). Null when disabled. */
  readonly endpointHost: string | null;
  /** Whether E2EE-decrypted content is permitted to leave (admin ceiling; default false). */
  readonly includeE2ee: boolean;
  /** Whether attachments are permitted to leave (admin ceiling; default false). */
  readonly includeAttachments: boolean;
}

/** The safe default config: everything off ⇒ the whole Assist UI is hidden. */
export const DISABLED_CONFIG: AssistConfig = {
  availability: 'disabled',
  capabilities: [],
  endpointHost: null,
  includeE2ee: false,
  includeAttachments: false,
};

/** The wire shape `GET /api/assist/config` returns (snake_case; e9 satisfies). */
export interface WireAssistConfig {
  availability: AssistAvailability;
  capabilities: AssistCapability[];
  endpoint_host: string | null;
  include_e2ee: boolean;
  include_attachments: boolean;
}

export function configFromWire(wire: WireAssistConfig): AssistConfig {
  return {
    availability: wire.availability,
    capabilities: [...wire.capabilities],
    endpointHost: wire.endpoint_host,
    includeE2ee: wire.include_e2ee,
    includeAttachments: wire.include_attachments,
  };
}

/** Does the user actually have `cap`, on an enabled gateway? */
export function hasCapability(config: AssistConfig, cap: AssistCapability): boolean {
  return config.availability === 'enabled' && config.capabilities.includes(cap);
}

// ── Invoke (the one path context can leave the device) ─────────────────────────

/** How the mailbox context is tagged so redaction (server-side) can honour ceilings. */
export type ContentKind = 'plain' | 'e2ee' | 'attachment';

/** A piece of mailbox context handed to a capability. */
export interface ContextItem {
  readonly account: string;
  readonly folder: string;
  readonly text: string;
  readonly kind: ContentKind;
}

/** A capability invocation (`POST /api/assist/invoke`). Never a send. */
export interface InvokeRequest {
  readonly capability: AssistCapability;
  readonly prompt: string;
  readonly context: readonly ContextItem[];
}

/**
 * What actually left the device on a single invocation — surfaced verbatim in the
 * per-message "what left the device" disclosure so the claim is honest, not vague.
 */
export interface Disclosure {
  /** The endpoint host the proxied request reached (never the browser directly). */
  readonly endpointHost: string;
  /** Human-readable summary of the content classes that were forwarded. */
  readonly sent: readonly string[];
  /** Content classes that were withheld (E2EE / attachments by default). */
  readonly withheld: readonly string[];
}

/**
 * A tool action the assistant PROPOSES. It is NEVER executed by the Assist UI.
 * The user reviews it; anything that would transmit routes to the Outbox for the
 * in-app human confirmation the whole product guarantees (§14, §20.3).
 */
export interface ProposedAction {
  readonly id: string;
  readonly tool: string;
  /** Human summary of what the tool would do. */
  readonly summary: string;
  /** True when confirming this proposal would eventually enqueue a send (Outbox-gated). */
  readonly wouldSend: boolean;
}

/**
 * The result of an invocation: the model text + the honest disclosure, plus any
 * tool actions the assistant PROPOSED (assistant capability only). Proposed
 * actions are shown for human review — the UI never executes them.
 */
export interface InvokeResult {
  readonly text: string;
  readonly disclosure: Disclosure;
  readonly actions: readonly ProposedAction[];
}

// ── Chat (assistant capability — same tool surface as MCP) ─────────────────────

export type ChatRole = 'user' | 'assistant';

export interface ChatMessage {
  readonly id: string;
  readonly role: ChatRole;
  readonly text: string;
  readonly actions?: readonly ProposedAction[];
}

// ── Composer tools (grammar / rewrite / tone / translate) ──────────────────────

/** The inline composer transforms (§14.3). Each maps to a capability. */
export type ComposerTool = 'grammar' | 'rewrite' | 'tone' | 'translate';

export interface ComposerToolSpec {
  readonly id: ComposerTool;
  readonly label: string;
  readonly capability: AssistCapability;
  /** Whether the tool needs an argument (tone target / target language). */
  readonly arg?: { readonly label: string; readonly options: readonly string[] };
}

export const COMPOSER_TOOLS: readonly ComposerToolSpec[] = [
  { id: 'grammar', label: 'Fix grammar', capability: 'grammar' },
  { id: 'rewrite', label: 'Rewrite', capability: 'draft' },
  {
    id: 'tone',
    label: 'Adjust tone',
    capability: 'draft',
    arg: { label: 'Tone', options: ['Neutral', 'Friendly', 'Formal', 'Concise', 'Assertive'] },
  },
  {
    id: 'translate',
    label: 'Translate',
    capability: 'draft',
    arg: { label: 'Language', options: ['English', 'German', 'French', 'Spanish', 'Portuguese', 'Dutch'] },
  },
];

// ── Auto-tag (suggest-mode by default; auto-mode is opt-in) ────────────────────

export type AutoTagMode = 'suggest' | 'auto';

/** A single tag the model proposes for a message. */
export interface TagSuggestion {
  readonly keyword: string;
  readonly label: string;
  /** Model confidence 0..1 (display only). */
  readonly confidence: number;
}

/** An entry in the auto-tag audit trail (suggested → applied/reverted, by whom). */
export interface TagAuditEntry {
  readonly id: string;
  readonly messageId: string;
  readonly keyword: string;
  readonly action: 'suggested' | 'applied' | 'reverted';
  /** 'assist' for auto-mode, 'user' when a person clicked apply. */
  readonly actor: 'assist' | 'user';
  readonly ts: string;
}

// ── The honest "what left the device" disclosure (SPEC §14, plan §1.5 / R4) ─────

/**
 * The static, always-visible summary of what Assist can and cannot send. Shown
 * per-message so the user always knows what, if anything, left the device.
 * E2EE-decrypted content and attachments are EXCLUDED by default.
 */
export const WHAT_LEFT_THE_DEVICE: readonly string[] = [
  'the selected message text (subject + body) for the chosen capability',
  'the endpoint host it was sent to',
  'never: end-to-end-encrypted content (excluded by default)',
  'never: attachments (excluded by default)',
  'never: your credentials or other accounts',
];

/**
 * Build the honest disclosure sentence for a given config, spelling out the
 * endpoint host and the ceilings actually in force.
 */
export function disclosureSentence(config: AssistConfig): string {
  if (config.availability !== 'enabled' || config.endpointHost === null) {
    return 'Assist is off. No message content leaves this device.';
  }
  const excluded: string[] = [];
  if (!config.includeE2ee) excluded.push('end-to-end-encrypted content');
  if (!config.includeAttachments) excluded.push('attachments');
  const exclNote =
    excluded.length > 0 ? ` It never sends ${excluded.join(' or ')}.` : ' Your admin has allowed encrypted content and attachments to be sent.';
  return `When you use an Assist tool, the selected message text is proxied to ${config.endpointHost}.${exclNote} Send is never automated — you always confirm before anything leaves your Outbox.`;
}

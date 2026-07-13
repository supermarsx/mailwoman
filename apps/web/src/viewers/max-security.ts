// Max-security opening mode — policy + precedence + render plan (plan §3 e5, §7.2).
//
// The Reader toolbar carries a three-position switch (plain-text /
// sanitized-no-media / full-sanitized). This module owns the LOGIC behind it:
//   • the effective-mode resolution with precedence  admin-floor > per-sender >
//     global default  (the admin floor is a *minimum* security level — it can
//     only make a message more locked-down, never less);
//   • a per-sender + global policy store (persisted in localStorage — the same
//     pattern the tags registry uses; the engine takes it over later);
//   • the render plan e8 wires into the body path (which CSP, whether HTML is
//     rendered, what the wasm sanitizer should strip);
//   • the attachment-gating predicate e8 wires into the Reader attachment open
//     path (in a locked-down mode attachments open ONLY through the re-encode
//     preview jail — the existing viewer sandbox — never as original bytes).
//
// The CSP/`srcdoc` builders live in `sandbox.ts`; this module never renders.

import { createSignal, type Accessor } from 'solid-js';
import { bodyCsp, type SecurityMode } from './sandbox.ts';

export type { SecurityMode } from './sandbox.ts';

/** The switch positions, ordered as shown left→right (least→most locked-down). */
export const SECURITY_MODES: readonly SecurityMode[] = [
  'full-sanitized',
  'sanitized-no-media',
  'plain-text',
];

/** Human labels for the switch / settings UI. */
export const SECURITY_MODE_LABELS: Record<SecurityMode, string> = {
  'full-sanitized': 'Full',
  'sanitized-no-media': 'No media',
  'plain-text': 'Plain text',
};

/** One-line explanations (tooltip / a11y description). */
export const SECURITY_MODE_HINTS: Record<SecurityMode, string> = {
  'full-sanitized': 'Sanitized HTML with images and styling.',
  'sanitized-no-media': 'Sanitized HTML with all images and media removed.',
  'plain-text': 'No HTML rendered — message shown as plain text only.',
};

/** Restrictiveness rank: higher = more locked-down / more secure. */
const RANK: Record<SecurityMode, number> = {
  'full-sanitized': 0,
  'sanitized-no-media': 1,
  'plain-text': 2,
};

/** Type guard for values arriving from storage / config / the wire. */
export function isSecurityMode(v: unknown): v is SecurityMode {
  return v === 'full-sanitized' || v === 'sanitized-no-media' || v === 'plain-text';
}

/** Is `mode` at least as locked-down as `floor`? */
export function isAtLeastAsStrict(mode: SecurityMode, floor: SecurityMode): boolean {
  return RANK[mode] >= RANK[floor];
}

/** Raise `mode` up to `floor` when it sits below the admin minimum; never lowers. */
export function clampToFloor(mode: SecurityMode, floor: SecurityMode | null): SecurityMode {
  if (floor === null) return mode;
  return RANK[mode] >= RANK[floor] ? mode : floor;
}

/** The full policy that resolves to an effective mode for a given sender. */
export interface MaxSecurityPolicy {
  /** Config-read admin floor (minimum security). `null` = no floor imposed. */
  adminFloor: SecurityMode | null;
  /** The user's global default when no per-sender override applies. */
  global: SecurityMode;
  /** Per-sender overrides, keyed by lowercased email address. */
  perSender: Readonly<Record<string, SecurityMode>>;
}

/** Normalize a sender address into the per-sender map key. */
export function senderKey(address: string | null | undefined): string {
  return (address ?? '').trim().toLowerCase();
}

/** Resolve the effective mode with precedence admin-floor > per-sender > global.
 *  Per-sender overrides the global default; the admin floor then clamps the
 *  result up so it can never be *less* locked-down than the configured minimum. */
export function resolveMode(policy: MaxSecurityPolicy, sender: string | null): SecurityMode {
  const key = senderKey(sender);
  const override = key.length > 0 ? policy.perSender[key] : undefined;
  const chosen = override ?? policy.global;
  return clampToFloor(chosen, policy.adminFloor);
}

// ── Render plan (consumed by e8 when wiring the body path) ────────────────────

/** The sanitizer directive for a mode — what the (wasm) sanitizer must strip. */
export type SanitizeProfile = 'full' | 'no-media' | 'none';

/** Everything the render path needs to honor a mode, derived once. */
export interface RenderPlan {
  mode: SecurityMode;
  /** `false` for plain-text (render escaped text, not HTML). */
  renderHtml: boolean;
  /** `true` when images/media must not appear (no-media and plain-text). */
  stripMedia: boolean;
  /** Directive for the client-side sanitizer. */
  sanitizeProfile: SanitizeProfile;
  /** The CSP the body frame is pinned to (from sandbox.ts). */
  bodyCsp: string;
}

/** Derive the render plan for a mode. */
export function renderPlan(mode: SecurityMode): RenderPlan {
  return {
    mode,
    renderHtml: mode !== 'plain-text',
    stripMedia: mode !== 'full-sanitized',
    sanitizeProfile:
      mode === 'plain-text' ? 'none' : mode === 'sanitized-no-media' ? 'no-media' : 'full',
    bodyCsp: bodyCsp(mode),
  };
}

// ── Attachment gating (consumed by e8 in the Reader attachment open path) ─────

/** In any locked-down mode, attachments must open ONLY via the re-encode preview
 *  jail (the viewer sandbox), never as original bytes / a raw download. Only
 *  `full-sanitized` (the default posture) allows the original-bytes path. */
export function requiresPreviewJail(mode: SecurityMode): boolean {
  return mode !== 'full-sanitized';
}

/** Convenience inverse: may this mode expose the attachment's original bytes? */
export function allowsOriginalBytes(mode: SecurityMode): boolean {
  return !requiresPreviewJail(mode);
}

// ── Policy store (per-sender + global; admin floor is config-read) ────────────

const STORAGE_KEY = 'mw.maxsec.v1';

interface Persisted {
  global: SecurityMode;
  perSender: Record<string, SecurityMode>;
}

/** Read the admin floor from injected runtime config (config-read, not user
 *  editable). e7/e8 populate `globalThis.__MW_CONFIG__`; absent/invalid = none. */
export function readAdminFloor(): SecurityMode | null {
  const cfg = (globalThis as { __MW_CONFIG__?: { maxSecurityFloor?: unknown } }).__MW_CONFIG__;
  const v = cfg?.maxSecurityFloor;
  return isSecurityMode(v) ? v : null;
}

function load(): Persisted {
  const empty: Persisted = { global: 'full-sanitized', perSender: {} };
  try {
    const raw = globalThis.localStorage?.getItem(STORAGE_KEY);
    if (raw === null || raw === undefined) return empty;
    const parsed = JSON.parse(raw) as Partial<Persisted>;
    const global = isSecurityMode(parsed.global) ? parsed.global : 'full-sanitized';
    const perSender: Record<string, SecurityMode> = {};
    for (const [addr, mode] of Object.entries(parsed.perSender ?? {})) {
      if (isSecurityMode(mode)) perSender[senderKey(addr)] = mode;
    }
    return { global, perSender };
  } catch {
    return empty;
  }
}

export interface MaxSecurityStore {
  /** The user's global default. */
  global: Accessor<SecurityMode>;
  setGlobal(mode: SecurityMode): void;
  /** Per-sender overrides (lowercased-address keyed). */
  perSender: Accessor<Readonly<Record<string, SecurityMode>>>;
  /** Set (or, with `null`, clear) a per-sender override. */
  setSenderMode(address: string, mode: SecurityMode | null): void;
  /** The config-read admin floor (minimum security), or `null`. */
  adminFloor: Accessor<SecurityMode | null>;
  /** The assembled policy (reactive). */
  policy: Accessor<MaxSecurityPolicy>;
  /** Effective mode for `sender` after precedence + floor. */
  effectiveMode(sender: string | null): SecurityMode;
  /** Render plan for `sender`'s effective mode. */
  planFor(sender: string | null): RenderPlan;
}

/** Create the max-security policy store (signals + localStorage persistence). */
export function createMaxSecurityStore(): MaxSecurityStore {
  const initial = load();
  const [global, setGlobalSig] = createSignal<SecurityMode>(initial.global);
  const [perSender, setPerSender] = createSignal<Readonly<Record<string, SecurityMode>>>(
    initial.perSender,
  );
  const [adminFloor] = createSignal<SecurityMode | null>(readAdminFloor());

  function persist(): void {
    try {
      const data: Persisted = { global: global(), perSender: { ...perSender() } };
      globalThis.localStorage?.setItem(STORAGE_KEY, JSON.stringify(data));
    } catch {
      // Non-fatal: the policy still applies in-memory this session.
    }
  }

  const policy = (): MaxSecurityPolicy => ({
    adminFloor: adminFloor(),
    global: global(),
    perSender: perSender(),
  });

  return {
    global,
    perSender,
    adminFloor,
    policy,
    setGlobal(mode) {
      setGlobalSig(mode);
      persist();
    },
    setSenderMode(address, mode) {
      const key = senderKey(address);
      if (key.length === 0) return;
      setPerSender((prev) => {
        const next = { ...prev };
        if (mode === null) delete next[key];
        else next[key] = mode;
        return next;
      });
      persist();
    },
    effectiveMode(sender) {
      return resolveMode(policy(), sender);
    },
    planFor(sender) {
      return renderPlan(resolveMode(policy(), sender));
    },
  };
}

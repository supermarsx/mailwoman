// Cookie consent runtime: loads the configuration the administrator set for this
// domain and region, holds the visitor's decision, applies it (block and purge),
// records it on the server and runs the storage scan. The control itself is
// `ConsentLayer.tsx`; the rules are in `policy.ts`.
//
// Nothing here blocks anything until the configuration says the control is on, and
// a configuration that cannot be loaded leaves storage alone (the last decision
// stored in this browser still applies).

import { createMemo, createRoot, createSignal } from 'solid-js';
import { apiFetch } from '../api/http.ts';
import { withBase } from '../api/basePath.ts';
import { installGate, purge, rawSet, scanStorage, unclassified } from './enforce.ts';
import {
  choicesFor, parseStored, resolve,
  type Category, type Choices, type ConsentConfig, type Decision, type Effective, type Signals, type StoredConsent,
} from './policy.ts';

export const CONSENT_KEY = 'mw.consent';
export const SCAN_KEY = 'mw.consent.scan';
const CONFIG_KEY = 'mw.consent.config';

const [config, setConfig] = createSignal<ConsentConfig | null>(null);
const [stored, setStored] = createSignal<StoredConsent | null>(null);
const [preferencesOpen, setPreferencesOpen] = createSignal(false);
const [tick, setTick] = createSignal(0);
// The admin console previews a draft configuration; nothing is stored or recorded.
const [preview, setPreview] = createSignal<ConsentConfig | null>(null);
const active = (): ConsentConfig | null => preview() ?? config();

function signals(): Signals {
  const nav = typeof navigator === 'undefined' ? undefined : (navigator as Navigator & { globalPrivacyControl?: boolean; doNotTrack?: string });
  return { gpc: nav?.globalPrivacyControl === true, dnt: nav?.doNotTrack === '1' || (typeof window !== 'undefined' && (window as unknown as { doNotTrack?: string }).doNotTrack === '1') };
}

const effective = createRoot(() => createMemo((): Effective | null => {
  tick();
  const c = active();
  return c ? resolve(c, preview() ? null : stored(), signals(), Date.now()) : null;
}));

export const consentConfig = active;
export const previewing = (): boolean => preview() !== null;
/** Show the control with a draft configuration (admin preview); `null` ends it. */
export function previewConsent(c: ConsentConfig | null): void { setPreview(c); }
export const consentEffective = effective;
export const consentStored = stored;
export const preferencesVisible = preferencesOpen;

/** Whether a category may be used now; true while the control is off or not yet loaded. */
export function categoryAllowed(category: Category): boolean {
  const e = effective();
  return e === null ? true : e.allowed[category];
}

/** Whether the appearance sync (a functional feature) may run. */
export const functionalAllowed = (): boolean => categoryAllowed('functional');

const listeners = new Set<() => void>();
/** Called after a decision or a policy change alters what is allowed. */
export function onConsentChange(fn: () => void): () => void {
  listeners.add(fn);
  return () => { listeners.delete(fn); };
}

export function openCookiePreferences(): void { setPreferencesOpen(true); }
export function closeCookiePreferences(): void { setPreferencesOpen(false); setPreview(null); }

function readStored(): StoredConsent | null {
  try { return parseStored(localStorage.getItem(CONSENT_KEY)); } catch { return null; }
}
function saveStored(value: StoredConsent): void {
  rawSet(localStorage, CONSENT_KEY, JSON.stringify(value));
}

let uninstall: (() => void) | null = null;

async function apply(): Promise<void> {
  const c = config();
  const e = c ? resolve(c, stored(), signals(), Date.now()) : null;
  if (!c || !c.enabled || !e) return;
  if (!uninstall) uninstall = installGate(() => { const cc = config(); return cc ? { config: cc, effective: resolve(cc, stored(), signals(), Date.now()) } : null; });
  await purge(c, e);
  for (const fn of listeners) fn();
}

async function fetchConfig(): Promise<ConsentConfig | null> {
  try {
    const response = await apiFetch(withBase('/api/cookies/config'), { cache: 'no-store' });
    if (!response.ok) return null;
    return (await response.json()) as ConsentConfig;
  } catch { return null; }
}

function cachedConfig(): ConsentConfig | null {
  try { const raw = localStorage.getItem(CONFIG_KEY); return raw ? (JSON.parse(raw) as ConsentConfig) : null; } catch { return null; }
}

/** A readable `mw_csrf` cookie means a signed-in browser session. */
function signedIn(): boolean {
  try { return /(?:^|;\s*)mw_csrf=/.test(document.cookie); } catch { return false; }
}

async function postDecision(value: StoredConsent): Promise<void> {
  const body = JSON.stringify({ version: value.version, decision: value.decision, categories: value.categories, signal: value.signal });
  const init = { method: 'POST', headers: { 'Content-Type': 'application/json' }, body } as const;
  try {
    if (signedIn()) {
      const response = await apiFetch(withBase('/api/cookies/consent/account'), init);
      if (response.status !== 401) return;
    }
    await apiFetch(withBase('/api/cookies/consent'), init);
  } catch { /* the decision stays in this browser */ }
}

/** Record a decision: store it, apply it, tell the server. */
export async function decide(decision: Decision, choices?: Partial<Choices>): Promise<void> {
  if (preview()) { setPreview(null); return; }
  const c = config();
  if (!c || !c.enabled) return;
  const base = decision === 'acceptAll' || decision === 'rejectAll' ? choicesFor(decision, c) : decision === 'notice'
    ? { necessary: true, functional: effective()?.allowed.functional ?? false, analytics: effective()?.allowed.analytics ?? false, marketing: effective()?.allowed.marketing ?? false }
    : { necessary: true, functional: false, analytics: false, marketing: false };
  const categories: Choices = { ...base, ...choices, necessary: true };
  const value: StoredConsent = { version: c.version ?? 1, decision, categories, signal: effective()?.signal ?? 'none', decidedAt: Date.now() };
  saveStored(value);
  setStored(value);
  setPreferencesOpen(false);
  await apply();
  void postDecision(value);
}

/** Another device of a signed-in account may already have decided. */
async function adoptFromAccount(): Promise<void> {
  if (stored() !== null || !signedIn()) return;
  try {
    const response = await apiFetch(withBase('/api/cookies/consent/account'), { cache: 'no-store' });
    if (response.status !== 200) return;
    const body = (await response.json()) as { version: number; decision: Decision; categories: Choices; signal: StoredConsent['signal']; decidedAt: number };
    const value = parseStored(JSON.stringify({ ...body, decidedAt: body.decidedAt * 1000 }));
    if (value && stored() === null) { saveStored(value); setStored(value); await apply(); }
  } catch { /* no server copy */ }
}

/** Scan this browser's storage and report names no inventory entry or rule covers. */
export async function scanNow(report = true): Promise<{ total: number; unclassified: { kind: string; name: string }[] }> {
  const c = config();
  const items = await scanStorage();
  const unknown = c ? unclassified(items, c) : [];
  rawSet(localStorage, SCAN_KEY, String(Date.now()));
  if (report && c?.enabled && unknown.length > 0) {
    try {
      await apiFetch(withBase('/api/cookies/detected'), { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ items: unknown.slice(0, 100) }) });
    } catch { /* reporting is best effort */ }
  }
  return { total: items.length, unclassified: unknown };
}

function scanDue(c: ConsentConfig): boolean {
  const hours = c.scanIntervalHours ?? 0;
  if (hours <= 0) return false;
  let last = 0;
  try { last = Number(localStorage.getItem(SCAN_KEY) ?? 0) || 0; } catch { /* none */ }
  return Date.now() - last >= hours * 3_600_000;
}

let booted: Promise<void> | null = null;

/** Load the configuration and apply the stored decision. Safe to call more than once. */
export function bootConsent(): Promise<void> {
  booted ??= (async () => {
    setStored(readStored());
    const fresh = await fetchConfig();
    const c = fresh ?? cachedConfig();
    if (!c) return;
    if (fresh) rawSetConfig(fresh);
    setConfig(c);
    if (!c.enabled) return;
    await apply();
    await adoptFromAccount();
    if (scanDue(c)) void scanNow(true);
  })();
  return booted;
}

function rawSetConfig(c: ConsentConfig): void {
  // Without the control nothing is stored; with it, the last configuration lets a
  // reload with the server unreachable keep enforcing the stored decision.
  try {
    if (c.enabled) rawSet(localStorage, CONFIG_KEY, JSON.stringify({ ...c, inventory: c.inventory ?? [], rules: c.rules ?? [] }));
    else localStorage.removeItem(CONFIG_KEY);
  } catch { /* storage unavailable */ }
}

/** Tests: forget loaded state. */
export function resetConsentForTests(): void {
  uninstall?.();
  uninstall = null;
  booted = null;
  listeners.clear();
  setConfig(null);
  setStored(null);
  setPreferencesOpen(false);
  setPreview(null);
  setTick((n) => n + 1);
}
/** Tests and the admin preview: use this configuration without the server. */
export function setConsentConfigForTests(c: ConsentConfig | null): void { setConfig(c); setTick((n) => n + 1); }
export { unclassified };

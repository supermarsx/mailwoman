// Cookie consent: the pure rules. What the server's configuration and the stored
// decision mean for each category, how a storage name is classified, and when a
// stored decision is stale. No DOM and no network here, so every rule is a plain
// function the tests can call.

export type Category = 'necessary' | 'functional' | 'analytics' | 'marketing';
export const CATEGORIES: readonly Category[] = ['necessary', 'functional', 'analytics', 'marketing'];
export const OPTIONAL: readonly Exclude<Category, 'necessary'>[] = ['functional', 'analytics', 'marketing'];
export type StorageKind = 'cookie' | 'localStorage' | 'sessionStorage' | 'indexedDB';
export type Mode = 'optIn' | 'optOut' | 'noticeOnly';
export type Decision = 'acceptAll' | 'rejectAll' | 'custom' | 'notice';
export type Signal = 'none' | 'gpc' | 'dnt';
export type Choices = Record<Category, boolean>;

export interface CategoryConfig {
  id: Category;
  enabled: boolean;
  default: boolean;
  label: Record<string, string>;
  description: Record<string, string>;
}
export interface InventoryEntry {
  id: string;
  kind: StorageKind;
  pattern: string;
  category: Category;
  purpose: string;
  duration: string;
  party: string;
  provider: string;
}
export interface Rule { kind: StorageKind; pattern: string; category: Category }

/** `GET /api/cookies/config`; `{ enabled: false }` when the control is off. */
export interface ConsentConfig {
  enabled: boolean;
  mode?: Mode;
  region?: string | null;
  strictRegion?: boolean;
  version?: number;
  lifetimeDays?: number;
  reprompt?: boolean;
  layout?: 'banner' | 'card' | 'modal';
  position?: 'bottom' | 'top' | 'start' | 'end' | 'center';
  rejectProminent?: boolean;
  policyUrl?: string | null;
  privacyUrl?: string | null;
  honourGpc?: boolean;
  honourDnt?: boolean;
  scanIntervalHours?: number;
  unclassified?: 'report' | Exclude<Category, 'necessary'>;
  log?: boolean;
  text?: { title: Record<string, string>; body: Record<string, string> };
  categories?: CategoryConfig[];
  inventory?: InventoryEntry[];
  rules?: Rule[];
}

export interface StoredConsent {
  version: number;
  decision: Decision;
  categories: Choices;
  signal: Signal;
  decidedAt: number;
}

/** `*` matches any run of characters; everything else is exact. Mirrors the server. */
export function globMatch(pattern: string, name: string): boolean {
  const parts = pattern.split('*');
  if (parts.length === 1) return pattern === name;
  let rest = name;
  const last = parts.length - 1;
  for (let i = 0; i <= last; i += 1) {
    const part = parts[i]!;
    if (i === 0) {
      if (!rest.startsWith(part)) return false;
      rest = rest.slice(part.length);
    } else if (i === last) {
      return rest.endsWith(part);
    } else {
      const at = rest.indexOf(part);
      if (at < 0) return false;
      rest = rest.slice(at + part.length);
    }
  }
  return true;
}

export interface Classified { category: Category; source: 'rule' | 'inventory' }

/** The category of a name: administrator rules first, then the built-in inventory; an exact pattern beats a wildcard. */
export function classify(kind: StorageKind, name: string, config: Pick<ConsentConfig, 'inventory' | 'rules'>): Classified | null {
  const pick = (entries: readonly { kind: StorageKind; pattern: string; category: Category }[]): Category | null => {
    const same = entries.filter((e) => e.kind === kind);
    const hit = same.find((e) => !e.pattern.includes('*') && e.pattern === name) ?? same.find((e) => globMatch(e.pattern, name));
    return hit ? hit.category : null;
  };
  const fromRule = pick(config.rules ?? []);
  if (fromRule) return { category: fromRule, source: 'rule' };
  const fromInventory = pick(config.inventory ?? []);
  return fromInventory ? { category: fromInventory, source: 'inventory' } : null;
}

export interface Signals { gpc: boolean; dnt: boolean }

/** The privacy signal that applies under the configuration (`none` when it is not honoured). */
export function activeSignal(config: ConsentConfig, signals: Signals): Signal {
  if (config.honourGpc && signals.gpc) return 'gpc';
  if (config.honourDnt && signals.dnt) return 'dnt';
  return 'none';
}

export type StaleReason = 'none' | 'noDecision' | 'version' | 'expired';

/** Whether the stored decision still counts. */
export function staleness(config: ConsentConfig, stored: StoredConsent | null, nowMs: number): StaleReason {
  if (!stored) return 'noDecision';
  if (config.reprompt && stored.version !== config.version) return 'version';
  const lifetime = (config.lifetimeDays ?? 180) * 86_400_000;
  if (nowMs - stored.decidedAt > lifetime) return 'expired';
  return 'none';
}

export interface Effective {
  /** What each category may do right now. */
  allowed: Choices;
  /** The control should be shown (nothing valid is stored and the mode asks). */
  needsDecision: boolean;
  stale: StaleReason;
  signal: Signal;
  /** The toggles the visitor may change: offered categories the signal does not hold off. */
  adjustable: Record<Category, boolean>;
}

const none = (): Choices => ({ necessary: true, functional: false, analytics: false, marketing: false });

/** Resolve configuration, stored decision and browser signals into what is allowed now. */
export function resolve(config: ConsentConfig, stored: StoredConsent | null, signals: Signals, nowMs: number): Effective {
  if (!config.enabled) {
    return { allowed: { necessary: true, functional: true, analytics: true, marketing: true }, needsDecision: false, stale: 'none', signal: 'none', adjustable: { necessary: false, functional: false, analytics: false, marketing: false } };
  }
  const offered = (c: Category): boolean => c === 'necessary' || (config.categories ?? []).find((x) => x.id === c)?.enabled === true;
  const defaults = (): Choices => {
    const out = none();
    for (const c of OPTIONAL) out[c] = offered(c) && (config.categories ?? []).find((x) => x.id === c)?.default === true;
    return out;
  };
  const stale = staleness(config, stored, nowMs);
  const valid = stale === 'none' && stored !== null;
  const mode = config.mode ?? 'optIn';
  let allowed: Choices;
  if (valid) {
    allowed = { ...stored.categories, necessary: true };
  } else if (mode === 'optIn') {
    allowed = none();
  } else {
    // optOut and noticeOnly: the defaults apply until the visitor says otherwise.
    allowed = defaults();
  }
  const signal = activeSignal(config, signals);
  if (signal !== 'none') {
    allowed.analytics = false;
    allowed.marketing = false;
  }
  for (const c of OPTIONAL) if (!offered(c)) allowed[c] = false;
  const held = (c: Category): boolean => signal !== 'none' && (c === 'analytics' || c === 'marketing');
  const adjustable: Record<Category, boolean> = {
    necessary: false,
    functional: offered('functional') && mode !== 'noticeOnly',
    analytics: offered('analytics') && mode !== 'noticeOnly' && !held('analytics'),
    marketing: offered('marketing') && mode !== 'noticeOnly' && !held('marketing'),
  };
  // A visitor with a valid decision is not asked again; a notice is shown once.
  return { allowed, needsDecision: !valid, stale, signal, adjustable };
}

/** Whether storage of `kind`/`name` may be written now. Unknown names follow the `unclassified` setting. */
export function mayStore(kind: StorageKind, name: string, config: ConsentConfig, effective: Effective): boolean {
  if (!config.enabled) return true;
  const found = classify(kind, name, config);
  if (found) return effective.allowed[found.category];
  const policy = config.unclassified ?? 'report';
  return policy === 'report' ? true : effective.allowed[policy];
}

/** The decision a bulk button records, as the category choices. */
export function choicesFor(decision: 'acceptAll' | 'rejectAll', config: ConsentConfig): Choices {
  const out = none();
  if (decision === 'acceptAll') {
    for (const c of OPTIONAL) out[c] = (config.categories ?? []).find((x) => x.id === c)?.enabled === true;
  }
  return out;
}

/** Text for the visitor's language: the exact locale, then its base language, then `en`; `undefined` when the administrator set none. */
export function localized(map: Record<string, string> | undefined, locale: string): string | undefined {
  if (!map) return undefined;
  const pick = (key: string): string | undefined => {
    const value = map[key];
    return typeof value === 'string' && value.trim() ? value : undefined;
  };
  return pick(locale) ?? pick(locale.split('-')[0] ?? '') ?? pick('en');
}

/** A stored value read back from storage, or `null` when it is missing or malformed. */
export function parseStored(raw: string | null): StoredConsent | null {
  if (!raw) return null;
  try {
    const value: unknown = JSON.parse(raw);
    if (!value || typeof value !== 'object') return null;
    const v = value as Partial<StoredConsent>;
    const decisions: Decision[] = ['acceptAll', 'rejectAll', 'custom', 'notice'];
    if (typeof v.version !== 'number' || typeof v.decidedAt !== 'number' || !v.decision || !decisions.includes(v.decision) || !v.categories || typeof v.categories !== 'object') return null;
    const c = v.categories as Partial<Choices>;
    const out = none();
    for (const key of OPTIONAL) out[key] = c[key] === true;
    return { version: v.version, decision: v.decision, categories: out, signal: v.signal === 'gpc' || v.signal === 'dnt' ? v.signal : 'none', decidedAt: v.decidedAt };
  } catch {
    return null;
  }
}

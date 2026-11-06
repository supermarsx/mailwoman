// Retention rules for the signed-in person's own mail (Settings > Retention).
// The server runs them as a daily job through the mail engine; the limits come
// from the administrator's `retention.user.*` settings.
import { apiFetch } from './http.ts';
import { withBase } from './basePath.ts';

export const ACTION_KINDS = ['keep', 'delete', 'trash', 'archive', 'move', 'markRead', 'stripAttachments'] as const;
export type ActionKind = (typeof ACTION_KINDS)[number];
export type ReadState = 'any' | 'read' | 'unread';

export interface RuleScope {
  folders: string[];
  subfolders: boolean;
  labels: string[];
  senders: string[];
  domains: string[];
  minSizeKb: number | null;
  maxSizeKb: number | null;
  read: ReadState;
  hasAttachment: boolean | null;
}

export interface RuleAction { kind: ActionKind; days: number; target: string | null }
export interface RuleExclude { flagged: boolean; unread: boolean; tags: string[] }

/** What a rule is saved with. */
export interface RuleInput {
  name: string;
  enabled: boolean;
  scope: RuleScope;
  action: RuleAction;
  exclude: RuleExclude;
}

export interface Rule extends RuleInput {
  id: string;
  order: number;
  template: string | null;
  createdAt: number;
  updatedAt: number;
}

export interface RetentionPolicy {
  enabled: boolean;
  minDays: number;
  /** 0: no ceiling. */
  maxDays: number;
  maxRules: number;
  allowPermanentDelete: boolean;
  undoDays: number;
  mandatory: { role: string; days: number }[];
  protectedFolders: string[];
  /** Whether the administrator lets rules remove attachments. Absent: not allowed. */
  allowStripAttachments?: boolean;
  strip?: { minAgeDays: number; minAttachmentKb: number; keepInlineImages: boolean; undoDays: number };
}

export interface RetentionSettings {
  enabled: boolean;
  trashFirst: boolean;
  undoDays: number;
  pausedUntil: number;
  paused: boolean;
  lastRunAt: number;
  nextRunAt: number | null;
}

export interface FolderDefault {
  id: string;
  name: string;
  enabled: boolean;
  body: { scope: RuleScope; action: RuleAction; exclude: RuleExclude };
}

export interface RunSummary {
  id: string;
  startedAt: number;
  finishedAt: number;
  trigger: string;
  matched: number;
  acted: number;
  errors: number;
}

export interface Template { id: string; name: string; rule: { scope: RuleScope; action: RuleAction; exclude: RuleExclude } }

export interface RetentionOverview {
  policy: RetentionPolicy;
  settings: RetentionSettings;
  folderDefaults: FolderDefault[];
  rules: Rule[];
  templates: Template[];
  runs: RunSummary[];
}

export interface PreviewSample {
  id: string;
  folder: string;
  subject: string;
  from: string;
  receivedAt: number;
  action: 'trash' | 'move' | 'delete' | 'markRead' | 'stripAttachments';
  source: string;
}

export interface Counts {
  moved: number; trashed: number; purged: number; markedRead: number; stamped: number;
  /** Messages rewritten without attachments, the bytes freed, and messages left alone. */
  stripped?: number; reclaimedBytes?: number; stripSkipped?: number;
}

export interface Preview {
  skipped: string | null;
  scanned: number;
  matched: number;
  would: Counts;
  done: Counts;
  errors: number;
  truncated: boolean;
  bySource: Record<string, number>;
  sample: PreviewSample[];
  /** Enabled attachment-removal rules left unused because the administrator turned the action off. */
  stripRulesSkipped?: number;
  /** The freed-bytes figure includes a size-based guess beyond the messages opened. */
  stripEstimate?: boolean;
}

export interface EffectiveRetention {
  mail: { category: string; name?: string; days: number | null; action: string; source: 'administrator' | 'you'; paused?: boolean }[];
  userRules: { available: boolean; enabled: boolean; paused: boolean; rules: number; trashFirst: boolean; undoDays: number; allowPermanentDelete: boolean; lastRunAt: number };
  system: { id: string; class: 'short' | 'medium' | 'long' | null; days: number | null; keptForever: boolean; archived: boolean; archiveDays: number | null; source: string }[];
}

export class RetentionError extends Error {
  constructor(readonly status: number, readonly code: string, message: string) { super(message); }
}

async function call<T>(path: string, init: { method?: string; body?: unknown } = {}): Promise<T> {
  const headers: Record<string, string> = { Accept: 'application/json' };
  if (init.body !== undefined) headers['Content-Type'] = 'application/json';
  const response = await apiFetch(withBase(`/api/retention${path}`), {
    method: init.method ?? 'GET', credentials: 'same-origin', headers,
    ...(init.body === undefined ? {} : { body: JSON.stringify(init.body) }),
  });
  const body = await response.json().catch(() => ({})) as Record<string, unknown>;
  if (!response.ok) {
    throw new RetentionError(
      response.status,
      typeof body['error'] === 'string' ? body['error'] : 'error',
      typeof body['detail'] === 'string' ? body['detail'] : `request failed (${response.status})`,
    );
  }
  return body as T;
}

export const loadRetention = (): Promise<RetentionOverview> => call('/rules');
export const createRule = (input: RuleInput): Promise<Rule> => call('/rules', { method: 'POST', body: input });
export const updateRule = (id: string, input: RuleInput): Promise<Rule> => call(`/rules/${encodeURIComponent(id)}`, { method: 'PUT', body: input });
export const deleteRule = (id: string): Promise<{ deleted: boolean }> => call(`/rules/${encodeURIComponent(id)}`, { method: 'DELETE' });
export const adoptTemplate = (id: string): Promise<Rule> => call(`/templates/${encodeURIComponent(id)}/adopt`, { method: 'POST' });
export const saveSettings = (input: {
  enabled?: boolean; trashFirst?: boolean; undoDays?: number; pausedUntil?: number;
  folderDefaults?: { id?: string; name: string; enabled: boolean; folder: string; action: RuleAction; exclude: RuleExclude }[];
}): Promise<RetentionOverview> => call('/settings', { method: 'PUT', body: input });
/** Evaluate one rule being edited (`rule`), or every rule of the account. Nothing changes. */
export const previewRules = (rule?: RuleInput): Promise<Preview> => call('/preview', { method: 'POST', body: rule ? { rule } : {} });
export const runRules = (): Promise<Preview> => call('/run', { method: 'POST', body: {} });
export const loadEffective = (): Promise<EffectiveRetention> => call('/effective');

export const emptyScope = (): RuleScope => ({
  folders: [], subfolders: true, labels: [], senders: [], domains: [], minSizeKb: null, maxSizeKb: null, read: 'any', hasAttachment: null,
});
export const blankRule = (): RuleInput => ({
  name: '', enabled: true, scope: emptyScope(), action: { kind: 'delete', days: 90, target: null },
  exclude: { flagged: true, unread: false, tags: ['keep'] },
});

// Pure helpers for Settings > Retention: list parsing, the limits a rule is held
// to, and the message that describes a rule in one line.
import type { ActionKind, FolderDefault, RetentionPolicy, RuleAction, RuleExclude, RuleInput } from '../../api/retention.ts';

/** `a, b ,c` to `['a', 'b', 'c']`; empty entries dropped, duplicates removed. */
export function splitList(text: string): string[] {
  return [...new Set(text.split(/[,\n]/).map((x) => x.trim()).filter(Boolean))];
}

export const joinList = (items: readonly string[]): string => items.join(', ');

/** Actions that remove mail from where it is or hide it for good: the days floor and ceiling apply. */
export const isDestructive = (kind: ActionKind): boolean => kind === 'delete' || kind === 'trash';

/** Whether the days field means anything for this action. */
export const needsDays = (kind: ActionKind): boolean => kind !== 'keep';

/** The first problem with a rule that the form can see without asking the server, as a message id. */
export function ruleProblem(rule: RuleInput, policy: RetentionPolicy): { id: string; args: Record<string, number> } | null {
  if (rule.name.trim() === '') return { id: 'retention-error-name', args: {} };
  if (!Number.isInteger(rule.action.days) || rule.action.days < 0 || rule.action.days > 3650) return { id: 'retention-error-days', args: {} };
  if (isDestructive(rule.action.kind)) {
    if (rule.action.days < policy.minDays) return { id: 'retention-error-min', args: { days: policy.minDays } };
    if (policy.maxDays > 0 && rule.action.days > policy.maxDays) return { id: 'retention-error-max', args: { days: policy.maxDays } };
  }
  if (rule.action.kind === 'move' && !rule.action.target) return { id: 'retention-error-target', args: {} };
  if (rule.action.kind === 'stripAttachments') {
    if (!policy.allowStripAttachments) return { id: 'retention-error-strip-disabled', args: {} };
    const floor = policy.strip?.minAgeDays ?? 90;
    if (rule.action.days < floor) return { id: 'retention-error-strip-min', args: { days: floor } };
  }
  return null;
}

/** The actions a person may choose: removing attachments only when the administrator allows it. */
export const availableActions = (policy: RetentionPolicy, all: readonly ActionKind[]): ActionKind[] =>
  all.filter((k) => k !== 'stripAttachments' || policy.allowStripAttachments === true);

/** "1.5 MB" style size for the freed-space line. */
export function formatBytes(bytes: number): string {
  if (bytes >= 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  if (bytes >= 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  if (bytes >= 1024) return `${Math.round(bytes / 1024)} KB`;
  return `${bytes} B`;
}

/** The message that describes what happens to the mail a rule's scope matches. */
export function actionMessage(rule: RuleInput, permanent: boolean): { id: string; args: Record<string, number> } {
  const { kind, days } = rule.action;
  if (kind === 'keep') return { id: 'retention-summary-keep', args: {} };
  if (kind === 'delete') return { id: permanent ? 'retention-summary-delete-permanent' : 'retention-summary-delete', args: { days } };
  return { id: `retention-summary-${kind}`, args: { days } };
}

/** Whole days between two unix times, for the "next run" line. */
export const daysFromNow = (unix: number, now: number = Date.now() / 1000): number => Math.max(0, Math.ceil((unix - now) / 86400));

// ─── per-folder defaults ───────────────────────────────────────────────────────

/** What a folder row offers; `inherit` means no default of its own. */
export const FOLDER_CHOICES = ['inherit', 'keep', 'trash', 'delete', 'archive', 'markRead'] as const;
export type FolderChoice = (typeof FOLDER_CHOICES)[number];

export interface FolderRow { id: string; name: string; parentId: string | null; sortOrder: number; role: string | null }
export interface FlatFolder { id: string; name: string; depth: number; role: string | null }

/** Folders in tree order (parents before children, siblings by sort order then name). */
export function flattenFolders(folders: readonly FolderRow[]): FlatFolder[] {
  const ids = new Set(folders.map((f) => f.id));
  const kids = new Map<string | null, FolderRow[]>();
  for (const f of folders) {
    const parent = f.parentId !== null && ids.has(f.parentId) ? f.parentId : null;
    kids.set(parent, [...(kids.get(parent) ?? []), f]);
  }
  const out: FlatFolder[] = [];
  const seen = new Set<string>();
  const walk = (parent: string | null, depth: number): void => {
    const list = [...(kids.get(parent) ?? [])].sort((a, b) => a.sortOrder - b.sortOrder || a.name.localeCompare(b.name));
    for (const f of list) {
      if (seen.has(f.id)) continue;
      seen.add(f.id);
      out.push({ id: f.id, name: f.name, depth, role: f.role });
      walk(f.id, depth + 1);
    }
  };
  walk(null, 0);
  return out;
}

export interface FolderSetting { choice: FolderChoice; days: number }

/** The editable state per folder id, from the defaults the server holds. A default
 *  the list cannot express (a move) is left out and kept as it is on save. */
export function settingsFromDefaults(defaults: readonly FolderDefault[]): Map<string, FolderSetting> {
  const out = new Map<string, FolderSetting>();
  for (const d of defaults) {
    const folder = d.body.scope.folders[0];
    const kind = d.body.action.kind;
    if (!folder || !d.enabled || out.has(folder) || !(FOLDER_CHOICES as readonly string[]).includes(kind)) continue;
    out.set(folder, { choice: kind as FolderChoice, days: d.body.action.days });
  }
  return out;
}

export interface DefaultsPayload { id?: string; name: string; enabled: boolean; folder: string; action: RuleAction; exclude: RuleExclude }

const SAFE_EXCLUDE: RuleExclude = { flagged: true, unread: false, tags: ['keep'] };

/** The `folderDefaults` array to save: one entry per folder with a choice other than
 *  `inherit`, keeping ids and exclusions of entries that already exist, and every
 *  existing entry this list does not manage (moves, disabled, several per folder). */
export function defaultsPayload(
  folders: readonly FlatFolder[],
  settings: ReadonlyMap<string, FolderSetting>,
  existing: readonly FolderDefault[],
): DefaultsPayload[] {
  const managed = new Set<string>();
  const out: DefaultsPayload[] = [];
  const kept = new Set<FolderDefault>();
  for (const f of folders) {
    const s = settings.get(f.id);
    if (!s || s.choice === 'inherit') continue;
    const old = existing.find((d) => d.enabled && d.body.scope.folders[0] === f.id && (FOLDER_CHOICES as readonly string[]).includes(d.body.action.kind));
    if (old) kept.add(old);
    managed.add(f.id);
    out.push({
      ...(old ? { id: old.id } : {}),
      name: f.name,
      enabled: true,
      folder: f.id,
      action: { kind: s.choice, days: s.choice === 'keep' ? 0 : s.days, target: null },
      exclude: old ? old.body.exclude : { ...SAFE_EXCLUDE },
    });
  }
  for (const d of existing) {
    if (kept.has(d)) continue;
    const folder = d.body.scope.folders[0] ?? '';
    const expressible = d.enabled && (FOLDER_CHOICES as readonly string[]).includes(d.body.action.kind) && folders.some((f) => f.id === folder);
    if (expressible) continue; // the user set it back to inherit
    out.push({ id: d.id, name: d.name, enabled: d.enabled, folder, action: d.body.action, exclude: d.body.exclude });
  }
  return out;
}

/** A problem with a folder's setting that the page can see before asking the server. */
export function folderProblem(s: FolderSetting, policy: RetentionPolicy): { id: string; args: Record<string, number> } | null {
  if (s.choice === 'inherit' || s.choice === 'keep') return null;
  if (!Number.isInteger(s.days) || s.days < 0 || s.days > 3650) return { id: 'retention-error-days', args: {} };
  if (isDestructive(s.choice)) {
    if (s.days < policy.minDays) return { id: 'retention-error-min', args: { days: policy.minDays } };
    if (policy.maxDays > 0 && s.days > policy.maxDays) return { id: 'retention-error-max', args: { days: policy.maxDays } };
  }
  return null;
}

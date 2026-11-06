// Where the copy of a sent message goes. Pure resolution, shared by the composer
// (shows "Sent items: <folder>") and mirrored by the server when the submission
// completes. Precedence: a per-message choice, then the admin-forced policy
// (skipCopy), then the mode.

export type SentFolderMode = 'default' | 'fixed' | 'byRecipient' | 'byIdentity' | 'bySubjectRule';

export interface SentFolderRule {
  /** Match recipients whose address ends with this domain (`example.com`). */
  recipientDomain?: string;
  /** Match subjects containing this text, case-insensitive. */
  subjectContains?: string;
  /** Folder role or id. */
  folder: string;
}

export interface SentFolderPolicy {
  mode: SentFolderMode;
  fixedId?: string;
  perIdentity?: Record<string, string>;
  rules?: SentFolderRule[];
  skipCopy: boolean;
}

/** The per-message choice: a folder, `none` (do not save), or `default`. */
export type SentFolderChoice = { kind: 'default' } | { kind: 'none' } | { kind: 'folder'; folder: string };

export interface SentFolderInput {
  choice?: SentFolderChoice;
  policy: SentFolderPolicy;
  identityId?: string;
  recipients: readonly string[];
  subject: string;
  /** Folder roles/ids that exist and can be written to. */
  writable: ReadonlySet<string>;
  /** The normal Sent folder. */
  sent: string;
}

export type SentFolderResolution =
  | { skip: true; reason: 'message' | 'policy' }
  | { skip: false; folder: string; source: 'message' | 'fixed' | 'identity' | 'recipient' | 'subject' | 'default'; fellBackFrom?: string };

const domainOf = (addr: string): string => addr.slice(addr.lastIndexOf('@') + 1).trim().toLowerCase();

function matchRule(rule: SentFolderRule, i: SentFolderInput): boolean {
  const d = rule.recipientDomain?.trim().toLowerCase().replace(/^@/, '');
  const s = rule.subjectContains?.trim().toLowerCase();
  if (!d && !s) return false;
  if (d && !i.recipients.some((r) => { const rd = domainOf(r); return rd === d || rd.endsWith(`.${d}`); })) return false;
  if (s && !i.subject.toLowerCase().includes(s)) return false;
  return true;
}

export function resolveSentFolder(i: SentFolderInput): SentFolderResolution {
  if (i.policy.skipCopy) return { skip: true, reason: 'policy' };
  if (i.choice?.kind === 'none') return { skip: true, reason: 'message' };
  let wanted: { folder: string; source: Exclude<Extract<SentFolderResolution, { skip: false }>['source'], 'default'> } | null = null;
  if (i.choice?.kind === 'folder') wanted = { folder: i.choice.folder, source: 'message' };
  else {
    const p = i.policy;
    if (p.mode === 'fixed' && p.fixedId) wanted = { folder: p.fixedId, source: 'fixed' };
    else if (p.mode === 'byIdentity' && i.identityId && p.perIdentity?.[i.identityId]) wanted = { folder: p.perIdentity[i.identityId]!, source: 'identity' };
    else if (p.mode === 'byRecipient') {
      const r = p.rules?.find((x) => x.recipientDomain && !x.subjectContains && matchRule(x, i));
      if (r) wanted = { folder: r.folder, source: 'recipient' };
    } else if (p.mode === 'bySubjectRule') {
      // First matching rule wins; rules may combine recipient and subject conditions.
      const r = p.rules?.find((x) => matchRule(x, i));
      if (r) wanted = { folder: r.folder, source: 'subject' };
    }
  }
  if (!wanted) return { skip: false, folder: i.sent, source: 'default' };
  if (i.writable.has(wanted.folder)) return { skip: false, folder: wanted.folder, source: wanted.source };
  return { skip: false, folder: i.sent, source: wanted.source, fellBackFrom: wanted.folder };
}

const MODES: readonly SentFolderMode[] = ['default', 'fixed', 'byRecipient', 'byIdentity', 'bySubjectRule'];

/** Build the policy from the settings values (`compose.sentFolder.*`); anything malformed is ignored. */
export function parseSentFolderPolicy(v: {
  mode?: unknown; fixedId?: unknown; perIdentity?: unknown; rules?: unknown; skipCopy?: unknown;
}): SentFolderPolicy {
  const lines = (x: unknown): string[] => (Array.isArray(x) ? x.filter((s): s is string => typeof s === 'string') : []);
  const perIdentity: Record<string, string> = {};
  for (const line of lines(v.perIdentity)) {
    const eq = line.indexOf('=');
    if (eq > 0 && eq < line.length - 1) perIdentity[line.slice(0, eq).trim()] = line.slice(eq + 1).trim();
  }
  const rules: SentFolderRule[] = [];
  for (const line of lines(v.rules)) {
    const eq = line.lastIndexOf('=');
    if (eq <= 0 || eq === line.length - 1) continue;
    const folder = line.slice(eq + 1).trim();
    const cond = line.slice(0, eq).trim();
    if (cond.startsWith('domain:')) rules.push({ recipientDomain: cond.slice(7).trim(), folder });
    else if (cond.startsWith('subject:')) rules.push({ subjectContains: cond.slice(8).trim(), folder });
  }
  const mode = MODES.find((m) => m === v.mode) ?? 'default';
  return { mode, fixedId: typeof v.fixedId === 'string' ? v.fixedId : '', perIdentity, rules, skipCopy: v.skipCopy === true };
}

/** Folder roles and ids a copy may be filed into, mapped to the mailbox id. */
export function writableFolders(boxes: readonly { id: string; role?: string | null; myRights?: { mayAddItems?: boolean } | null }[]): Map<string, string> {
  const out = new Map<string, string>();
  for (const b of boxes) {
    if (b.myRights?.mayAddItems === false) continue;
    out.set(b.id, b.id);
    if (b.role) out.set(b.role, b.id);
  }
  return out;
}

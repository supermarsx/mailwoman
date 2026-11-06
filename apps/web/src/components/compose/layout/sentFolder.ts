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

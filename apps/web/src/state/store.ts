import { createSignal, type Accessor } from 'solid-js';
import { ApiError, type Client, type LoginInput, type Me } from '../api/client.ts';
import {
  emailGetFull,
  listMailbox,
  mailboxGet,
  responseFor,
  sendEnvelope,
  type DraftInput,
} from '../api/jmap.ts';
import {
  CAP_MAIL,
  type Email,
  type EmailGetResponse,
  type EmailSetResponse,
  type EmailSubmissionSetResponse,
  type Id,
  type Mailbox,
  type MailboxGetResponse,
} from '../api/jmap-types.ts';

export type ToastKind = 'info' | 'success' | 'error';
export interface Toast {
  kind: ToastKind;
  message: string;
}

export interface AppState {
  me: Accessor<Me | null>;
  authChecked: Accessor<boolean>;
  online: Accessor<boolean>;
  mailboxes: Accessor<Mailbox[]>;
  selectedMailboxId: Accessor<Id | null>;
  messages: Accessor<Email[]>;
  listLoading: Accessor<boolean>;
  openEmail: Accessor<Email | null>;
  sanitizedHtml: Accessor<string | null>;
  readLoading: Accessor<boolean>;
  toast: Accessor<Toast | null>;

  accountId: Accessor<string | null>;
  sentMailboxId: Accessor<Id | null>;
  draftsMailboxId: Accessor<Id | null>;

  init(): Promise<void>;
  login(input: LoginInput): Promise<void>;
  logout(): Promise<void>;
  selectMailbox(id: Id): Promise<void>;
  openMessage(id: Id): Promise<void>;
  closeMessage(): void;
  sendMessage(input: Omit<DraftInput, 'from' | 'draftMailboxId' | 'sentMailboxId'>): Promise<void>;
  showToast(kind: ToastKind, message: string, ttlMs?: number): void;
}

function roleOf(mailboxes: Mailbox[], role: string): Id | null {
  return mailboxes.find((m) => m.role === role)?.id ?? null;
}

export function createAppState(client: Client): AppState {
  const [me, setMe] = createSignal<Me | null>(null);
  const [authChecked, setAuthChecked] = createSignal(false);
  const [online, setOnline] = createSignal(true);
  const [accountId, setAccountId] = createSignal<string | null>(null);
  const [mailboxes, setMailboxes] = createSignal<Mailbox[]>([]);
  const [selectedMailboxId, setSelectedMailboxId] = createSignal<Id | null>(null);
  const [messages, setMessages] = createSignal<Email[]>([]);
  const [listLoading, setListLoading] = createSignal(false);
  const [openEmail, setOpenEmail] = createSignal<Email | null>(null);
  const [sanitizedHtml, setSanitizedHtml] = createSignal<string | null>(null);
  const [readLoading, setReadLoading] = createSignal(false);
  const [toast, setToast] = createSignal<Toast | null>(null);

  let wasOffline = false;
  client.onNetwork((up) => {
    setOnline(up);
    if (!up) {
      wasOffline = true;
      setToast({ kind: 'error', message: 'Connection lost — retrying…' });
    } else if (wasOffline) {
      wasOffline = false;
      showToast('success', 'Back online', 2500);
    }
  });

  let toastTimer: ReturnType<typeof setTimeout> | undefined;
  function showToast(kind: ToastKind, message: string, ttlMs = 3500): void {
    if (toastTimer !== undefined) clearTimeout(toastTimer);
    setToast({ kind, message });
    toastTimer = setTimeout(() => setToast(null), ttlMs);
  }

  async function loadMailboxes(): Promise<void> {
    const session = await client.session();
    const primary = session.primaryAccounts[CAP_MAIL];
    const acct = primary ?? Object.keys(session.accounts)[0] ?? null;
    setAccountId(acct);
    if (acct === null) {
      setMailboxes([]);
      return;
    }
    const res = await client.jmap(mailboxGet(acct));
    const boxes = responseFor<MailboxGetResponse>(res, 'c0').list;
    // Inbox first, then by sortOrder/name.
    boxes.sort((a, b) => {
      if (a.role === 'inbox') return -1;
      if (b.role === 'inbox') return 1;
      if (a.sortOrder !== b.sortOrder) return a.sortOrder - b.sortOrder;
      return a.name.localeCompare(b.name);
    });
    setMailboxes(boxes);
    const inbox = boxes.find((m) => m.role === 'inbox') ?? boxes[0];
    if (inbox !== undefined) {
      await selectMailbox(inbox.id);
    }
  }

  async function selectMailbox(id: Id): Promise<void> {
    setSelectedMailboxId(id);
    setOpenEmail(null);
    setSanitizedHtml(null);
    const acct = accountId();
    if (acct === null) return;
    setListLoading(true);
    try {
      const res = await client.jmap(listMailbox(acct, id));
      const got = responseFor<EmailGetResponse>(res, 'g');
      setMessages(got.list);
    } finally {
      setListLoading(false);
    }
  }

  async function openMessage(id: Id): Promise<void> {
    const acct = accountId();
    if (acct === null) return;
    setReadLoading(true);
    setSanitizedHtml(null);
    try {
      const res = await client.jmap(emailGetFull(acct, id));
      const email = responseFor<EmailGetResponse>(res, 'g').list[0] ?? null;
      setOpenEmail(email);
      if (email !== null) {
        const raw = extractHtmlBody(email);
        const clean = await client.sanitize(raw);
        setSanitizedHtml(clean);
      }
    } finally {
      setReadLoading(false);
    }
  }

  function closeMessage(): void {
    setOpenEmail(null);
    setSanitizedHtml(null);
  }

  async function sendMessage(
    input: Omit<DraftInput, 'from' | 'draftMailboxId' | 'sentMailboxId'>,
  ): Promise<void> {
    const acct = accountId();
    const user = me();
    if (acct === null || user === null) throw new Error('not authenticated');
    const drafts = roleOf(mailboxes(), 'drafts') ?? selectedMailboxId();
    if (drafts === null) throw new Error('no mailbox to hold the draft');
    const sent = roleOf(mailboxes(), 'sent');
    const draft: DraftInput = {
      from: { name: null, email: user.username },
      draftMailboxId: drafts,
      to: input.to,
      subject: input.subject,
      htmlBody: input.htmlBody,
      ...(sent !== null ? { sentMailboxId: sent } : {}),
    };
    const res = await client.jmap(sendEnvelope(acct, draft));
    const setRes = responseFor<EmailSetResponse>(res, 'set');
    if (setRes.notCreated?.['draft'] !== undefined) {
      const e = setRes.notCreated['draft'];
      throw new Error(`draft rejected: ${e.type}`);
    }
    const subRes = responseFor<EmailSubmissionSetResponse>(res, 'submit');
    if (subRes.notCreated?.['send'] !== undefined) {
      const e = subRes.notCreated['send'];
      throw new Error(`send rejected: ${e.type}`);
    }
    showToast('success', 'Message sent');
    // Refresh Sent if we are viewing it; otherwise refresh current view.
    const current = selectedMailboxId();
    if (current !== null) await selectMailbox(current);
  }

  async function init(): Promise<void> {
    try {
      const user = await client.me();
      setMe(user);
      setAccountId(user.accountId);
      await loadMailboxes();
    } catch (err) {
      if (err instanceof ApiError && err.status === 401) {
        setMe(null);
      } else {
        throw err;
      }
    } finally {
      setAuthChecked(true);
    }
  }

  async function login(input: LoginInput): Promise<void> {
    const user = await client.login(input);
    setMe(user);
    setAccountId(user.accountId);
    await loadMailboxes();
  }

  async function logout(): Promise<void> {
    try {
      await client.logout();
    } finally {
      setMe(null);
      setAccountId(null);
      setMailboxes([]);
      setMessages([]);
      setSelectedMailboxId(null);
      setOpenEmail(null);
      setSanitizedHtml(null);
    }
  }

  return {
    me,
    authChecked,
    online,
    mailboxes,
    selectedMailboxId,
    messages,
    listLoading,
    openEmail,
    sanitizedHtml,
    readLoading,
    toast,
    accountId,
    sentMailboxId: () => roleOf(mailboxes(), 'sent'),
    draftsMailboxId: () => roleOf(mailboxes(), 'drafts'),
    init,
    login,
    logout,
    selectMailbox,
    openMessage,
    closeMessage,
    sendMessage,
    showToast,
  };
}

/** Pick the HTML body value to sanitize, falling back to escaped text/preview. */
export function extractHtmlBody(email: Email): string {
  const parts = email.htmlBody ?? [];
  const values = email.bodyValues ?? {};
  for (const part of parts) {
    if (part.partId !== null) {
      const v = values[part.partId];
      if (v !== undefined) return v.value;
    }
  }
  // No HTML part: wrap the plain text/preview so the sanitizer still runs.
  const text = email.textBody
    ?.map((p) => (p.partId !== null ? (values[p.partId]?.value ?? '') : ''))
    .join('\n');
  const fallback = text && text.length > 0 ? text : email.preview;
  return `<pre>${escapeHtml(fallback)}</pre>`;
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
}

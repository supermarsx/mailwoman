// Store composition root (plan §3 e0 store-slices refactor).
//
// `AppState`'s public shape (accessors + actions) is FROZEN by this refactor so
// components and the web tests do not change. The implementation is split into
// disjoint slices under `state/slices/` — `mail` (session + mail, e7), and the
// currently-empty `tags`/`outbox`/`realtime`/`offline`/`theme` seams the other
// web executors fill. This module wires the store "core" (network status +
// toasts) and composes every slice into `AppState`.

import { createSignal, type Accessor } from 'solid-js';
import type { Client, LoginInput, Me } from '../api/client.ts';
import type { DraftInput } from '../api/jmap.ts';
import type { Email, Id, Mailbox } from '../api/jmap-types.ts';
import type { SliceContext } from './slices/context.ts';
import { createMailSlice } from './slices/mail.ts';
import { createTagsSlice } from './slices/tags.ts';
import { createOutboxSlice } from './slices/outbox.ts';
import { createRealtimeSlice } from './slices/realtime.ts';
import { createOfflineSlice } from './slices/offline.ts';
import { createThemeSlice } from './slices/theme.ts';

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

/** The cross-cutting store core: connection status + the transient toast. */
interface StoreCore {
  online: Accessor<boolean>;
  toast: Accessor<Toast | null>;
  showToast(kind: ToastKind, message: string, ttlMs?: number): void;
}

function createStoreCore(client: Client): StoreCore {
  const [online, setOnline] = createSignal(true);
  const [toast, setToast] = createSignal<Toast | null>(null);

  let toastTimer: ReturnType<typeof setTimeout> | undefined;
  function showToast(kind: ToastKind, message: string, ttlMs = 3500): void {
    if (toastTimer !== undefined) clearTimeout(toastTimer);
    setToast({ kind, message });
    toastTimer = setTimeout(() => setToast(null), ttlMs);
  }

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

  return { online, toast, showToast };
}

export function createAppState(client: Client): AppState {
  const core = createStoreCore(client);
  const ctx: SliceContext = { client, showToast: core.showToast };

  const mail = createMailSlice(ctx);
  // The remaining slices are seams the other web executors fill (plan §3);
  // wired here so they compose into `AppState` as they grow.
  const tags = createTagsSlice(ctx);
  const outbox = createOutboxSlice(ctx);
  const realtime = createRealtimeSlice(ctx);
  const offline = createOfflineSlice(ctx);
  const theme = createThemeSlice(ctx);

  return {
    online: core.online,
    toast: core.toast,
    showToast: core.showToast,
    ...mail,
    ...tags,
    ...outbox,
    ...realtime,
    ...offline,
    ...theme,
  };
}

export { extractHtmlBody } from './slices/mail.ts';

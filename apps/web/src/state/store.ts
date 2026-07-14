// Store composition root (plan §3 e0 store-slices refactor).
//
// `AppState`'s public shape (accessors + actions) is FROZEN by this refactor so
// components and the web tests do not change. The implementation is split into
// disjoint slices under `state/slices/` — `mail` (session + mail, e7), and the
// currently-empty `tags`/`outbox`/`realtime`/`offline`/`theme` seams the other
// web executors fill. This module wires the store "core" (network status +
// toasts) and composes every slice into `AppState`.

import { createSignal, type Accessor } from 'solid-js';
import type { Client } from '../api/client.ts';
import type { SliceContext } from './slices/context.ts';
import { createMailSlice, type MailSlice } from './slices/mail.ts';
import { createTagsSlice, type TagsSlice } from './slices/tags.ts';
import { createOutboxSlice, type OutboxSlice } from './slices/outbox.ts';
import { createRealtimeSlice, type RealtimeSlice } from './slices/realtime.ts';
import { createOfflineSlice, type OfflineSlice } from './slices/offline.ts';
import { createThemeSlice, type ThemeSlice } from './slices/theme.ts';
import { createCalendarSlice, type CalendarSlice } from './slices/calendar.ts';
import { createTasksSlice, type TasksSlice } from './slices/tasks.ts';
import { createNotesSlice, type NotesSlice } from './slices/notes.ts';
import { createContactsSlice, type ContactsSlice } from './slices/contacts.ts';
import { createKeysSlice, type KeysSlice } from './slices/keys.ts';
import { createAssistSlice, type AssistSlice } from './slices/assist.ts';
import { createDirectorySlice, type DirectorySlice } from './slices/directory.ts';
import { createNextcloudSlice, type NextcloudSlice } from './slices/nextcloud.ts';
import { AssistService } from '../modules/assist/service.ts';
import type { Fetcher as DirectoryFetcher } from '../modules/directory/service.ts';
import type { Fetcher as NextcloudFetcher } from '../modules/nextcloud/service.ts';
import { broadcastChannelAvailable, openStoreChannel } from '../worker/broadcast.ts';
import { broadcastEnvelope } from '../worker/protocol.ts';
import type { WorkerEnvelope } from '../contracts/worker.ts';

export type ToastKind = 'info' | 'success' | 'error';
export interface Toast {
  kind: ToastKind;
  message: string;
}

/** The cross-cutting store-core API (connection status + transient toast). */
export interface StoreCoreApi {
  online: Accessor<boolean>;
  toast: Accessor<Toast | null>;
  showToast(kind: ToastKind, message: string, ttlMs?: number): void;
}

/**
 * `AppState` is the store core composed with every slice's public interface.
 * Each web executor owns its slice's shape in `slices/*.ts`; this intersection
 * is the only place they meet, so they never collide on the field list.
 */
export type AppState = StoreCoreApi &
  MailSlice &
  TagsSlice &
  OutboxSlice &
  RealtimeSlice &
  OfflineSlice &
  ThemeSlice &
  // ── V3 PIM slices (plan §2.5). Additive — the V2 accessors above are
  // unchanged, preserving the frozen `AppState` public shape. ──
  CalendarSlice &
  TasksSlice &
  NotesSlice &
  ContactsSlice &
  // ── V4 crypto/security slice (plan §2.5). Additive — the accessors above are
  // unchanged, preserving the frozen `AppState` public shape. e0 stub; e2 fills. ──
  KeysSlice & {
    // ── V7 last-mile mailbox integrations (plan §2.7/§14, e14b). Namespaced so
    // their rich APIs don't collide with the flat accessors above; each is inert
    // (hidden) until its backend is configured, keeping the disabled path unchanged.
    readonly assist: AssistSlice;
    readonly directory: DirectorySlice;
    readonly nextcloud: NextcloudSlice;
  };

/** Optional transport doubles for the V7 mailbox slices (injected by tests; production
 *  defaults to same-origin services so the mailbox behaves identically). */
export interface AppStateDeps {
  readonly assistService?: AssistService;
  readonly directoryFetcher?: DirectoryFetcher;
  readonly nextcloudFetcher?: NextcloudFetcher;
}

function createStoreCore(client: Client): StoreCoreApi {
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

/** A light peer-tab state-sync over the `mw-store` BroadcastChannel (plan §2.6).
 *  This tab posts a ping after each mutation; a peer's ping calls `onRemote`
 *  (which refetches). A real `BroadcastChannel` never echoes to the sender, so
 *  tabs don't loop. Absent the API (jsdom / private windows) this is inert. The
 *  full SharedWorker store proxy (worker/proxy.ts) is a deliberate follow-up. */
function createPeerSync(onRemote: () => void): { publish(): void } {
  if (!broadcastChannelAvailable()) return { publish: () => undefined };
  const port = openStoreChannel();
  if (port === null) return { publish: () => undefined };
  port.onmessage = (ev: { data: unknown }): void => {
    const env = ev.data as WorkerEnvelope;
    if (env !== null && typeof env === 'object' && env.kind === 'broadcast' && env.method === 'state') {
      onRemote();
    }
  };
  return { publish: () => port.postMessage(broadcastEnvelope({ type: 'refetch' })) };
}

export function createAppState(client: Client, deps: AppStateDeps = {}): AppState {
  const core = createStoreCore(client);
  const ctx: SliceContext = { client, showToast: core.showToast };

  // Independent slices first (no cross-slice deps).
  const tags = createTagsSlice(ctx);
  const theme = createThemeSlice(ctx);
  const offline = createOfflineSlice(ctx);
  const outbox = createOutboxSlice(ctx);

  // V3 PIM slices (plan §2.5): independent seams, mock-backed until e10.
  const calendar = createCalendarSlice(ctx);
  const tasks = createTasksSlice(ctx);
  const notes = createNotesSlice(ctx);
  const contacts = createContactsSlice(ctx);

  // V4 crypto/security slice (plan §2.5): mock-backed + worker-stub until e8.
  const keys = createKeysSlice(ctx);

  // V7 mailbox integrations (plan §2.7/§14, e14b): Assist (AI), directory/GAL, and
  // Nextcloud files. Each stays hidden until its backend is configured (loadConfig /
  // ensureEnabled probes), so an unconfigured deployment's compose/read UX is
  // unchanged. Transports are injectable for tests; production uses same-origin.
  const assist = createAssistSlice(ctx, deps.assistService ?? new AssistService());
  const directory = createDirectorySlice(deps.directoryFetcher);
  const nextcloud = createNextcloudSlice(deps.nextcloudFetcher);

  // Late-bound so the mail slice can broadcast before the peer channel exists.
  let publishPeerSync: () => void = () => undefined;

  // Mail slice gets the V2 integration seams: offline queue routing, live
  // network status, the reduced offline search, and the peer-sync broadcast.
  const mailCtx: SliceContext = {
    ...ctx,
    online: core.online,
    enqueueOffline: offline.enqueueOffline,
    searchOffline: offline.searchOffline,
    broadcastChange: () => publishPeerSync(),
  };
  const mail = createMailSlice(mailCtx);

  // Realtime push (plan §2.2): the controller's reconciler fires `onChanged`
  // for the datatypes that actually moved → refetch the live list / outbox.
  // `startRealtime()`/`stopRealtime()` are driven by App around the session.
  const realtime = createRealtimeSlice(ctx, {
    onChanged: (accountId, types) => {
      if (accountId !== mail.accountId()) return;
      if (types.includes('Email') || types.includes('Mailbox')) void mail.refreshCurrentMailbox();
      if (types.includes('EmailSubmission')) void outbox.refreshOutbox();
    },
  });

  // Peer-tab sync: an inbound peer ping refetches the current mailbox.
  publishPeerSync = createPeerSync(() => void mail.refreshCurrentMailbox()).publish;

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
    ...calendar,
    ...tasks,
    ...notes,
    ...contacts,
    ...keys,
    assist,
    directory,
    nextcloud,
  };
}

export { extractHtmlBody } from './slices/mail.ts';

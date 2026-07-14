import { onMount, onCleanup, createEffect, lazy, Suspense, Show, Switch, Match, type JSX } from 'solid-js';
import { AppContext } from './state/context.ts';
import { createAppState } from './state/store.ts';
import { createConfiguredClient } from './api/transport.ts';
import { getPlatform, initPlatform } from './platform/index.ts';
import { capabilityEnabled } from './platform/capabilities.ts';
import { Login } from './screens/Login.tsx';
import { MailboxScreen } from './screens/Mailbox.tsx';
import { Toast } from './components/Toast.tsx';
import { ConnectionToast } from './realtime/ConnectionToast.tsx';

// V6 admin panel (plan §2.6, §3 e7): a LAZY, admin-session-gated route reached
// ONLY via dynamic import, so the whole `screens/Admin/**` tree code-splits into
// its own chunk and is ABSENT from the login→inbox mailbox bundle (bundle gate).
// The panel runs under a separate admin session domain; the normal SPA path below
// is byte-unchanged (the early return only fires on the `/admin` path).
const AdminScreen = lazy(() => import('./screens/Admin/index.tsx'));

// V6 OAuth 2.1 consent (plan §3 e8/e11): the resource-owner grant/deny screen,
// reached ONLY via the `/oauth/authorize` redirect. Lazily imported so it
// code-splits out of the mailbox bundle; it reads the authorize params from
// `window.location.search` and posts to `/oauth/{consent,decision}`.
const ConsentScreen = lazy(() => import('./screens/Consent/index.tsx'));

/** Is the app being served under the separate `/admin` route? */
function isAdminRoute(): boolean {
  if (typeof location === 'undefined') return false;
  const path = location.pathname.replace(/\/+$/, '');
  return path === '/admin' || path.startsWith('/admin/');
}

/** Is the app being served under the `/oauth/authorize` consent route? */
function isOAuthAuthorizeRoute(): boolean {
  if (typeof location === 'undefined') return false;
  return location.pathname.replace(/\/+$/, '') === '/oauth/authorize';
}

export function App(): JSX.Element {
  if (isAdminRoute()) {
    return (
      <Suspense fallback={<div class="boot">Loading…</div>}>
        <AdminScreen />
      </Suspense>
    );
  }

  if (isOAuthAuthorizeRoute()) {
    return (
      <Suspense fallback={<div class="boot">Loading…</div>}>
        <ConsentScreen />
      </Suspense>
    );
  }

  const client = createConfiguredClient();
  const app = createAppState(client);

  onMount(() => {
    void app.init();
    // V7 Assist (plan §14): read the gateway config once at boot. A gateway that is
    // off/unreachable resolves to DISABLED_CONFIG, so every Assist surface stays
    // hidden and the mailbox UX is unchanged (no Assist affordances render).
    void app.assist.loadConfig();
    // Resolve the platform capability layer for this runtime (plan §2.1). In a
    // browser this is a no-op returning the browser impl; in a shell it installs
    // the native impl (dynamically importing tauri.ts). e7 relies on this at boot.
    void initPlatform();
  });

  // Open the realtime push transport once a session exists, and tear it down on
  // logout (plan §2.2). Inert under jsdom (no WebSocket/EventSource).
  createEffect(() => {
    if (app.me() !== null) app.startRealtime();
    else app.stopRealtime();
  });

  // V5 push subscribe on login (plan §3 e6). Fire-and-forget + gated: a plain
  // browser (no shell, no injected capability) never touches the push endpoints.
  createEffect(() => {
    if (app.me() !== null && capabilityEnabled('push')) {
      void getPlatform()
        .pushSubscribe()
        .catch(() => undefined);
    }
  });

  // V5 native new-mail notification + unread badge (plan §3 e6), fed by the live
  // message list (which the realtime StateChange refetch updates). Gated so the
  // browser path is byte-identical; notifications only fire while the tab is
  // hidden, and the first run just seeds the baseline (no notification storm).
  let knownUnread: Set<string> | null = null;
  createEffect(() => {
    const messages = app.messages();
    if (!capabilityEnabled('notifications')) return;
    const platform = getPlatform();
    const unread = messages.filter((m) => m.keywords?.['$seen'] !== true);
    void platform.setBadgeCount(unread.length);
    if (knownUnread !== null && typeof document !== 'undefined' && document.hidden) {
      for (const m of unread) {
        if (!knownUnread.has(m.id)) {
          void platform.notify({
            id: m.id,
            title: 'New message',
            body: m.subject ?? '(no subject)',
            ...(m.threadId !== undefined ? { threadId: m.threadId } : {}),
          });
        }
      }
    }
    knownUnread = new Set(unread.map((m) => m.id));
  });

  // V5 deep links / mailto (plan §3 e6). Inert in a browser (onOpenUrl never
  // fires); in a shell the OS hands mailto:/mailwoman: URLs here. Gated.
  if (capabilityEnabled('deepLinks')) {
    const off = getPlatform().onOpenUrl((url) => {
      window.dispatchEvent(new CustomEvent('mw:open-url', { detail: url }));
    });
    onCleanup(off);
  }

  // Keep the offline slice's cached header slice in sync with the visible list,
  // so the reduced offline search + offline reads have data to work from (§2.5).
  createEffect(() => {
    app.cacheHeaders(app.messages());
  });

  return (
    <AppContext.Provider value={app}>
      <Show when={app.authChecked()} fallback={<div class="boot">Loading…</div>}>
        <Switch>
          <Match when={app.me() === null}>
            <Login />
          </Match>
          <Match when={app.me() !== null}>
            <MailboxScreen />
          </Match>
        </Switch>
      </Show>
      <Toast />
      <ConnectionToast />
    </AppContext.Provider>
  );
}

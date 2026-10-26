import { on, onMount, onCleanup, createEffect, createSignal, Show, Switch, Match, type JSX } from 'solid-js';
import { AppContext } from './state/context.ts';
import { createAppState } from './state/store.ts';
import { createConfiguredClient } from './api/transport.ts';
import { isNativeAuth, nativeServerBase } from './api/serverAddress.ts';
import { stripBase } from './api/basePath.ts';
import { isConsolePage } from './api/adminPath.ts';
import { getPlatform, initPlatform } from './platform/index.ts';
import { capabilityEnabled } from './platform/capabilities.ts';
import { Login } from './screens/Login.tsx';
import { ToastRegion } from './components/toast/index.ts';
import { ConnectionToast } from './realtime/ConnectionToast.tsx';
import { AsyncBoundary, LazyRoute } from './components/ErrorBoundary.tsx';
import { AsyncError, AsyncPending } from './components/AsyncState.tsx';
import { t } from './i18n/index.ts';
import { clearSettings, loadSettings } from './api/settings.ts';
import { startAnalytics } from './api/startAnalytics.ts';

import { observePreferenceAccount, preferenceAccountIsCurrent, type PreferenceAccount } from './api/accountPreferenceLifecycle.ts';
import { SettingsService } from './screens/Settings/service.ts';
import { shouldNotify } from './api/notificationPolicy.ts';

// Every route below is a DYNAMIC IMPORT, and each now loads through `LazyRoute`
// rather than a bare `<Suspense fallback="Loading…">`. A chunk that fails to
// arrive — a 404 against a tab left open across a redeploy, or one dropped
// request — used to leave that fallback on screen for the life of the tab, with
// nothing to recover it and nothing to report it. `LazyRoute` bounds the pending
// state and its retry constructs a new `import()`, which is what makes the retry
// capable of succeeding (see ErrorBoundary.tsx).

/** V6 admin panel (plan §2.6, §3 e7): admin-session-gated, its own chunk, ABSENT
 *  from the login→inbox bundle (bundle gate). The normal SPA path is unchanged;
 *  the early return only fires on the `/admin` path. */
const loadAdminScreen = () => import('./screens/Admin/index.tsx');

/** V6 OAuth 2.1 consent (plan §3 e8/e11): the resource-owner grant/deny screen,
 *  reached ONLY via the `/oauth/authorize` redirect. */
const loadConsentScreen = () => import('./screens/Consent/index.tsx');
const loadIdentityConsent = () => import('./screens/IdentityConsent/index.tsx');

/** t10 UI-plugin tier (plan §3 e13, SPEC §22.2): a fail-soft surface rendering the
 *  approved+enabled sandboxed UI plugins. Renders NOTHING when no plugin is
 *  approved (or the registry is absent/offline), so the mailbox layout is
 *  unchanged. Mounted only inside the authenticated branch below. */
const loadUiPluginTier = () => import('./plugins-ui/Tier.tsx');

/** Forced password change (t27, OH-1): the only screen an account gets while its
 *  admin `force_password_change` flag is set. Its own chunk — most sessions never
 *  need it, and it must not pull the mail UI in. */
const loadForcedPasswordChange = () => import('./modules/passwd/ForcedPasswordChange.tsx');

/** The mailbox itself, now code-split out of the entry chunk: the shell renders
 *  Login without it, so a logged-out visitor never downloads the whole mail UI. */
const loadMailboxScreen = () =>
  import('./screens/Mailbox.tsx').then((m) => ({ default: m.MailboxScreen }));

// Both route tests run on the pathname with the deploy prefix REMOVED. Under
// `MW_BASE_PATH=/mail` the browser is at `/mail/admin`, which matches neither
// literal — and because these are early returns rather than lookups, the failure
// is silent: the admin console and the OAuth consent screen would render the
// MAILBOX instead of erroring. `stripBase` is a no-op at the origin root, so the
// unprefixed deployment is byte-unchanged.

/** Is this page the admin console? The server marks the page it serves at the
 *  operator's admin path (`MW_ADMIN_PATH`, default `/admin`); see `adminPath.ts`. */
function isAdminRoute(): boolean {
  return isConsolePage();
}

/** Is the app being served under the `/oauth/authorize` consent route? */
function isOAuthAuthorizeRoute(): boolean {
  if (typeof location === 'undefined') return false;
  return stripBase(location.pathname).replace(/\/+$/, '') === '/oauth/authorize';
}

export function App(): JSX.Element {
  if (isAdminRoute()) {
    return <LazyRoute load={loadAdminScreen} />;
  }

  if (typeof location !== 'undefined' && stripBase(location.pathname).replace(/\/+$/, '') === '/oidc/consent') {
    return <LazyRoute load={loadIdentityConsent} />;
  }

  if (isOAuthAuthorizeRoute()) {
    return <LazyRoute load={loadConsentScreen} />;
  }

  // A desktop or mobile shell built without a server address, with no
  // provisioning file naming one (t32). There is nothing to sign in to and the
  // user cannot supply an address, so nothing is requested.
  if (isNativeAuth() && nativeServerBase() === null) {
    return (
      <div class="async-state async-state--error" role="alert" data-testid="shell-not-configured">
        <p class="async-state__message">{t('common-shell-not-configured')}</p>
      </div>
    );
  }

  const client = createConfiguredClient();
  const app = createAppState(client);
  startAnalytics();
  createEffect(on(app.accountId, account => {
    clearSettings();
    if (account) void loadSettings();
  }));
  onCleanup(clearSettings);

  // `init()` resolves the session. A 401 is not a failure — it is the logged-out
  // answer and `init` handles it — so anything that lands here is a real boot
  // failure: the server is unreachable, or it answered something unusable.
  //
  // It used to be fired as `void app.init()`, so such a failure became an
  // unhandled rejection and the shell fell through to the LOGIN FORM, because
  // `me()` is null either way. The user was invited to authenticate against a
  // server that had just failed to answer, and their credentials would fail for
  // a reason the screen could not explain.
  const [bootError, setBootError] = createSignal<unknown>(null);
  function boot(): void {
    setBootError(null);
    void app.init().catch(setBootError);
  }

  // A desktop or mobile shell talks to the server it was built or provisioned for
  // (t32), which the user cannot change, so its error card also says who can. In
  // a browser the server is the page's own origin, and nothing changes.
  const native = isNativeAuth();

  // A session whose account must change its password first. The server refuses
  // its JMAP, push and realtime requests (403 `passwordChangeRequired`), so it
  // is a session but not yet a mail session: nothing below may treat it as one.
  const held = (): boolean => app.me()?.passwordChangeRequired === true;
  const inMail = (): boolean => app.me() !== null && !held();

  onMount(() => {
    boot();
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
    if (inMail()) app.startRealtime();
    else app.stopRealtime();
  });

  const [notificationAccount, setNotificationAccount] = createSignal<PreferenceAccount | null>(null);
  onMount(() => { onCleanup(observePreferenceAccount(setNotificationAccount)); });

  // V5 native new-mail notification + unread badge (plan §3 e6), fed by the live
  // message list (which the realtime StateChange refetch updates). Gated so the
  // browser path is byte-identical; notifications only fire while the tab is
  // hidden, and the first run just seeds the baseline (no notification storm).
  let knownUnread: Set<string> | null = null;
  let knownNotificationAccount: PreferenceAccount | null = null;
  createEffect(() => {
    const messages = app.messages();
    const account = notificationAccount();
    if (account !== knownNotificationAccount) {
      knownUnread = null;
      knownNotificationAccount = account;
    }
    if (!account || !inMail()) { knownUnread = null; return; }
    if (!capabilityEnabled('notifications') && (typeof Notification === 'undefined' || Notification.permission !== 'granted')) return;
    const platform = getPlatform();
    const unread = messages.filter((m) => m.keywords?.['$seen'] !== true);
    void platform.setBadgeCount(unread.length);
    if (knownUnread !== null && typeof document !== 'undefined' && document.hidden) {
      const candidates = unread.filter(message => !knownUnread?.has(message.id));
      if (candidates.length > 0) {
        // Recheck administrator policy immediately before each delivery batch.
        // A failed check or an account transition suppresses the batch.
        void new SettingsService().notifications().then(config => {
          if (!preferenceAccountIsCurrent(account) || !document.hidden) return;
          for (const message of candidates) {
            if (shouldNotify(config, { ...message, mailboxNames: app.mailboxes().filter(mailbox => message.mailboxIds?.[mailbox.id]).map(mailbox => mailbox.name) })) void platform.notify({
              id: message.id, title: t('common-notify-new-message'),
              body: message.subject ?? t('common-notify-no-subject'),
              ...(message.threadId !== undefined ? { threadId: message.threadId } : {}),
            });
          }
        }).catch(() => undefined);
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
      <Show
        when={app.authChecked()}
        fallback={<AsyncPending variant="boot" onRetry={boot} />}
      >
        <Switch>
          {/* Ordered before the logged-out branch: a boot FAILURE and a
              logged-out session are indistinguishable by `me()` alone, and
              offering a login form for an unreachable server is the wrong
              answer to the wrong question. */}
          <Match when={bootError() !== null}>
            <AsyncError error={bootError()} onRetry={boot} />
            <Show when={native}>
              <p class="async-state__message" data-testid="boot-contact-administrator">
                {t('common-contact-administrator')}
              </p>
            </Show>
          </Match>
          <Match when={app.me() === null}>
            <Login />
          </Match>
          <Match when={held()}>
            {/* Ordered before the mailbox: the change form and a sign-out
                control are all a held account gets. */}
            <LazyRoute load={loadForcedPasswordChange} />
          </Match>
          <Match when={inMail()}>
            <>
              {/* The mailbox is the app; if it throws, a blank shell with a
                  working toast region is not a usable fallback. */}
              <AsyncBoundary>
                <LazyRoute load={loadMailboxScreen} />
              </AsyncBoundary>
              {/* The plugin tier is fail-soft by design — it renders nothing
                  when no plugin is approved — so its failure must stay contained
                  and must not take the mailbox down with it. */}
              <LazyRoute load={loadUiPluginTier} failSoft />
            </>
          </Match>
        </Switch>
      </Show>
      <ToastRegion />
      <ConnectionToast />
    </AppContext.Provider>
  );
}

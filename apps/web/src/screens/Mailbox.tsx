import { createMemo, createSignal, For, Show, Suspense, onMount, onCleanup, type JSX } from 'solid-js';
import { Dynamic } from 'solid-js/web';
import { useApp } from '../state/context.ts';
import { t, isolate, loadCatalog } from '../i18n/index.ts';
import * as a11y from '../components/mailA11y.css.ts';
import { useRealtime } from '../realtime/context.ts';
import { MessageList } from '../components/MessageList.tsx';
import { Reader } from '../components/Reader.tsx';
import { Compose } from '../components/Compose.tsx';
import { Outbox } from '../components/Outbox.tsx';
import { InboxTabs } from '../components/InboxTabs.tsx';
import { UndoToast } from '../components/UndoToast.tsx';
import { SubTabStrip } from '../components/SubTabStrip.tsx';
import { Ribbon } from '../components/Ribbon.tsx';
import { Settings } from './Settings.tsx';
import { Attachments } from './Attachments.tsx';
import { APP_MODULES, KEYS_MODULE } from '../shell/modules.ts';
import { createShellRouter, isPimSurface, type ShellSurface } from '../shell/router.ts';
import { shouldRefetchPim } from '../realtime/pimRefetch.ts';
// V7 Assist in the mailbox (plan §14.3, e14b): the chat panel + the semantic-search
// toggle. Both are gated on the Assist gateway/capabilities, so a Disabled gateway
// renders NOTHING and the mailbox is unchanged.
import { AssistPanel, SemanticSearchToggle } from '../modules/assist/index.ts';

// The nav-rail app modules: the four V3 PIM modules + the V4 key-management
// module (plan §2.5, e8 mount). Keys is reachable at `#/keys` beside them.
const APP_NAV_MODULES = [...APP_MODULES, KEYS_MODULE];

/** The search box above the message list; submits an `Email/query` (engine →
 *  mw-search online, reduced cached search offline). */
function SearchBox(): JSX.Element {
  const app = useApp();
  const [query, setQuery] = createSignal(app.search());
  // V7 semantic search (§14.3): off by default; the toggle only renders when the
  // `search-semantic` Assist capability is granted, and its state rides the query.
  const [semantic, setSemantic] = createSignal(false);

  return (
    <form
      class="mail-search"
      role="search"
      onSubmit={(e) => {
        e.preventDefault();
        void app.searchMessages(query(), { semantic: semantic() });
      }}
    >
      <input
        class="mail-search__input"
        type="search"
        aria-label={t('mail-search-label')}
        placeholder={t('mail-search-placeholder')}
        value={query()}
        onInput={(e) => setQuery(e.currentTarget.value)}
      />
      <button type="submit" class={`btn btn--ghost mail-search__submit ${a11y.focusable}`}>
        {t('mail-search')}
      </button>
      <Show when={app.searchActive()}>
        <button
          type="button"
          class={`btn btn--ghost mail-search__clear ${a11y.focusable}`}
          onClick={() => {
            setQuery('');
            void app.clearSearch();
          }}
        >
          {t('mail-search-clear')}
        </button>
      </Show>
      <SemanticSearchToggle config={app.assist.config()} enabled={semantic()} onChange={setSemantic} />
    </form>
  );
}

/** Refetch the open PIM module after a pushed change (plan §1.8 realtime). */
function refetchPim(app: ReturnType<typeof useApp>, surface: ShellSurface): void {
  if (surface === 'calendar') void app.loadCalendars();
  else if (surface === 'tasks') void app.loadTasks();
  else if (surface === 'notes') void app.loadNotes();
  else if (surface === 'contacts') void app.loadContacts();
}

export function MailboxScreen(): JSX.Element {
  const app = useApp();
  const { subTabs } = useRealtime();
  const [composing, setComposing] = createSignal(false);
  const [settingsOpen, setSettingsOpen] = createSignal(false);

  // The shell router (plan §2.5): Mail/Outbox/Attachments + the four PIM modules
  // are hash-routed surfaces, so each PIM module is reachable + deep-linkable.
  const router = createShellRouter();
  const surface = (): ShellSurface => router.route().surface;

  // V7 Assist context (§14.3): the open message (subject + preview) the assistant
  // may reason over. Only plain text is ever forwarded (E2EE/attachments excluded by
  // the gateway ceilings); empty when nothing is open.
  const assistContext = createMemo(() => {
    const email = app.openEmail();
    const acct = app.accountId();
    if (email === null || acct === null) return [];
    const box = app.mailboxes().find((m) => m.id === app.selectedMailboxId());
    const text = [email.subject ?? '', email.preview ?? ''].filter((s) => s.length > 0).join('\n');
    return [{ account: acct, folder: box?.name ?? 'Mail', text, kind: 'plain' as const }];
  });

  // Pull the mail catalog once for the whole mailbox area (idempotent).
  onMount(() => void loadCatalog('mail'));

  // Seed a single "messages" sub-tab so the multi-surface strip is live.
  onMount(() => {
    if (subTabs.tabs().length === 0) {
      subTabs.open({ kind: 'messages', title: 'Mail', id: 'mail', pinned: true });
    }
  });

  // Realtime PIM refetch (plan §1.8): the push controller broadcasts a coarse
  // ping on every PIM mutation (t5-e8); on it, refetch the open PIM module so it
  // updates without a manual refresh. Granular PIM keys are honored if present.
  onMount(() => {
    const off = app.onRealtimeChange((change) => {
      const s = surface();
      if (isPimSurface(s) && shouldRefetchPim(s, change)) refetchPim(app, s);
      // V4 (plan §2.2/§2.5): a crypto `StateChange` arrives as the coarse push
      // ping (like PIM); when the key list is open, reload it so a key generated
      // /imported/trusted in another session appears without a manual refresh.
      else if (s === 'keys') void app.loadKeys();
    });
    onCleanup(off);
  });

  return (
    <div class="shell">
      <Show when={app.layout() === 'ribbon'}>
        <Ribbon onCompose={() => setComposing(true)} onOpenSettings={() => setSettingsOpen(true)} />
      </Show>
      <aside class="sidebar">
        <div class="sidebar__head">
          <span class="sidebar__brand">{t('mail-brand')}</span>
          <Show when={app.me()}>{(m) => <span class="sidebar__user">{isolate(m().username)}</span>}</Show>
          <button
            type="button"
            class={`btn btn--ghost sidebar__settings ${a11y.iconButton}`}
            aria-label={t('mail-nav-settings')}
            onClick={() => setSettingsOpen(true)}
          >
            ⚙
          </button>
        </div>
        <button type="button" class={`btn btn--primary sidebar__compose ${a11y.focusable}`} onClick={() => setComposing(true)}>
          {t('mail-compose')}
        </button>
        <nav class="sidebar__nav" aria-label={t('mail-nav-mailboxes')}>
          <For each={app.mailboxes()}>
            {(box) => (
              <button
                type="button"
                class={`sidebar__box ${a11y.focusable}`}
                classList={{ 'sidebar__box--active': surface() === 'mail' && app.selectedMailboxId() === box.id }}
                onClick={() => {
                  router.navigate('mail');
                  void app.selectMailbox(box.id);
                }}
              >
                <span class="sidebar__box-name">{box.name}</span>
                <Show when={box.unreadEmails > 0}>
                  <span class="sidebar__badge">{box.unreadEmails}</span>
                </Show>
              </button>
            )}
          </For>
          <button
            type="button"
            class={`sidebar__box ${a11y.focusable}`}
            classList={{ 'sidebar__box--active': surface() === 'attachments' }}
            onClick={() => router.navigate('attachments')}
          >
            <span class="sidebar__box-name">{t('mail-nav-attachments')}</span>
          </button>
          <button
            type="button"
            class={`sidebar__box ${a11y.focusable}`}
            classList={{ 'sidebar__box--active': surface() === 'outbox' }}
            onClick={() => {
              router.navigate('outbox');
              void app.refreshOutbox();
            }}
          >
            <span class="sidebar__box-name">{t('mail-nav-outbox')}</span>
            <Show when={app.cancelableOutbox().length > 0}>
              <span class="sidebar__badge">{app.cancelableOutbox().length}</span>
            </Show>
          </button>
        </nav>

        {/* PIM modules (plan §2.5): Calendar / Tasks / Notes / Contacts, each
            reachable from the nav rail — the explicit mount step V2 lacked. */}
        <nav class="sidebar__nav sidebar__nav--apps" aria-label={t('mail-nav-apps')}>
          <For each={APP_NAV_MODULES}>
            {(m) => (
              <button
                type="button"
                class={`sidebar__box ${a11y.focusable}`}
                classList={{ 'sidebar__box--active': surface() === m.id }}
                data-testid={`nav-${m.id}`}
                onClick={() => router.navigate(m.id as ShellSurface)}
              >
                <span class="sidebar__box-icon" aria-hidden="true">{m.icon}</span>
                <span class="sidebar__box-name">{m.label}</span>
              </button>
            )}
          </For>
        </nav>
        <button type="button" class={`btn btn--ghost sidebar__logout ${a11y.focusable}`} onClick={() => void app.logout()}>
          {t('mail-logout')}
        </button>
        <Show when={!app.online()}>
          <span class="sidebar__offline" aria-live="polite">
            {t('mail-offline')}
          </span>
        </Show>
      </aside>

      <Show when={surface() === 'mail'}>
        <div class="mail-pane">
          <SubTabStrip />
          <SearchBox />
          <InboxTabs />
          <MessageList />
        </div>
        <Reader />
        {/* V7 Assist chat panel (§14.3): reasons over the open thread; proposed
            actions route to review (composer), never auto-sent. Renders NOTHING when
            the assistant capability is absent / the gateway is disabled. */}
        <AssistPanel
          config={app.assist.config()}
          service={app.assist.service}
          context={assistContext()}
          onReviewAction={() => setComposing(true)}
        />
      </Show>
      <Show when={surface() === 'outbox'}>
        <Outbox />
      </Show>
      <Show when={surface() === 'attachments'}>
        <Attachments
          load={() => app.listAttachments()}
          onOpen={(item) => {
            router.navigate('mail');
            void app.openMessage(item.emailId);
          }}
        />
      </Show>

      {/* Engine-backed PIM + key-management module surfaces, mounted (lazily)
          from the frozen registry — reachable from the nav rail above. */}
      <For each={APP_NAV_MODULES}>
        {(m) => (
          <Show when={surface() === m.id}>
            <main class="module-pane" data-surface={m.id}>
              <Suspense fallback={<div class="module-loading">{t('mail-module-loading', { module: m.label })}</div>}>
                <Dynamic component={m.mount()} />
              </Suspense>
            </main>
          </Show>
        )}
      </For>

      <Show when={composing()}>
        <Compose onClose={() => setComposing(false)} />
      </Show>
      <Show when={settingsOpen()}>
        <Settings onClose={() => setSettingsOpen(false)} />
      </Show>
      <UndoToast />
    </div>
  );
}

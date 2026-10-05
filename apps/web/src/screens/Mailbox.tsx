import { createEffect, createMemo, createSignal, For, Show, Suspense, onMount, onCleanup, type JSX } from 'solid-js';
import { Dynamic } from 'solid-js/web';
import { useApp } from '../state/context.ts';
import { t, isolate, loadCatalog } from '../i18n/index.ts';
import * as a11y from '../components/mailA11y.css.ts';
import { useRealtime } from '../realtime/context.ts';
import { MessageList } from '../components/MessageList.tsx';
import { Reader } from '../components/Reader.tsx';
import { createNarrowViewport } from '../components/narrowViewport.ts';
import { Compose } from '../components/Compose.tsx';
import type { ComposeInitial } from '../components/compose/reply.ts';
import { Outbox } from '../components/Outbox.tsx';
import { InboxTabs } from '../components/InboxTabs.tsx';
import { UndoToast } from '../components/UndoToast.tsx';
import { SubTabStrip } from '../components/SubTabStrip.tsx';
import { Ribbon } from '../components/Ribbon.tsx';
import { Settings } from './Settings.tsx';
import { SharingDialog } from './SharingDialog.tsx';
import { Attachments } from './Attachments.tsx';
import { AsyncBoundary } from '../components/ErrorBoundary.tsx';
import { AsyncPending } from '../components/AsyncState.tsx';
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
  // What the open composer was started from: a reply or forward built by the
  // reader, or `null` for a new message. Compose reads it once, when it mounts.
  const [composeInitial, setComposeInitial] = createSignal<ComposeInitial | null>(null);
  function openNewMessage(): void {
    setComposeInitial(null);
    setComposing(true);
  }
  function openReply(initial: ComposeInitial): void {
    // One composer at a time: a reply does not replace one that is open.
    if (composing()) return;
    setComposeInitial(initial);
    setComposing(true);
  }
  function closeCompose(): void {
    setComposing(false);
    setComposeInitial(null);
  }
  const [settingsOpen, setSettingsOpen] = createSignal(false);
  // t13 26.13 (E9 mount): the mailbox ACL editor, reachable from the mailbox
  // context. Open for the currently-selected mailbox; only meaningful once a
  // mailbox + account are resolved.
  const [sharingOpen, setSharingOpen] = createSignal(false);
  const shareMailbox = createMemo(() => {
    const id = app.selectedMailboxId();
    if (id === null) return null;
    const box = app.mailboxes().find((m) => m.id === id);
    return box ? { id: box.id, name: box.name } : null;
  });

  // The shell router (plan §2.5): Mail/Outbox/Attachments + the four PIM modules
  // are hash-routed surfaces, so each PIM module is reachable + deep-linkable.
  const router = createShellRouter();
  const surface = (): ShellSurface => router.route().surface;

  // Narrow-viewport shell (t28-e6). At phone width the sidebar is an off-canvas
  // drawer opened from a top bar (styles/app.css); `navOpen` is that drawer's
  // state and has no effect on the desktop grid. The bar itself only renders
  // when narrow, so the desktop DOM is the same as before.
  const narrow = createNarrowViewport();
  const [navOpen, setNavOpen] = createSignal(false);
  let sidebarEl: HTMLElement | undefined;
  let menuButton: HTMLButtonElement | undefined;
  // A drawer left open across a resize to desktop width would keep its scrim
  // state for the next narrowing; close it when the viewport widens.
  createEffect(() => {
    if (!narrow()) setNavOpen(false);
  });

  function openNav(): void {
    setNavOpen(true);
    // Into the drawer: the current destination if there is one, else its first control.
    queueMicrotask(() => {
      const target =
        sidebarEl?.querySelector<HTMLElement>('.sidebar__box--active') ??
        sidebarEl?.querySelector<HTMLElement>('button');
      target?.focus();
    });
  }

  /** Close the drawer. Focus returns to the menu button unless whatever closed
   *  it (Compose, Settings, the sharing dialog) has already taken focus. */
  function closeNav(): void {
    if (!navOpen()) return;
    setNavOpen(false);
    queueMicrotask(() => {
      const active = document.activeElement;
      if (active === null || active === document.body || sidebarEl?.contains(active) === true) {
        menuButton?.focus();
      }
    });
  }

  /** Drawer keyboard handling: Escape closes it, and Tab cycles inside it — the
   *  page behind is under the scrim, so focus must not wander into it. */
  function onNavKeyDown(e: KeyboardEvent): void {
    if (!navOpen() || sidebarEl === undefined) return;
    if (e.key === 'Escape') {
      e.preventDefault();
      closeNav();
      return;
    }
    if (e.key !== 'Tab') return;
    const stops = Array.from(sidebarEl.querySelectorAll<HTMLElement>('button:not([disabled])'));
    const first = stops[0];
    const last = stops[stops.length - 1];
    if (first === undefined || last === undefined) return;
    if (e.shiftKey && document.activeElement === first) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault();
      first.focus();
    }
  }

  // What the top bar names: the open mailbox on the mail surface, otherwise the
  // surface the nav rail navigated to.
  const barTitle = createMemo((): string => {
    const s = surface();
    if (s === 'mail') {
      const box = app.mailboxes().find((m) => m.id === app.selectedMailboxId());
      return box?.name ?? t('mail-brand');
    }
    if (s === 'outbox') return t('mail-nav-outbox');
    if (s === 'attachments') return t('mail-nav-attachments');
    return APP_NAV_MODULES.find((m) => m.id === s)?.label ?? t('mail-brand');
  });

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
        <Ribbon onCompose={openNewMessage} onOpenSettings={() => setSettingsOpen(true)} />
      </Show>
      <Show when={narrow()}>
        <header class="shell__bar">
          <button
            type="button"
            ref={menuButton}
            class={`btn btn--ghost shell__menu ${a11y.iconButton}`}
            aria-label={t('common-nav-open')}
            aria-expanded={navOpen()}
            aria-controls="shell-nav"
            onClick={openNav}
          >
            ☰
          </button>
          <span class="shell__bar-title">{barTitle()}</span>
          <button
            type="button"
            class={`btn btn--primary shell__bar-compose ${a11y.focusable}`}
            onClick={() => setComposing(true)}
          >
            {t('mail-compose')}
          </button>
        </header>
        <Show when={navOpen()}>
          <button
            type="button"
            class="shell__scrim"
            tabindex={-1}
            aria-label={t('common-nav-close')}
            onClick={closeNav}
          />
        </Show>
      </Show>
      {/* Every control in the sidebar either navigates or opens a dialog, so any
          button activated inside it also closes the narrow drawer. */}
      <aside
        class="sidebar"
        id="shell-nav"
        classList={{ 'sidebar--open': navOpen() }}
        ref={sidebarEl}
        onKeyDown={onNavKeyDown}
        onClick={(e) => {
          if (e.target instanceof Element && e.target.closest('button') !== null) closeNav();
        }}
      >
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
          {/* Share the selected folder (t13 ACL editor). Only offered once a
              mailbox + account are resolved; the editor self-gates writes on the
              caller's administer right. */}
          <Show when={shareMailbox() !== null && app.accountId() !== null}>
            <button
              type="button"
              class={`sidebar__box ${a11y.focusable}`}
              data-testid="nav-sharing"
              onClick={() => setSharingOpen(true)}
            >
              <span class="sidebar__box-name">{t('mail-nav-sharing')}</span>
            </button>
          </Show>
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
        {/* At phone width the open reader covers the list; `inert` keeps the
            covered list out of the tab order and the accessibility tree. */}
        <div class="mail-pane" inert={narrow() && app.openEmail() !== null}>
          <SubTabStrip />
          <SearchBox />
          <InboxTabs />
          <MessageList />
        </div>
        <Reader onCompose={openReply} />
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
              {/* Each module is a dynamic import. Its `Suspense` had no boundary
                  and no ceiling, so a chunk that failed to arrive left this
                  fallback on screen for the life of the tab — the nav rail said
                  Calendar, and Calendar never came. The boundary also contains a
                  module that loads and then throws, which previously took the
                  whole mailbox down with it. Retry re-renders the module; the
                  registry holds one `lazy()` per module, so a chunk failure is
                  recovered by reloading rather than by this button — which is
                  why the pending state times out into an error instead of
                  promising a retry that would replay the memoised rejection. */}
              <AsyncBoundary>
                <Suspense
                  fallback={<AsyncPending message={t('mail-module-loading', { module: m.label })} />}
                >
                  <Dynamic component={m.mount()} />
                </Suspense>
              </AsyncBoundary>
            </main>
          </Show>
        )}
      </For>

      <Show when={composing()}>
        <Compose onClose={closeCompose} {...(composeInitial() !== null ? { initial: composeInitial()! } : {})} />
      </Show>
      <Show when={settingsOpen()}>
        <Settings onClose={() => setSettingsOpen(false)} />
      </Show>
      <Show when={sharingOpen() && shareMailbox() !== null && app.accountId() !== null}>
        <SharingDialog
          mailboxId={shareMailbox()!.id}
          accountId={app.accountId()!}
          mailboxName={shareMailbox()!.name}
          onClose={() => setSharingOpen(false)}
        />
      </Show>
      <UndoToast />
    </div>
  );
}

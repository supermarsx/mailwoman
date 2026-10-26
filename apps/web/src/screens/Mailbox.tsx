import { ResizeHandle } from '../components/ResizeHandle.tsx';
import { accountUiPreferences, setAccountUiPreferences } from '../api/accountPreferences.ts';
import { readingPane } from '../components/readingPane.ts';
import { orderedFolders, folderIcon } from './Settings/folderPreferenceModel.ts';
import type { MessageView } from '../state/slices/messageView.ts';
import { createSettingsNavigation, type SettingsNavigation } from './Settings/navigation.tsx';
import '../styles/workspace.css.ts';
import { Icon } from '../components/Icon.tsx';
import { AttachmentTab, type AttachmentTabItem } from '../viewers/AttachmentTab.tsx';
import { buildDownloadUrl } from '../viewers/attachments.ts';
import type { StripItem } from '../viewers/ThumbnailStrip.tsx';
import { useBranding, brandAssetUrl } from '../branding.ts';
import { createEffect, createMemo, createSignal, untrack, For, Show, Suspense, onMount, onCleanup, type JSX } from 'solid-js';
import { Dynamic } from 'solid-js/web';
import { useApp } from '../state/context.ts';
import { createEntryMotion } from '../shell/createEntryMotion.ts';
import * as motion from '../styles/motion.css.ts';
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
import { useActions } from '../keymap/index.ts';
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
  const branding=useBranding();
  const app = useApp();
  const { subTabs } = useRealtime();
  interface Pane { id:string; kind:'messages'|'composer'|'settings'|'attachment'; title:string; view?:MessageView; initial?:ComposeInitial; navigation?:SettingsNavigation; attachment?:AttachmentTabItem }
  const [panes,setPanes]=createSignal<Pane[]>([]);
  const workspaceLayout=()=>accountUiPreferences().workspaceLayout ?? 'tabs';
  const setWorkspaceLayout=(value:'tabs'|'side-by-side'|'stack'):void=>setAccountUiPreferences({workspaceLayout:value});
  const folderMode=()=>accountUiPreferences().folderSidebarMode ?? 'expanded';
  const [folderDrag,setFolderDrag]=createSignal<number|null>(null);
  const [listDrag,setListDrag]=createSignal<number|null>(null);
  const [tileDrag,setTileDrag]=createSignal<number|null>(null);
  const [viewport,setViewport]=createSignal({width:window.innerWidth,height:window.innerHeight});
  const [bodySize,setBodySize]=createSignal({width:window.innerWidth-220,height:window.innerHeight-100});
  const [tileSize,setTileSize]=createSignal({width:window.innerWidth-520,height:window.innerHeight-100});
  let shellElement:HTMLDivElement|undefined;let bodyElement:HTMLDivElement|undefined;let tilesElement:HTMLDivElement|undefined;
  const folderMax=()=>Math.max(160,Math.min(600,viewport().width-560));
  const folderWidth=()=>folderMode()==='hidden' ? 0 : folderMode()==='compact' ? 64 : Math.max(160,Math.min(folderMax(),folderDrag() ?? accountUiPreferences().folderSidebarWidth ?? 220));
  const listMax=()=>Math.max(readingPane()==='bottom'?160:240,Math.min(1200,(readingPane()==='bottom'?bodySize().height:bodySize().width)-280));
  const listSize=()=>Math.max(readingPane()==='bottom'?160:240,Math.min(listMax(),listDrag() ?? (readingPane()==='bottom' ? accountUiPreferences().messageListHeight ?? 320 : accountUiPreferences().messageListWidth ?? 340)));
  const tileExtent=()=>Math.max(1,workspaceLayout()==='stack' ? tileSize().height : tileSize().width);
  const tileFirst=()=>tileDrag() ?? tileExtent()*(accountUiPreferences().workspaceSplitRatio ?? 0.5);
  onMount(()=>{const update=():void=>{setViewport({width:window.innerWidth,height:window.innerHeight});if(bodyElement?.clientWidth)setBodySize({width:bodyElement.clientWidth,height:bodyElement.clientHeight});if(tilesElement?.clientWidth)setTileSize({width:tilesElement.clientWidth,height:tilesElement.clientHeight});};update();window.addEventListener('resize',update);const observer=typeof ResizeObserver==='undefined'?null:new ResizeObserver(update);if(bodyElement)observer?.observe(bodyElement);if(tilesElement)observer?.observe(tilesElement);onCleanup(()=>{window.removeEventListener('resize',update);observer?.disconnect();});});
  createEffect(()=>{shellElement?.style.setProperty('--folder-width',`${folderWidth()}px`);shellElement?.style.setProperty('--folder-handle',folderMode()==='expanded'?'6px':'0px');bodyElement?.style.setProperty('--list-size',`${listSize()}px`);tilesElement?.style.setProperty('--tile-first',`${tileFirst()}px`);});
  const activePane=createMemo(()=>panes().find(pane=>pane.id===subTabs.activeId()));
  const composing=()=>activePane()?.kind==='composer';
  const listVisible=()=>!activePane() || (activePane()?.kind==='messages' && (accountUiPreferences().readerCollapsed || (!narrow() && readingPane()!=='off' && !accountUiPreferences().messageListCollapsed)));
  const readersVisible=()=>!!activePane() && !(activePane()?.kind==='messages' && accountUiPreferences().readerCollapsed);
  const splitVisible=()=>listVisible() && readersVisible();
  function openComposer(initial?:ComposeInitial):void {
    router.navigate('mail');
    const title=initial?.subject || t('mail-compose');
    const id=subTabs.open({kind:'composer',title});
    setPanes(old=>[...old,{id,kind:'composer',title,...(initial ? {initial} : {})}]);
  }
  function openNewMessage():void {openComposer();}
  function openAttachmentTab(item:StripItem):void {
    const accountId=app.accountId();const template=app.downloadUrl();
    if(!accountId || !template)return;
    router.navigate('mail');
    const id=`attachment:${accountId}:${item.blobId}`;
    if(!panes().some(pane=>pane.id===id))setPanes(old=>[...old,{id,kind:'attachment',title:item.name,attachment:{...item,downloadUrl:buildDownloadUrl(template,{accountId,blobId:item.blobId,name:item.name,mime:item.mime})}}]);
    subTabs.open({id,kind:'attachment',title:item.name});
  }
  function openReply(initial:ComposeInitial):void {openComposer(initial);}
  function openSettings():void {
    router.navigate('mail');
    const id=`settings:${app.accountId() ?? ''}`;
    if(!panes().some(pane=>pane.id===id)) setPanes(old=>[...old,{id,kind:'settings',title:t('mail-nav-settings'),navigation:createSettingsNavigation()}]);
    subTabs.open({id,kind:'settings',title:t('mail-nav-settings')});
  }
  async function openExport(messageId:string):Promise<void> {
    const account=app.accountId();
    openSettings();
    const pane=panes().find(item=>item.id===`settings:${account ?? ''}`);
    if(account===app.accountId()) await pane?.navigation?.navigate({section:'mail-transfer',editor:{kind:'edit',id:messageId}});
  }
  function openMessageTab(messageId:string):void {
    router.navigate('mail');
    const id=`message:${app.accountId() ?? ''}:${messageId}`;
    const existing=panes().find(pane=>pane.id===id);
    if(existing) {subTabs.activate(id);return;}
    const title=app.messages().find(message=>message.id===messageId)?.subject || t('mail-workspace-message');
    const view=app.createReaderView();
    setPanes(old=>[...old,{id,kind:'messages',title,view}]);
    subTabs.open({id,kind:'messages',title});
    void view.openMessage(messageId);
  }
  async function closePane(id:string):Promise<void> {
    if(id==='mail') return;
    const pane=panes().find(item=>item.id===id);
    if(pane?.navigation && !await pane.navigation.canLeave()) return;
    const wasActive=subTabs.activeId()===id;
    pane?.view?.dispose();setPanes(old=>old.filter(item=>item.id!==id));subTabs.close(id);
    if(wasActive) queueMicrotask(()=>document.getElementById(`subtab-tab-${subTabs.activeId()}`)?.focus());
  }
  createEffect(()=>{app.setActiveReader(surface()==='mail' && readersVisible() ? activePane()?.view ?? null : null);});
  let workspaceAccount=app.accountId();
  createEffect(()=>{const account=app.accountId();if(account!==workspaceAccount){for(const pane of untrack(panes)){pane.view?.dispose();subTabs.close(pane.id);}setPanes([]);workspaceAccount=account;subTabs.activate('mail');}});

  onCleanup(()=>{for(const pane of panes()) {pane.view?.dispose();subTabs.close(pane.id);}app.setActiveReader(null);});
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
  // An unknown address (`#/bogus`) is replaced by the inbox; say why (t32-h10).
  const router = createShellRouter(() => {
    void loadCatalog('errors').then(() => app.showToast('info', t('errors-mail-route-not-found')));
  });
  const surface = (): ShellSurface => router.route().surface;
  const mailEntering = createEntryMotion(surface);

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

  // Keyboard shortcuts owned by the mailbox screen (src/keymap/: one dispatcher,
  // the preset chosen in Settings). `next`/`previous` are the message list's and
  // `reply` the reader's. Each declines (returns false) off the mail surface or
  // with nothing to act on, so the key does what it would have done anyway.
  function shortcutTargets(): string[] {
    if (surface() !== 'mail') return [];
    if (subTabs.activeId() === 'mail') return app.actionTargets();
    const pane = panes().find(item => item.id === subTabs.activeId());
    const id = pane?.kind === 'messages' ? pane.view?.openEmail()?.id : undefined;
    return id ? [id] : [];
  }
  function shortcutMailAction(action: (ids: string[]) => unknown): boolean {
    const ids = shortcutTargets();
    if (!ids.length) return false;
    void action(ids); return true;
  }
  useActions({
    settings: () => { openSettings(); return true; },
    refresh: () => { if (surface() !== 'mail') return false; void app.refreshCurrentMailbox(); return true; },
    trash: () => shortcutMailAction(ids => app.trashMessage(ids)),
    markRead: () => shortcutMailAction(ids => app.markRead(ids, true)),
    markUnread: () => shortcutMailAction(ids => app.markRead(ids, false)),
    spam: () => shortcutMailAction(ids => app.markSpam(ids)),
    compose: () => {
      if (composing()) return false;
      openNewMessage();
      return true;
    },
    archive: () => {
      if (surface() !== 'mail') return false;
      const ids = shortcutTargets();
      if (ids.length === 0) return false;
      void app.archiveMessage(ids);
      return true;
    },
    search: () => {
      if (surface() !== 'mail') return false;
      const box = document.querySelector<HTMLInputElement>('.mail-search__input');
      if (box === null) return false;
      box.focus();
      return true;
    },
  });

  // Seed a single "messages" sub-tab so the multi-surface strip is live.
  onMount(() => {
    if (subTabs.tabs().length === 0) {
      subTabs.open({ kind: 'messages', title: t('common-module-mail'), id: 'mail', pinned: true });
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
    <div class="shell workspace-shell" ref={shellElement} data-folder-mode={folderMode()}>

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
            <Icon name="menu" />
          </button>
          <span class="shell__bar-title">{barTitle()}</span>
          <button
            type="button"
            class={`btn btn--primary shell__bar-compose ${a11y.focusable}`}
            onClick={openNewMessage}
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
        hidden={!narrow() && folderMode()==='hidden'}
        id="shell-nav"
        classList={{ 'sidebar--open': navOpen() }}
        ref={sidebarEl}
        onKeyDown={onNavKeyDown}
        onClick={(e) => {
          if (e.target instanceof Element && e.target.closest('button') !== null) closeNav();
        }}
      >
        <div class="sidebar__head">
          <span class="sidebar__brand"><Show when={brandAssetUrl(branding()?.logoUrl)}>{url=><img src={url()} alt={branding()?.brandName||'Mailwoman'} style={{'max-width':'3rem','max-height':'2rem','object-fit':'contain'}}/>}</Show>{branding()?.brandName || t('mail-brand')}</span>
          <Show when={app.me()}>{(m) => <span class="sidebar__user">{isolate(m().username)}</span>}</Show>
          <button
            type="button"
            class={`btn btn--ghost sidebar__settings ${a11y.iconButton}`}
            aria-label={t('mail-nav-settings')}
            onClick={openSettings}
          >
            <Icon name="settings" />
          </button>
        </div>
        <button type="button" class={`btn btn--primary sidebar__compose ${a11y.focusable}`} aria-label={t('mail-compose')} title={t('mail-compose')} onClick={openNewMessage}>
          <Icon name="edit"/><span class="sidebar__box-name">{t('mail-compose')}</span>
        </button>
        <nav class="sidebar__nav" aria-label={t('mail-nav-mailboxes')}>
          <For each={orderedFolders(app.mailboxes(),accountUiPreferences())}>
            {(box) => (
              <button
                type="button"
                class={`sidebar__box ${a11y.focusable}`}
                classList={{ 'sidebar__box--active': surface() === 'mail' && app.selectedMailboxId() === box.id }}
                aria-label={box.name} title={box.name}
                onClick={() => {
                  router.navigate('mail');
                  subTabs.activate('mail');
                  void app.selectMailbox(box.id);
                }}
              >
                <Icon name={folderIcon(box,accountUiPreferences())}/><span class="sidebar__box-name">{box.name}</span>
                <Show when={Number.isFinite(box.totalEmails)}><span class="sidebar__badge" title={t('mail-folder-counts',{unread:box.unreadEmails,total:box.totalEmails})} aria-label={t('mail-folder-counts',{unread:box.unreadEmails,total:box.totalEmails})}>{box.unreadEmails}<span class="sidebar__total"> / {box.totalEmails}</span></span></Show>
              </button>
            )}
          </For>
          <button
            type="button"
            class={`sidebar__box ${a11y.focusable}`}
            classList={{ 'sidebar__box--active': surface() === 'attachments' }} aria-label={t('mail-nav-attachments')} title={t('mail-nav-attachments')}
            onClick={() => router.navigate('attachments')}
          >
            <Icon name="attachment"/><span class="sidebar__box-name">{t('mail-nav-attachments')}</span>
          </button>
          <button
            type="button"
            class={`sidebar__box ${a11y.focusable}`}
            classList={{ 'sidebar__box--active': surface() === 'outbox' }} aria-label={t('mail-nav-outbox')} title={t('mail-nav-outbox')}
            onClick={() => {
              router.navigate('outbox');
              void app.refreshOutbox();
            }}
          >
            <Icon name="clock"/><span class="sidebar__box-name">{t('mail-nav-outbox')}</span>
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
              data-testid="nav-sharing" aria-label={t('mail-nav-sharing')} title={t('mail-nav-sharing')}
              onClick={() => setSharingOpen(true)}
            >
              <Icon name="users"/><span class="sidebar__box-name">{t('mail-nav-sharing')}</span>
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
                aria-label={m.label} title={m.label}
                data-testid={`nav-${m.id}`}
                onClick={() => router.navigate(m.id as ShellSurface)}
              >
                <span class="sidebar__box-icon" aria-hidden="true"><Icon name={m.icon} /></span>
                <span class="sidebar__box-name">{m.label}</span>
              </button>
            )}
          </For>
        </nav>
        <button type="button" class={`btn btn--ghost sidebar__logout ${a11y.focusable}`} aria-label={t('mail-logout')} title={t('mail-logout')} onClick={() => void app.logout()}>
          <Icon name="signOut"/><span class="sidebar__box-name">{t('mail-logout')}</span>
        </button>
        <Show when={!app.online()}>
          <span class="sidebar__offline" aria-live="polite">
            {t('mail-offline')}
          </span>
        </Show>
      </aside>

      <Show when={!narrow() && folderMode()==='expanded'}><ResizeHandle label={t('mail-resize-folders')} controls="shell-nav" value={folderWidth()} min={160} max={folderMax()} onChange={setFolderDrag} onCommit={value=>{setAccountUiPreferences({folderSidebarWidth:value});setFolderDrag(null);}} /></Show>
      <Show when={!narrow() && folderMode()==='hidden' && surface()!=='mail'}><button type="button" class={`btn workspace-show-folders ${a11y.iconButton}`} aria-label={t('mail-show-folders')} title={t('mail-show-folders')} onClick={()=>setAccountUiPreferences({folderSidebarMode:'expanded'})}><Icon name="folder"/></button></Show>
      <div class="mail-workspace" hidden={surface()!=='mail'}>
        <SubTabStrip onClose={id=>void closePane(id)} canClose={id=>id!=='mail'} allowTearOff={false} />
        <div class="workspace-bar" role="toolbar" aria-label={t('mail-workspace-layout')}>
          <button type="button" class={`btn btn--ghost ${a11y.iconButton}`} aria-label={t(folderMode()==='hidden'?'mail-show-folders':'mail-hide-folders')} title={t(folderMode()==='hidden'?'mail-show-folders':'mail-hide-folders')} onClick={()=>setAccountUiPreferences({folderSidebarMode:folderMode()==='hidden'?'expanded':'hidden'})}><Icon name="folder"/></button>
          <button type="button" class={`btn btn--ghost ${a11y.iconButton}`} aria-pressed={folderMode()==='compact'} aria-label={t('mail-compact-folders')} title={t('mail-compact-folders')} onClick={()=>setAccountUiPreferences({folderSidebarMode:folderMode()==='compact'?'expanded':'compact'})}><Icon name="menu"/></button>
          <button type="button" class={`btn btn--ghost ${a11y.iconButton}`} aria-pressed={!accountUiPreferences().messageListCollapsed} aria-label={t('mail-toggle-list')} title={t('mail-toggle-list')} onClick={()=>setAccountUiPreferences({messageListCollapsed:!accountUiPreferences().messageListCollapsed,readerCollapsed:false})}><Icon name="mail"/></button>
          <button type="button" class={`btn btn--ghost ${a11y.iconButton}`} aria-pressed={!accountUiPreferences().readerCollapsed} aria-label={t('mail-toggle-reader')} title={t('mail-toggle-reader')} onClick={()=>setAccountUiPreferences({readerCollapsed:!accountUiPreferences().readerCollapsed,messageListCollapsed:false})}><Icon name="eye"/></button>

          <For each={['tabs','side-by-side','stack'] as const}>{layout=><button type="button" class="btn btn--ghost" aria-pressed={workspaceLayout()===layout} onClick={()=>setWorkspaceLayout(layout)}>{t(`mail-workspace-${layout}`)}</button>}</For>
          <button type="button" class={`btn btn--ghost ${a11y.iconButton}`} title={t('mail-workspace-refresh')} aria-label={t('mail-workspace-refresh')} onClick={()=>void app.refreshCurrentMailbox()}><Icon name="refresh" /></button>
        </div>
        <Show when={app.layout()==='ribbon'}><Ribbon onCompose={openNewMessage} onOpenSettings={openSettings}/></Show>
        <div class="workspace-body" ref={bodyElement} data-reading-pane={readingPane()} classList={{'workspace-body--split':splitVisible()}}>
          <div class={`mail-pane ${motion.surface}`} hidden={!listVisible()} data-mw-entering={mailEntering()}>
            <SearchBox /><InboxTabs /><MessageList onOpenMessage={openMessageTab}/>
          </div>
          <Show when={splitVisible()}><ResizeHandle label={t('mail-resize-list')} orientation={readingPane()==='bottom'?'horizontal':'vertical'} value={listSize()} min={readingPane()==='bottom'?160:240} max={listMax()} onChange={setListDrag} onCommit={value=>{setAccountUiPreferences(readingPane()==='bottom'?{messageListHeight:value}:{messageListWidth:value});setListDrag(null);}}/></Show>
          <div class="workspace-panes" ref={tilesElement} data-layout={narrow() ? 'tabs' : workspaceLayout()} data-pair={panes().length===2} hidden={!readersVisible()}>
            <For each={panes()}>{(pane,index)=><><section class="workspace-pane" role="tabpanel" aria-label={pane.title}
              hidden={subTabs.activeId()==='mail' || ((narrow() || workspaceLayout()==='tabs') && subTabs.activeId()!==pane.id)}
              inert={subTabs.activeId()==='mail' || ((narrow() || workspaceLayout()==='tabs') && subTabs.activeId()!==pane.id)}
              onFocusIn={()=>subTabs.activate(pane.id)} onPointerDown={()=>subTabs.activate(pane.id)}>
              <Show when={pane.kind==='messages' && pane.view}>{view=><><Show when={view().error()}><p role="alert">{t('mail-workspace-load-error')}</p></Show><Reader view={view()} active={subTabs.activeId()===pane.id} onClose={()=>void closePane(pane.id)} onCompose={openReply} onOpenAttachment={openAttachmentTab} onExport={messageId=>void openExport(messageId)}/></>}</Show>
              <Show when={pane.kind==='attachment' && pane.attachment}>{item=><AttachmentTab item={item()}/>}</Show>
              <Show when={pane.kind==='composer'}><Compose embedded active={subTabs.activeId()===pane.id} onClose={()=>void closePane(pane.id)} {...(pane.initial ? {initial:pane.initial} : {})}/></Show>
              <Show when={pane.kind==='settings' && pane.navigation}>{navigation=><Settings embedded navigation={navigation()} onClose={()=>void closePane(pane.id)}/>}</Show>
            </section><Show when={index()===0 && panes().length===2 && !narrow() && workspaceLayout()!=='tabs'}><ResizeHandle label={t('mail-resize-tiles')} orientation={workspaceLayout()==='stack'?'horizontal':'vertical'} value={tileFirst()} min={tileExtent()*0.2} max={tileExtent()*0.8} onChange={setTileDrag} onCommit={value=>{setAccountUiPreferences({workspaceSplitRatio:Math.max(0.2,Math.min(0.8,value/tileExtent()))});setTileDrag(null);}}/></Show></>}</For>
          </div>
        </div>
        <AssistPanel config={app.assist.config()} service={app.assist.service} context={assistContext()} onReviewAction={openNewMessage}/>
      </div>
      <Show when={surface() === 'outbox'}>
        <Outbox />
      </Show>
      <Show when={surface() === 'attachments'}>
        <Attachments
          load={() => app.listAttachments()}
          onOpen={(item) => {
            router.navigate('mail');
            openMessageTab(item.emailId);
          }}
        />
      </Show>

      {/* Engine-backed PIM + key-management module surfaces, mounted (lazily)
          from the frozen registry — reachable from the nav rail above. */}
      <For each={APP_NAV_MODULES}>
        {(m) => (
          <Show when={surface() === m.id}>
            <main class="module-pane" data-surface={m.id} data-mw-surface>
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

      <Show when={sharingOpen() && shareMailbox() !== null && app.accountId() !== null}>
        <SharingDialog
          mailboxId={shareMailbox()!.id}
          accountId={app.accountId()!}
          mailboxName={shareMailbox()!.name}
          onClose={() => setSharingOpen(false)}
        />
      </Show>
    </div>
  );
}

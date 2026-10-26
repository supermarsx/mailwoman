import {DetachedWorkspacePane,type DetachedPaneHandle} from '../components/DetachedWorkspacePane.tsx';
import {WorkspaceLayoutControls} from '../components/WorkspaceLayoutControls.tsx';
import {createWorkspaceViews,isSplitLayout,isStackedLayout,workspaceLayout as parseWorkspaceLayout,type WorkspaceLayout,type WorkspaceViewSlot} from '../realtime/workspaceLayout.ts';
import {workspaceTabDragId,endWorkspaceTabDrag} from '../realtime/workspaceTabDrag.ts';
import {resolveWorkspaceAttachment} from '../realtime/workspaceAttachment.ts';
import { settings, settingsLoadStatus } from '../api/settings.ts';
import { MAX_WORKSPACE_TABS, readWorkspace, saveWorkspace, clearWorkspace, type WorkspaceReference } from '../realtime/workspaceStorage.ts';
import {accountDraftApi,localAccountDraft} from '../api/accountDrafts.ts';
import { listDrafts,saveDraft } from '../components/compose/drafts-store.ts';
import { FolderCountBadge } from '../components/FolderCountBadge.tsx';
import { PaneNavigationContext, confirmShellNavigation } from '../shell/navigationGuard.ts';
import { preferenceAccount, preferenceAccountIsCurrent, observePreferenceAccount } from '../api/accountPreferenceLifecycle.ts';
import { AppearanceCustomizationRuntime } from '../components/AppearanceCustomizationRuntime.tsx';
import {createSharingAccess} from '../api/mailboxSharing.ts';
import {createConfiguredClient} from '../api/transport.ts';
import {FolderContextMenu} from '../components/FolderContextMenu.tsx';
import {peekMessageDrag,takeMessageDrag} from '../components/messageDrag.ts';
import {folderDropTarget} from '../components/messageDrag.css.ts';
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
  interface Pane { detach?:DetachedPaneHandle; beforeClose?:()=>Promise<boolean>; id:string; kind:'messages'|'composer'|'settings'|'attachment'|'module'|'outbox'|'attachments'|'sharing'; title:string; surface?:ShellSurface; reference?:string; draftId?:string; accountId?:string; view?:MessageView; initial?:ComposeInitial; navigation?:SettingsNavigation; attachment?:AttachmentTabItem }
  const [panes,setPanes]=createSignal<Pane[]>([]);
  const [workspaceReady,setWorkspaceReady]=createSignal(false);
  const [draftRevision,setDraftRevision]=createSignal(0);
  const [workspaceOwner,setWorkspaceOwner]=createSignal(preferenceAccount());
  const workspaceLayout=()=>parseWorkspaceLayout(accountUiPreferences().workspaceLayout);
  const setWorkspaceLayout=(value:WorkspaceLayout):void=>setAccountUiPreferences({workspaceLayout:value});
  const folderMode=()=>accountUiPreferences().folderSidebarMode ?? 'expanded';
  const [folderDrag,setFolderDrag]=createSignal<number|null>(null);
  const [dropFolder,setDropFolder]=createSignal<string|null>(null);
  function acceptsMessageDrop(event:DragEvent,id:string):boolean {
    if(app.listLoading())return false;
    const box=app.mailboxes().find(item=>item.id===id);
    const ids=peekMessageDrag(event,app.accountId());
    return !!box&&box.mailwomanSearchQuery===undefined&&!!ids&&ids.every(messageId=>app.messages().some(message=>message.id===messageId));
  }
  function dropMessages(event:DragEvent,id:string):void {
    setDropFolder(null);if(!acceptsMessageDrop(event,id))return;
    const ids=takeMessageDrag(event,app.accountId());if(!ids)return;
    event.preventDefault();
    const moving=ids.filter(messageId=>!app.messages().find(message=>message.id===messageId)?.mailboxIds[id]);
    if(moving.length)void app.moveMessage(moving,id);
  }
  onMount(()=>{const clear=()=>setDropFolder(null);window.addEventListener('dragend',clear);onCleanup(()=>window.removeEventListener('dragend',clear));});

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
  const tileExtent=()=>Math.max(1,isStackedLayout(workspaceLayout()) ? tileSize().height : tileSize().width);
  const tileFirst=()=>tileDrag() ?? tileExtent()*(accountUiPreferences().workspaceSplitRatio ?? 0.5);
  onMount(()=>{const update=():void=>{setViewport({width:window.innerWidth,height:window.innerHeight});if(bodyElement?.clientWidth)setBodySize({width:bodyElement.clientWidth,height:bodyElement.clientHeight});if(tilesElement?.clientWidth)setTileSize({width:tilesElement.clientWidth,height:tilesElement.clientHeight});};update();window.addEventListener('resize',update);const observer=typeof ResizeObserver==='undefined'?null:new ResizeObserver(update);if(bodyElement)observer?.observe(bodyElement);if(tilesElement)observer?.observe(tilesElement);onCleanup(()=>{window.removeEventListener('resize',update);observer?.disconnect();});});
  createEffect(()=>{shellElement?.style.setProperty('--folder-width',`${folderWidth()}px`);shellElement?.style.setProperty('--folder-handle',folderMode()==='expanded'?'6px':'0px');bodyElement?.style.setProperty('--list-size',`${listSize()}px`);tilesElement?.style.setProperty('--tile-first',`${tileFirst()}px`);});
  const orderedPanes=createMemo(()=>{const order=subTabs.tabs().map(tab=>tab.id);return [...panes()].sort((a,b)=>order.indexOf(a.id)-order.indexOf(b.id));});
  const [paneWindowRevision,setPaneWindowRevision]=createSignal(0);
  const views=createWorkspaceViews({ids:()=>{paneWindowRevision();return orderedPanes().filter(pane=>!pane.detach?.isDetached()).map(pane=>pane.id);},activeId:subTabs.activeId,owner:()=>{const owner=workspaceOwner();return owner?.storageAuthority?JSON.stringify([owner.id,owner.storageAuthority,owner.generation]):null;},activate:subTabs.activate});
  const effectiveLayout=():WorkspaceLayout=>narrow()?'tabs':workspaceLayout();
  const paneVisible=(id:string):boolean=>{paneWindowRevision();if(subTabs.activeId()==='mail')return false;if(effectiveLayout()==='tabs')return subTabs.activeId()===id;return !panes().find(pane=>pane.id===id)?.detach?.isDetached()&&(!isSplitLayout(effectiveLayout())||views.slot(id)!==null);};
  const [dropView,setDropView]=createSignal<WorkspaceViewSlot|null>(null);
  function dragView(event:DragEvent,slot:WorkspaceViewSlot|null,drop=false):void {
    if(slot===null||!isSplitLayout(effectiveLayout()))return;
    const id=workspaceTabDragId(event,drop);
    const pane=id?panes().find(pane=>pane.id===id):undefined;
    if(!id||!pane)return;
    event.preventDefault();
    if(drop){setDropView(null);pane.detach?.reattach();views.assignView(id,slot);endWorkspaceTabDrag();}
    else {setDropView(slot);if(event.dataTransfer)event.dataTransfer.dropEffect='move';}
  }
  onMount(()=>{const clear=()=>setDropView(null);window.addEventListener('dragend',clear);onCleanup(()=>window.removeEventListener('dragend',clear));});
  const activePane=createMemo(()=>panes().find(pane=>pane.id===subTabs.activeId()));
  const composing=()=>activePane()?.kind==='composer';
  const listVisible=()=>!activePane() || (activePane()?.kind==='messages' && (accountUiPreferences().readerCollapsed || (!narrow() && readingPane()!=='off' && !accountUiPreferences().messageListCollapsed)));
  const readersVisible=()=>!!activePane() && !(activePane()?.kind==='messages' && accountUiPreferences().readerCollapsed);
  const splitVisible=()=>listVisible() && readersVisible();
  let restoreIntent=0;
  function noteWorkspaceIntent():void {restoreIntent+=1;}
  function canOpenPane(id?:string):boolean {if(id&&panes().some(pane=>pane.id===id))return true;if(panes().length<MAX_WORKSPACE_TABS-1)return true;app.showToast('info',t('mail-workspace-limit'));return false;}
  function openComposer(initial?:ComposeInitial,resumeDraftId?:string,restoredId?:string):void {
    if(!restoredId)noteWorkspaceIntent();
    if(!canOpenPane(restoredId))return;
    router.navigate('mail');
    const title=initial?.subject || t('mail-compose');
    const id=subTabs.open({kind:'composer',title,...(restoredId?{id:restoredId}:{})});
    setPanes(old=>[...old,{id,kind:'composer',title,...(resumeDraftId?{draftId:resumeDraftId}:{}),...(initial ? {initial} : {})}]);
  }
  function openNewMessage():void {openComposer();}
  function openAttachmentTab(item:StripItem,restoring=false):void {
    if(!restoring)noteWorkspaceIntent();
    const accountId=app.accountId();const template=app.downloadUrl();
    if(!accountId || !template)return;
    router.navigate('mail');
    const id=`attachment:${accountId}:${item.blobId}`;if(!canOpenPane(id))return;
    if(!panes().some(pane=>pane.id===id))setPanes(old=>[...old,{id,kind:'attachment',title:item.name,attachment:{...item,downloadUrl:buildDownloadUrl(template,{accountId,blobId:item.blobId,name:item.name,mime:item.mime})}}]);
    subTabs.open({id,kind:'attachment',title:item.name});
  }
  function openReply(initial:ComposeInitial):void {openComposer(initial);}
  function openSettings(restoring=false):void {
    if(!restoring)noteWorkspaceIntent();
    router.navigate('mail');
    const id=`settings:${app.accountId() ?? ''}`;if(!canOpenPane(id))return;
    if(!panes().some(pane=>pane.id===id)) setPanes(old=>[...old,{id,kind:'settings',title:t('mail-nav-settings'),navigation:createSettingsNavigation()}]);
    subTabs.open({id,kind:'settings',title:t('mail-nav-settings')});
  }
  async function openExport(messageId:string):Promise<void> {
    const account=app.accountId();
    openSettings();
    const pane=panes().find(item=>item.id===`settings:${account ?? ''}`);
    if(account===app.accountId()) await pane?.navigation?.navigate({section:'mail-transfer',editor:{kind:'edit',id:messageId}});
  }
  async function openDraftOrMessage(messageId:string):Promise<void>{
    const owner=preferenceAccount(),message=app.messages().find(item=>item.id===messageId);
    if(owner?.storageAuthority&&message?.keywords?.['$draft']){
      try{const result=await accountDraftApi(owner).resolve(messageId);if(!preferenceAccountIsCurrent(owner))return;if(result.draft?.state){const draft=localAccountDraft(result.draft);saveDraft(draft);openComposer(undefined,draft.id);return;}}
      catch{if(!preferenceAccountIsCurrent(owner))return;}
    }
    openMessageTab(messageId);
  }
  function openMessageTab(messageId:string,restoring=false):void {
    if(!restoring)noteWorkspaceIntent();
    router.navigate('mail');
    const id=`message:${app.accountId() ?? ''}:${messageId}`;if(!canOpenPane(id))return;
    const existing=panes().find(pane=>pane.id===id);
    if(existing) {subTabs.activate(id);return;}
    const title=app.messages().find(message=>message.id===messageId)?.subject || t('mail-workspace-message');
    const view=app.createReaderView();
    setPanes(old=>[...old,{id,kind:'messages',title,view,reference:messageId}]);
    subTabs.open({id,kind:'messages',title});
    void view.openMessage(messageId);
  }
  async function closePane(id:string):Promise<void> {
    if(id==='mail') return;
    noteWorkspaceIntent();
    const pane=panes().find(item=>item.id===id),owner=preferenceAccount();
    pane?.detach?.reattach();
    if(pane?.beforeClose && !await pane.beforeClose()) return;
    if(pane?.navigation && !await pane.navigation.canLeave()) return;
    if(!await confirmShellNavigation(id)) return;
    if((owner&&!preferenceAccountIsCurrent(owner))||panes().find(item=>item.id===id)!==pane)return;
    const wasActive=subTabs.activeId()===id;
    pane?.view?.dispose();setPanes(old=>old.filter(item=>item.id!==id));subTabs.close(id);
    if(wasActive) queueMicrotask(()=>document.getElementById(`subtab-tab-${subTabs.activeId()}`)?.focus());
  }
  createEffect(()=>{app.setActiveReader(surface()==='mail' && readersVisible() ? activePane()?.view ?? null : null);});
  let workspaceGeneration:number|undefined;
  onCleanup(observePreferenceAccount(owner=>{if(owner?.generation===workspaceGeneration)return;workspaceGeneration=owner?.generation;restoreIntent=0;setWorkspaceReady(false);for(const pane of untrack(panes)){pane.view?.dispose();subTabs.close(pane.id);}setPanes([]);subTabs.activate('mail');setWorkspaceOwner(owner);}));

  onCleanup(()=>{for(const pane of panes()) {pane.view?.dispose();subTabs.close(pane.id);}app.setActiveReader(null);});
  // t13 26.13 (E9 mount): the mailbox ACL editor, reachable from the mailbox
  // context. Open for the currently-selected mailbox; only meaningful once a
  // mailbox + account are resolved.
  const sharingAccess=createSharingAccess(app.accountId,createConfiguredClient());
  createEffect(()=>{const id=app.selectedMailboxId(),account=app.accountId();if(id&&account)untrack(()=>{const box=app.mailboxes().find(value=>value.id===id);if(box&&box.mailwomanSearchQuery===undefined)void sharingAccess.probe(id,true);});});
  async function openSharing(mailboxId:string,stillCurrent:()=>boolean=()=>true,restoring=false):Promise<void>{
    if(!restoring)noteWorkspaceIntent();
    const accountId=app.accountId();
    if(!accountId||!await sharingAccess.probe(mailboxId,true)||accountId!==app.accountId()||!sharingAccess.canShare(mailboxId)){if(!stillCurrent())return;app.showToast('error',t('mail-sharing-unavailable'));return;}
    if(!stillCurrent())return;
    const box=app.mailboxes().find(value=>value.id===mailboxId);if(!box)return;
    const id=`sharing:${accountId}:${mailboxId}`;if(!canOpenPane(id))return;router.navigate('mail');
    if(!panes().some(pane=>pane.id===id))setPanes(old=>[...old,{id,kind:'sharing',title:box.name,reference:mailboxId,accountId}]);
    subTabs.open({id,kind:'sharing',title:box.name});
  }

  // The shell router (plan §2.5): Mail/Outbox/Attachments + the four PIM modules
  // are hash-routed surfaces, so each PIM module is reachable + deep-linkable.
  // An unknown address (`#/bogus`) is replaced by the inbox; say why (t32-h10).
  const router = createShellRouter(() => {
    void loadCatalog('errors').then(() => app.showToast('info', t('errors-mail-route-not-found')));
  },{retainedSurfaces:true,onExternalNavigation:()=>noteWorkspaceIntent()});
  const surface = (): ShellSurface => router.route().surface;
  createEffect(()=>{const destination=surface();untrack(()=>{
    if(destination==='mail'){if(activePane()?.surface)subTabs.activate('mail');return;}
    const accountId=app.accountId();if(!accountId)return;
    const module=APP_NAV_MODULES.find(item=>item.id===destination);
    const kind=destination==='attachments'?'attachments':destination==='outbox'?'outbox':'module';
    const title=module?.label??t(destination==='attachments'?'mail-nav-attachments':'mail-nav-outbox');
    const id=`surface:${accountId}:${destination}`;if(!canOpenPane(id)){router.navigate(activePane()?.surface??'mail');return;}
    if(!panes().some(pane=>pane.id===id))setPanes(old=>[...old,{id,kind,title,surface:destination,reference:destination}]);
    subTabs.open({id,kind,title});
  });});
  createEffect(()=>{const id=subTabs.activeId();untrack(()=>{const destination=panes().find(pane=>pane.id===id)?.surface??'mail';if(surface()!==destination)router.navigate(destination);});});
  let restoringGeneration:number|undefined;
  createEffect(()=>{
    const owner=workspaceOwner(),status=settingsLoadStatus();const enabled=settings()['mail.resumeTabs']?.value!==false;
    if(!enabled){clearWorkspace();setWorkspaceReady(true);return;}
    if(!owner?.storageAuthority||owner.id!==app.accountId()||status!=='ready'||restoringGeneration===owner.generation)return;
    restoringGeneration=owner.generation;
    const intent=restoreIntent;
    const restoreCurrent=()=>preferenceAccountIsCurrent(owner)&&restoreIntent===intent&&settings()['mail.resumeTabs']?.value!==false;
    const saved=readWorkspace(owner.id,owner.storageAuthority);
    void (async()=>{
      try {
        if(!saved||intent>0)return;
        for(const tab of saved.tabs){
          if(!restoreCurrent())return;
          if(tab.id==='mail')continue;
          try {
          if(tab.kind==='messages'&&tab.reference)openMessageTab(tab.reference,true);
          else if(tab.kind==='settings')openSettings(true);
          else if(tab.kind==='composer'&&tab.reference&&listDrafts(owner.id,owner.storageAuthority).some(draft=>draft.id===tab.reference))openComposer(undefined,tab.reference,tab.id);
          else if(tab.kind==='sharing'&&tab.reference)await openSharing(tab.reference,restoreCurrent,true);
          else if(tab.kind==='module'||tab.kind==='outbox'||tab.kind==='attachments'){
            router.navigate((tab.kind==='module'?tab.reference:tab.kind) as ShellSurface);
            await Promise.resolve();
          } else if(tab.kind==='attachment'&&tab.reference&&tab.messageId){
            const item=await resolveWorkspaceAttachment(createConfiguredClient(),owner.id,tab.messageId,tab.reference);
            if(!restoreCurrent())return;
            if(item)openAttachmentTab(item,true);
          }

          const current=subTabs.tabs().find(value=>value.id===tab.id);
          if(current&&current.pinned!==tab.pinned)subTabs.togglePin(current.id);
          }catch{/* Removed or unauthorized references are omitted. */}
        }
        if(!restoreCurrent())return;
        saved.tabs.forEach((tab,index)=>subTabs.move(tab.id,index));
        subTabs.activate(saved.activeId);
        views.restore(saved.viewAssignments);
      } finally {if(preferenceAccountIsCurrent(owner))setWorkspaceReady(true);}
    })().catch(()=>{/* A stale/denied reference is never replaced by cached content. */});
  });
  createEffect(()=>{
    const owner=workspaceOwner();draftRevision();const tabs=subTabs.tabs(),activeId=subTabs.activeId();const currentPanes=panes();
    if(!workspaceReady()||!owner?.storageAuthority||!preferenceAccountIsCurrent(owner)||settingsLoadStatus()!=='ready'||settings()['mail.resumeTabs']?.value===false)return;
    const references:WorkspaceReference[]=tabs.slice(0,32).map(tab=>{
      const pane=currentPanes.find(value=>value.id===tab.id);
      const reference=pane?.kind==='composer'?pane.draftId:pane?.attachment?.blobId??pane?.reference;
      return {id:tab.id,kind:tab.kind,pinned:tab.pinned,...(reference?{reference}:{}),...(pane?.attachment?.emailId?{messageId:pane.attachment.emailId}:{})};
    });
    saveWorkspace({version:2,accountId:owner.id,storageAuthority:owner.storageAuthority,tabs:references,activeId:activeId??'mail',layout:workspaceLayout(),viewAssignments:views.assignments()});
  });
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
    <div class="shell workspace-shell mw-custom-mail" onPointerDown={noteWorkspaceIntent} onClick={noteWorkspaceIntent} onKeyDown={noteWorkspaceIntent} ref={shellElement} data-folder-mode={folderMode()}>
      <AppearanceCustomizationRuntime owner={() => {const accountId = app.accountId(); return accountId ? {kind: 'mailbox', accountId} : null;}} />

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
          <span class="sidebar__brand"><Show when={brandAssetUrl(branding()?.logoUrl)}>{url=><img src={url()} alt={branding()?.brandName||'Mailwoman'} class="sidebar__brand-logo"/>}</Show>{branding()?.brandName || t('mail-brand')}</span>
          <Show when={app.me()}>{(m) => <span class="sidebar__user">{isolate(m().username)}</span>}</Show>
        </div>
        <button type="button" class={`btn btn--primary sidebar__compose ${a11y.focusable}`} aria-label={t('mail-compose')} title={t('mail-compose')} onClick={openNewMessage}>
          <Icon name="edit"/><span class="sidebar__box-name">{t('mail-compose')}</span>
        </button>
        <div class="sidebar__scroll">
        <nav class="sidebar__nav" aria-label={t('mail-nav-mailboxes')}>
          <For each={orderedFolders(app.mailboxes(),accountUiPreferences()).map(box=>box.id)}>
            {(id) => {const box=createMemo(()=>app.mailboxes().find(value=>value.id===id)!);return (
              <FolderContextMenu box={box()} onOpen={()=>{router.navigate('mail');subTabs.activate('mail');void app.selectMailbox(box().id);}} onSettings={()=>{openSettings();const pane=panes().find(item=>item.id===`settings:${app.accountId() ?? ''}`);void pane?.navigation?.navigate({section:'folders'});}} canShare={()=>sharingAccess.canShare(box().id)} onProbe={()=>void sharingAccess.probe(box().id,true)} onShare={()=>void openSharing(box().id)}>
              <button
                type="button"
                class={`sidebar__box ${a11y.focusable}`}
                classList={{ 'sidebar__box--active': surface() === 'mail' && app.selectedMailboxId() === box().id,[folderDropTarget]:dropFolder()===box().id }}
                onDragOver={event=>{if(acceptsMessageDrop(event,box().id)){event.preventDefault();if(event.dataTransfer)event.dataTransfer.dropEffect='move';setDropFolder(box().id);}}}
                onDragLeave={()=>setDropFolder(null)} onDrop={event=>dropMessages(event,box().id)}
                aria-label={box().name} title={[box().name,...(accountUiPreferences().folderLabels?.[box().id]??[]).map(id=>app.tagByKeyword(id)?.name??id)].join(' · ')}
                onClick={() => {
                  router.navigate('mail');
                  subTabs.activate('mail');
                  void app.selectMailbox(box().id);
                }}
              >
                <Icon name={folderIcon(box(),accountUiPreferences())}/><span class="sidebar__box-name">{box().name}</span>
                <FolderCountBadge folder={box()}/>
              </button>
              </FolderContextMenu>
            );}}
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
          <Show when={sharingAccess.canShare(app.selectedMailboxId()??'') && app.accountId() !== null}>
            <button
              type="button"
              class={`sidebar__box ${a11y.focusable}`}
              data-testid="nav-sharing" aria-label={t('mail-nav-sharing')} title={t('mail-nav-sharing')}
              onClick={() => {const id=app.selectedMailboxId();if(id)void openSharing(id);}}
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
        </div>
        <div class="sidebar__footer">
        <button type="button" class={`btn btn--ghost sidebar__settings ${a11y.focusable}`} aria-label={t('mail-nav-settings')} data-tooltip={t('mail-nav-settings')} onClick={()=>openSettings()}>
          <Icon name="settings"/><span class="sidebar__box-name">{t('mail-nav-settings')}</span>
        </button>
        <button type="button" class={`btn btn--ghost sidebar__logout ${a11y.focusable}`} aria-label={t('mail-logout')} title={t('mail-logout')} onClick={() => void app.logout()}>
          <Icon name="signOut"/><span class="sidebar__box-name">{t('mail-logout')}</span>
        </button>
        <Show when={!app.online()}>
          <span class="sidebar__offline" aria-live="polite">
            {t('mail-offline')}
          </span>
        </Show>
        </div>
      </aside>

      <Show when={!narrow() && folderMode()==='expanded'}><ResizeHandle label={t('mail-resize-folders')} controls="shell-nav" value={folderWidth()} min={160} max={folderMax()} onChange={setFolderDrag} onCommit={value=>{setAccountUiPreferences({folderSidebarWidth:value});setFolderDrag(null);}} /></Show>
      <Show when={!narrow() && folderMode()==='hidden' && surface()!=='mail'}><button type="button" class={`btn workspace-show-folders ${a11y.iconButton}`} aria-label={t('mail-show-folders')} title={t('mail-show-folders')} onClick={()=>setAccountUiPreferences({folderSidebarMode:'expanded'})}><Icon name="folder"/></button></Show>
      <div class="mail-workspace">
        <SubTabStrip onClose={id=>void closePane(id)} canClose={id=>id!=='mail'} canDetach={id=>id!=='mail'} onDetach={id=>{const pane=panes().find(value=>value.id===id);subTabs.activate(id);pane?.detach?.detach();}} />
        <div class="workspace-bar" role="toolbar" aria-label={t('mail-workspace-layout')}>
          <button type="button" class={`btn btn--ghost ${a11y.iconButton}`} aria-label={t(folderMode()==='hidden'?'mail-show-folders':'mail-hide-folders')} title={t(folderMode()==='hidden'?'mail-show-folders':'mail-hide-folders')} onClick={()=>setAccountUiPreferences({folderSidebarMode:folderMode()==='hidden'?'expanded':'hidden'})}><Icon name="folder"/></button>
          <button type="button" class={`btn btn--ghost ${a11y.iconButton}`} aria-pressed={folderMode()==='compact'} aria-label={t('mail-compact-folders')} title={t('mail-compact-folders')} onClick={()=>setAccountUiPreferences({folderSidebarMode:folderMode()==='compact'?'expanded':'compact'})}><Icon name="menu"/></button>
          <button type="button" class={`btn btn--ghost ${a11y.iconButton}`} aria-pressed={!accountUiPreferences().messageListCollapsed} aria-label={t('mail-toggle-list')} title={t('mail-toggle-list')} onClick={()=>setAccountUiPreferences({messageListCollapsed:!accountUiPreferences().messageListCollapsed,readerCollapsed:false})}><Icon name="mail"/></button>
          <button type="button" class={`btn btn--ghost ${a11y.iconButton}`} aria-pressed={!accountUiPreferences().readerCollapsed} aria-label={t('mail-toggle-reader')} title={t('mail-toggle-reader')} onClick={()=>setAccountUiPreferences({readerCollapsed:!accountUiPreferences().readerCollapsed,messageListCollapsed:false})}><Icon name="eye"/></button>

          <WorkspaceLayoutControls value={workspaceLayout()} onChange={setWorkspaceLayout}/>
          <button type="button" class={`btn btn--ghost ${a11y.iconButton}`} title={t('mail-workspace-refresh')} aria-label={t('mail-workspace-refresh')} onClick={()=>void app.refreshCurrentMailbox()}><Icon name="refresh" /></button>
        </div>
        <Show when={app.layout()==='ribbon'}><Ribbon onCompose={openNewMessage} onOpenSettings={openSettings}/></Show>
        <div class="workspace-body" ref={bodyElement} data-reading-pane={readingPane()} classList={{'workspace-body--split':splitVisible()}}>
          <div class={`mail-pane ${motion.surface}`} hidden={!listVisible()} data-mw-entering={mailEntering()}>
            <SearchBox /><InboxTabs /><MessageList onOpenMessage={id=>void openDraftOrMessage(id)} onCompose={openReply}/>
          </div>
          <Show when={splitVisible()}><ResizeHandle label={t('mail-resize-list')} orientation={readingPane()==='bottom'?'horizontal':'vertical'} value={listSize()} min={readingPane()==='bottom'?160:240} max={listMax()} onChange={setListDrag} onCommit={value=>{setAccountUiPreferences(readingPane()==='bottom'?{messageListHeight:value}:{messageListWidth:value});setListDrag(null);}}/></Show>
          <div class="workspace-panes" ref={tilesElement} data-layout={effectiveLayout()} data-pair={isSplitLayout(effectiveLayout())} hidden={!readersVisible()}>
            <For each={orderedPanes()}>{pane=><PaneNavigationContext.Provider value={pane.id}><section class="workspace-pane" role="tabpanel" aria-label={subTabs.tabs().find(tab=>tab.id===pane.id)?.title??pane.title}
              hidden={!paneVisible(pane.id)}
              inert={!paneVisible(pane.id)} data-tab-id={pane.id} data-view-slot={views.slot(pane.id)??undefined} data-drop-target={dropView()!==null&&dropView()===views.slot(pane.id)}
              onDragOver={event=>dragView(event,views.slot(pane.id))} onDrop={event=>dragView(event,views.slot(pane.id),true)} onDragLeave={()=>setDropView(null)}
              onFocusIn={()=>subTabs.activate(pane.id)} onPointerDown={()=>subTabs.activate(pane.id)}>
              <DetachedWorkspacePane title={()=>subTabs.tabs().find(tab=>tab.id===pane.id)?.title??pane.title} register={handle=>{pane.detach=handle;setPaneWindowRevision(value=>value+1);}}>{()=> <>
              <Show when={pane.kind==='messages' && pane.view}>{view=><><Show when={view().error()}><p role="alert">{t('mail-workspace-load-error')}</p></Show><Reader view={view()} active={subTabs.activeId()===pane.id} onClose={()=>void closePane(pane.id)} onCompose={openReply} onOpenAttachment={openAttachmentTab} onExport={messageId=>void openExport(messageId)}/></>}</Show>
              <Show when={pane.kind==='attachment' && pane.attachment}>{item=><AttachmentTab item={item()}/>}</Show>
              <Show when={pane.kind==='composer'}><Compose embedded registerBeforeClose={handler=>{pane.beforeClose=handler;}} onSubjectChange={subject=>subTabs.rename(pane.id,subject.trim()||t('mail-compose'))} onDraftIdChange={id=>{if(pane.draftId!==id){pane.draftId=id;setDraftRevision(value=>value+1);}}} {...(pane.draftId?{resumeDraftId:pane.draftId}:{})} active={subTabs.activeId()===pane.id} onClose={()=>void closePane(pane.id)} {...(pane.initial ? {initial:pane.initial} : {})}/></Show>
              <Show when={pane.kind==='outbox'}><Outbox/></Show>
              <Show when={pane.kind==='attachments'}><Attachments loadPage={position=>app.listAttachmentsPage(position)} onOpen={openAttachmentTab} onOpenMessage={item=>openMessageTab(item.emailId)}/></Show>
              <Show when={pane.kind==='sharing'&&pane.reference&&pane.accountId}><SharingDialog embedded mailboxId={pane.reference!} accountId={pane.accountId!} mailboxName={pane.title} onClose={()=>void closePane(pane.id)}/></Show>
              <Show when={pane.kind==='module'&&APP_NAV_MODULES.find(module=>module.id===pane.surface)}>{module=><main class="module-pane" data-surface={module().id} data-mw-surface><AsyncBoundary><Suspense fallback={<AsyncPending message={t('mail-module-loading',{module:module().label})}/>}><Dynamic component={module().mount()}/></Suspense></AsyncBoundary></main>}</Show>
              <Show when={pane.kind==='settings' && pane.navigation}>{navigation=><Settings embedded navigation={navigation()} onClose={()=>void closePane(pane.id)}/>}</Show>
              </>}</DetachedWorkspacePane>
            </section></PaneNavigationContext.Provider>}</For>
            <Show when={isSplitLayout(effectiveLayout())}>
              <ResizeHandle label={t('mail-resize-tiles')} orientation={workspaceLayout()==='stack'?'horizontal':'vertical'} value={tileFirst()} min={tileExtent()*0.2} max={tileExtent()*0.8} onChange={setTileDrag} onCommit={value=>{setAccountUiPreferences({workspaceSplitRatio:Math.max(0.2,Math.min(0.8,value/tileExtent()))});setTileDrag(null);}}/>
              <For each={[0,1] as const}>{slot=><Show when={views.assignments()[slot]===null}><div class="workspace-view-empty" data-view-slot={slot} data-drop-target={dropView()===slot} role="region" aria-label={t(slot===0?'mail-workspace-view-first':'mail-workspace-view-second')} onDragOver={event=>dragView(event,slot)} onDrop={event=>dragView(event,slot,true)} onDragLeave={()=>setDropView(null)}><Icon name="mail"/><p>{t('mail-workspace-view-empty')}</p></div></Show>}</For>
            </Show>
          </div>
        </div>
        <AssistPanel config={app.assist.config()} service={app.assist.service} context={assistContext()} onReviewAction={openNewMessage}/>
      </div>

    </div>
  );
}

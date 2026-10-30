import {LinkedAccountControl} from '../components/accounts/LinkedAccountControl.tsx';
import {accountIndicatorLabel} from '../components/accounts/MailboxPicker.tsx';
import {mailboxNickname} from '../api/mailboxLayout.ts';
import {CollapsibleBar} from '../components/bars/index.ts';
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
import { QuotaMeter } from '../components/QuotaMeter.tsx';
import { PaneNavigationContext, confirmShellNavigation } from '../shell/navigationGuard.ts';
import { preferenceAccount, preferenceAccountIsCurrent, observePreferenceAccount } from '../api/accountPreferenceLifecycle.ts';
import { AppearanceCustomizationRuntime } from '../components/AppearanceCustomizationRuntime.tsx';
import {createSharingAccess} from '../api/mailboxSharing.ts';
import {createConfiguredClient} from '../api/transport.ts';
import {FolderPane} from '../components/FolderPane.tsx';
import {actions as sidebarFooterActions} from '../components/sidebarFooter.css.ts';
import {FolderContextMenu} from '../components/FolderContextMenu.tsx';
import {peekMessageDrag,takeMessageDrag} from '../components/messageDrag.ts';
import {folderDropTarget} from '../components/messageDrag.css.ts';
import { ResizeHandle } from '../components/ResizeHandle.tsx';
import { accountUiPreferences, setAccountUiPreferences } from '../api/accountPreferences.ts';
import { readingPane } from '../components/readingPane.ts';
import { messageOpenTarget, type OpenGesture } from '../components/messageOpen.ts';
import { orderedFolders, folderIcon } from './Settings/folderPreferenceModel.ts';
import type { MessageView } from '../state/slices/messageView.ts';
import { createSettingsNavigation, type SettingsNavigation } from './Settings/navigation.tsx';
import '../styles/workspace.css.ts';
import { Icon } from '../components/Icon.tsx';
import { AttachmentTab, type AttachmentTabItem } from '../viewers/AttachmentTab.tsx';
import { buildDownloadUrl } from '../viewers/attachments.ts';
import type { StripItem } from '../viewers/ThumbnailStrip.tsx';
import { useBranding, brandAssetUrl } from '../branding.ts';
import { batch, createEffect, createMemo, createSignal, on, untrack, For, Show, Suspense, onMount, onCleanup, type JSX } from 'solid-js';
import { Dynamic } from 'solid-js/web';
import { useApp } from '../state/context.ts';
import { createEntryMotion } from '../shell/createEntryMotion.ts';
import * as motion from '../styles/motion.css.ts';
import { t, isolate, loadCatalog } from '../i18n/index.ts';
import * as a11y from '../components/mailA11y.css.ts';
import { useRealtime } from '../realtime/context.ts';
import { MessageList } from '../components/MessageList.tsx';
import { Reader } from '../components/Reader.tsx';
import { createNarrowViewport, focusIsLost } from '../components/narrowViewport.ts';
import { Compose } from '../components/Compose.tsx';
import type { ComposeInitial } from '../components/compose/reply.ts';
import { Outbox } from '../components/Outbox.tsx';
import { InboxTabs } from '../components/InboxTabs.tsx';
import { EventsNavEntry, EventsViewBar } from '../components/EventsView.tsx';
import { EVENTS_VIEW_ID } from '../api/jmap.ts';
import { SubTabStrip } from '../components/SubTabStrip.tsx';
import { ConfirmDialog } from './Admin/shell/ConfirmDialog.tsx';
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
import { AssistPanel } from '../modules/assist/index.ts';
// t88: the search input, its icon buttons and the filter chips.
import { SearchBox } from '../components/search/SearchBox.tsx';

// The nav-rail app modules: the four V3 PIM modules + the V4 key-management
// module (plan §2.5, e8 mount). Keys is reachable at `#/keys` beside them.
const APP_NAV_MODULES = [...APP_MODULES, KEYS_MODULE];

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
  /** The Mail tab is the strip's first tab. Seeded here, before any effect,
   *  because the saved-tab restore below runs in an effect that can fire before
   *  onMount; seeding on mount only when the strip was empty left a reloaded
   *  workspace with its restored tabs and no Mail tab. */
  function seedMailTab(): void {
    if (subTabs.tabs().some(tab => tab.id === 'mail')) return;
    subTabs.open({ kind: 'messages', title: t('common-module-mail'), id: 'mail', pinned: true });
    subTabs.move('mail', 0);
  }
  seedMailTab();
  interface Pane { opener?:string; refresh?:()=>void; detach?:DetachedPaneHandle; beforeClose?:(discard?:boolean)=>Promise<boolean>; dirty?:()=>boolean; id:string; kind:'messages'|'composer'|'settings'|'attachment'|'module'|'outbox'|'attachments'|'sharing'|'folder'; title:string; surface?:ShellSurface; reference?:string; draftId?:string; accountId?:string; view?:MessageView; initial?:ComposeInitial; navigation?:SettingsNavigation; attachment?:AttachmentTabItem }
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
  // A pane whose tab is not open yet (setPanes runs just before subTabs.open)
  // sorts last, where its tab will be: sorting it first and then moving it made
  // <For> move the other panes' nodes, and moving a node drops its focus (t51).
  const orderedPanes=createMemo(()=>{const order=subTabs.tabs().map(tab=>tab.id);const at=(id:string):number=>{const index=order.indexOf(id);return index<0?Number.MAX_SAFE_INTEGER:index;};return [...panes()].sort((a,b)=>at(a.id)-at(b.id));});
  const [paneWindowRevision,setPaneWindowRevision]=createSignal(0);
  const views=createWorkspaceViews({ids:()=>{paneWindowRevision();return orderedPanes().filter(pane=>!pane.detach?.isDetached()).map(pane=>pane.id);},activeId:subTabs.activeId,owner:()=>{const owner=workspaceOwner();return owner?.storageAuthority?JSON.stringify([owner.id,owner.storageAuthority,owner.generation]):null;},activate:subTabs.activate});
  const effectiveLayout=():WorkspaceLayout=>narrow()?'tabs':workspaceLayout();
  const paneVisible=(id:string):boolean=>{paneWindowRevision();if(subTabs.activeId()==='mail')return false;if(effectiveLayout()==='tabs')return subTabs.activeId()===id||companion()===id;return !panes().find(pane=>pane.id===id)?.detach?.isDetached()&&(!isSplitLayout(effectiveLayout())||views.slot(id)!==null);};
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
  // The phone-width switch (t28-e6; the narrow shell below). Declared here: the
  // companion memo reads it on its first run.
  const narrow = createNarrowViewport();
  // t51: a message opened from a folder tab keeps that folder on screen beside it,
  // the way the Mail list stays beside its reader, so the row that opened it keeps
  // focus. Tabs layout only (split layouts give the message the other view slot,
  // masonry shows every pane); not where the reader covers the list.
  const companion=createMemo(():string|null=>{
    paneWindowRevision();
    const active=activePane();
    if(active?.kind!=='messages'||!active.opener||effectiveLayout()!=='tabs'||narrow()||readingPane()==='off'||accountUiPreferences().messageListCollapsed)return null;
    const opener=panes().find(pane=>pane.id===active.opener);
    return opener&&!opener.detach?.isDetached()?opener.id:null;
  });
  const composing=()=>activePane()?.kind==='composer';
  // t74: the reading pane. With it on (Right or Bottom, and not at phone width) a
  // message opened from the Mail list shows in this one reader beside the list
  // and the next one replaces it; it is not a tab. A double-click or Shift+Enter
  // asks for a tab, which takes this reader's view as it stands (no second fetch).
  // The pane is on screen (empty until a message is opened) whenever the setting
  // is on, so the first click does not narrow the list under the pointer and
  // swallow the second click of a double-click.
  const [preview,setPreview]=createSignal<MessageView|null>(null);
  let previewId:string|null=null;let previewLoading:Promise<void>|undefined;
  const previewVisible=()=>subTabs.activeId()==='mail'&&!narrow()&&readingPane()!=='off'&&!accountUiPreferences().readerCollapsed;
  const listVisible=()=>!activePane() ? (!previewVisible() || preview()===null || !accountUiPreferences().messageListCollapsed) : (activePane()?.kind==='messages' && (accountUiPreferences().readerCollapsed || (!narrow() && readingPane()!=='off' && !accountUiPreferences().messageListCollapsed)));
  const readersVisible=()=>!!activePane() && !(activePane()?.kind==='messages' && accountUiPreferences().readerCollapsed);
  const splitVisible=()=>listVisible() && (readersVisible() || previewVisible());
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
  function openFolderTab(mailboxId:string,restoring=false):void {
    const owner=preferenceAccount(),box=app.mailboxes().find(item=>item.id===mailboxId);
    if(!owner||owner.id!==app.accountId()||!box)return;
    const id=`folder:${owner.id}:${mailboxId}`;
    if(!canOpenPane(id))return;
    if(!restoring)noteWorkspaceIntent();
    if(!restoring)setNavOpen(false);
    router.navigate('mail');
    if(!panes().some(pane=>pane.id===id))setPanes(old=>[...old,{id,kind:'folder',title:box.name,reference:mailboxId}]);
    subTabs.open({id,kind:'folder',title:box.name});
  }
  // A renamed folder (t65, here or on another device) keeps its id: its open
  // folder and sharing tabs take the new name.
  createEffect(()=>{const boxes=app.mailboxes();for(const pane of panes()){if((pane.kind!=='folder'&&pane.kind!=='sharing')||!pane.reference)continue;const box=boxes.find(item=>item.id===pane.reference);if(box&&box.name!==pane.title){pane.title=box.name;subTabs.rename(pane.id,box.name);}}});
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
  /** A message opened from the Mail list: the reading pane or a tab (t74, messageOpen.ts). Drafts open in compose either way. */
  function openFromList(messageId:string,gesture:OpenGesture):Promise<void>{
    // A message already open in its own tab goes to that tab: no second reader,
    // no second load, and the first click of a repeat double-click does not
    // flash the reading pane before the tab comes back.
    const tabbed=panes().some(pane=>pane.id===`message:${app.accountId() ?? ''}:${messageId}`);
    return openDraftOrMessage(messageId,undefined,tabbed?'tab':messageOpenTarget(gesture,{readingPane:readingPane(),narrow:narrow()}));
  }
  async function openDraftOrMessage(messageId:string,opener?:string,target:'pane'|'tab'='tab'):Promise<void>{
    const owner=preferenceAccount(),message=app.messages().find(item=>item.id===messageId);
    if(owner?.storageAuthority&&message?.keywords?.['$draft']){
      try{const result=await accountDraftApi(owner).resolve(messageId);if(!preferenceAccountIsCurrent(owner))return;if(result.draft?.state){const draft=localAccountDraft(result.draft);saveDraft(draft);openComposer(undefined,draft.id);return;}}
      catch{if(!preferenceAccountIsCurrent(owner))return;}
    }
    if(target==='pane')openPreview(messageId);else openMessageTab(messageId,false,opener);
  }
  function openPreview(messageId:string):void {
    noteWorkspaceIntent();
    router.navigate('mail');
    subTabs.activate('mail');
    let view=preview();
    // The same message again (the second click of a double-click, a re-click): no refetch.
    if(view&&previewId===messageId&&!view.error()){announceOpen(view,previewLoading);return;}
    if(!view){view=app.createReaderView();setPreview(view);}
    previewId=messageId;
    const loading=view.openMessage(messageId);previewLoading=loading;
    announceOpen(view,loading);
  }
  function closePreview():void {const view=preview();previewId=null;previewLoading=undefined;setPreview(null);view?.dispose();}
  // t51: opening a message leaves focus on the row that opened it; a polite
  // status names the message so the open is not silent for a screen reader.
  // Where the reader covers the list (the focused row is hidden) the reader
  // moves focus to its subject instead, and this says nothing.
  const [openedNotice,setOpenedNotice]=createSignal('');
  let noticeSeq=0;
  function announceOpen(view:MessageView,loading?:Promise<void>):void {
    const seq=++noticeSeq;
    queueMicrotask(()=>{
      if(seq!==noticeSeq||focusIsLost())return;
      const say=():void=>{if(seq!==noticeSeq)return;const subject=view.openEmail()?.subject;setOpenedNotice('');queueMicrotask(()=>{if(seq===noticeSeq)setOpenedNotice(t('mail-opened-announcement',{subject:subject||t('mail-no-subject')}));});};
      if(loading)void loading.then(say,()=>undefined);else say();
    });
  }
  function openMessageTab(messageId:string,restoring=false,opener?:string):void {
    if(!restoring)noteWorkspaceIntent();
    router.navigate('mail');
    const id=`message:${app.accountId() ?? ''}:${messageId}`;if(!canOpenPane(id))return;
    const existing=panes().find(pane=>pane.id===id);
    if(opener&&isSplitLayout(effectiveLayout()))views.openBeside(opener);
    if(existing) {if(!restoring){if(opener)existing.opener=opener;else delete existing.opener;}subTabs.activate(id);if(!restoring&&existing.view)announceOpen(existing.view);return;}
    const title=app.messages().find(message=>message.id===messageId)?.subject || t('mail-workspace-message');
    // t74: the reading pane's message asked for a tab takes the pane's view with
    // it, loaded or loading; the pane is empty after. One batch, so no frame
    // shows the list alone between the two.
    const adopted=!restoring&&previewId===messageId?preview():null;
    const view=adopted??app.createReaderView();
    const loading=adopted?previewLoading:undefined;
    batch(()=>{
      if(adopted){previewId=null;previewLoading=undefined;setPreview(null);}
      setPanes(old=>[...old,{id,kind:'messages',title,view,reference:messageId,...(opener?{opener}:{})}]);
      subTabs.open({id,kind:'messages',title});
    });
    if(!restoring)announceOpen(view,adopted?loading:view.openMessage(messageId));
    else void view.openMessage(messageId);
  }
  /** `discard`: the user already chose to discard this tab's unsaved changes (a bulk close), so it does not ask again. */
  async function closePane(id:string,discard=false):Promise<void> {
    if(id==='mail') return;
    noteWorkspaceIntent();
    const pane=panes().find(item=>item.id===id),owner=preferenceAccount();
    // A composer asks while still detached, so its prompt opens in the window that holds it (t42).
    if(pane?.beforeClose && !await pane.beforeClose(discard)) return;
    pane?.detach?.reattach();
    if(pane?.navigation && !await pane.navigation.canLeave(discard)) return;
    if(!await confirmShellNavigation(id)) return;
    if((owner&&!preferenceAccountIsCurrent(owner))||panes().find(item=>item.id===id)!==pane)return;
    const wasActive=subTabs.activeId()===id;
    pane?.view?.dispose();setPanes(old=>old.filter(item=>item.id!==id));subTabs.close(id);
    if(wasActive) queueMicrotask(()=>document.getElementById(`subtab-tab-${subTabs.activeId()}`)?.focus());
  }
  // t50: Close other tabs / to the left / to the right / all tabs. The tabs with
  // unsaved changes are listed in one dialog: keep them open (the clean tabs still
  // close) or discard and close them all. Detached tabs close their window.
  const [dirtyClose,setDirtyClose]=createSignal<{titles:string[];answer:(discard:boolean)=>void}|null>(null);
  onCleanup(()=>dirtyClose()?.answer(false));
  const paneDirty=(pane:Pane):boolean=>pane.dirty?.()===true||pane.navigation?.isDirty()===true;
  async function closePanes(ids:string[]):Promise<void> {
    const targets=ids.filter(id=>id!=='mail'&&panes().some(pane=>pane.id===id));
    if(targets.length===0||dirtyClose())return;
    noteWorkspaceIntent();
    const owner=preferenceAccount();
    const dirty=new Set(targets.filter(id=>{const pane=panes().find(item=>item.id===id);return !!pane&&paneDirty(pane);}));
    let discard=false;
    if(dirty.size>0){
      const titles=[...dirty].map(id=>subTabs.tabs().find(tab=>tab.id===id)?.title??panes().find(pane=>pane.id===id)?.title??'');
      discard=await new Promise<boolean>(resolve=>setDirtyClose({titles,answer:value=>{setDirtyClose(null);resolve(value);}}));
      if(owner&&!preferenceAccountIsCurrent(owner))return;
    }
    for(const id of targets){if(dirty.has(id)&&!discard)continue;await closePane(id,dirty.has(id));}
  }
  createEffect(()=>{app.setActiveReader(surface()!=='mail' ? null : readersVisible() ? activePane()?.view ?? null : previewVisible() ? preview() : null);});
  let workspaceGeneration:number|undefined;
  onCleanup(observePreferenceAccount(owner=>{if(owner?.generation===workspaceGeneration)return;workspaceGeneration=owner?.generation;restoreIntent=0;setWorkspaceReady(false);for(const pane of untrack(panes)){pane.view?.dispose();subTabs.close(pane.id);}setPanes([]);closePreview();subTabs.activate('mail');setWorkspaceOwner(owner);}));

  onCleanup(()=>{for(const pane of panes()) {pane.view?.dispose();subTabs.close(pane.id);}preview()?.dispose();app.setActiveReader(null);});
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
    if(app.linkedMailboxId?.()&&destination!=='attachments'&&destination!=='outbox'){router.navigate('mail');return;}
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
          else if(tab.kind==='folder'&&tab.reference)openFolderTab(tab.reference,true);
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
  // `narrow` is declared above, before the companion memo that reads it.
  // One visible account indicator (t38 §2.3): the sidebar head owns it; compact
  // folders show it as an icon; a hidden desktop sidebar hands it to the top bar.
  const accountIconInSidebar = (): boolean => !narrow() && folderMode() === 'compact';
  const accountIconInTopBar = (): boolean => !narrow() && folderMode() === 'hidden';
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
  // Switching mailbox (from the drawer's picker or anywhere else) closes the drawer.
  createEffect(on(() => app.linkedMailboxId?.() ?? null, () => closeNav(), { defer: true }));
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
      if (app.selectedMailboxId() === EVENTS_VIEW_ID) return t('mail-nav-events');
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
    refresh: () => { if (surface() !== 'mail') return false; if(activePane()?.kind==='folder')activePane()?.refresh?.();else void app.refreshCurrentMailbox(); return true; },
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

  // Folder placeholders: as many as the last folder list had, so a mailbox
  // switch does not move what sits below the folders (t71).
  const [folderRows,setFolderRows]=createSignal(6);
  createEffect(()=>{const count=app.mailboxes().length;if(count>0)setFolderRows(Math.min(count,12));});

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
          {/* Acting in the wrong mailbox is the risk: while a linked child
              mailbox is active, the phone bar names it (the switcher is in the drawer). */}
          <Show when={app.linkedMailboxId?.() ? app.me()?.username : undefined}>{address=>
            <button type="button" class={`btn btn--ghost shell__account-chip ${a11y.focusable}`} data-account-indicator="chip"
              aria-label={accountIndicatorLabel(address(),true,mailboxNickname(app.linkedMailboxId()))} data-tooltip={accountIndicatorLabel(address(),true,mailboxNickname(app.linkedMailboxId()))} aria-controls="shell-nav" onClick={openNav}>
              <Icon name="users"/><span class="shell__account-chip-text">{isolate(mailboxNickname(app.linkedMailboxId())??address())}</span>
            </button>}
          </Show>
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
          // The mailbox picker's trigger only opens its menu; choosing a
          // mailbox closes the drawer through the switch itself (below).
          if (e.target instanceof Element && e.target.closest('button') !== null && e.target.closest('.sidebar__account') === null) closeNav();
        }}
      >
        <div class="sidebar__head">
          <span class="sidebar__brand" title={branding()?.brandName || t('mail-brand')}><Show when={brandAssetUrl(branding()?.logoUrl)}>{url=><img src={url()} alt={branding()?.brandName||'Mailwoman'} class="sidebar__brand-logo"/>}</Show>{branding()?.brandName || t('mail-brand')}</span>
          {/* The one account indicator and switcher (t38 §2.3): text in the
              expanded sidebar, a one-letter button when the folders are compact. */}
          <Show when={app.me()}><div class="sidebar__account" data-variant={accountIconInSidebar()?'icon':'full'}><LinkedAccountControl variant={accountIconInSidebar()?'icon':'full'}/></div></Show>
        </div>
        <button type="button" class={`btn btn--primary sidebar__compose ${a11y.focusable}`} aria-label={t('mail-compose')} title={t('mail-compose')} onClick={openNewMessage}>
          <Icon name="edit"/><span class="sidebar__box-name">{t('mail-compose')}</span>
        </button>
        <div class="sidebar__scroll">
        <nav class="sidebar__nav" aria-label={t('mail-nav-mailboxes')} aria-busy={!!app.me()&&app.mailboxes().length===0}>
          <Show when={app.me()&&app.mailboxes().length===0}><AsyncPending variant="folders" rows={folderRows()} quiet message={t('mail-folders-loading')} onRetry={()=>void app.refreshFolders()}/></Show>
          <For each={orderedFolders(app.mailboxes(),accountUiPreferences()).map(box=>box.id)}>
            {(id) => {const box=createMemo(()=>app.mailboxes().find(value=>value.id===id)!);return (
              <FolderContextMenu box={box()} onOpenTab={()=>openFolderTab(box().id)} onOpen={()=>{router.navigate('mail');subTabs.activate('mail');void app.selectMailbox(box().id);}} onSettings={()=>{openSettings();const pane=panes().find(item=>item.id===`settings:${app.accountId() ?? ''}`);void pane?.navigation?.navigate({section:'folders'});}} canShare={()=>sharingAccess.canShare(box().id)} onProbe={()=>void sharingAccess.probe(box().id,true)} onShare={()=>void openSharing(box().id)}>
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
          <EventsNavEntry active={surface() === 'mail' && app.selectedMailboxId() === EVENTS_VIEW_ID} onSelect={() => { router.navigate('mail'); subTabs.activate('mail'); void app.selectMailbox(EVENTS_VIEW_ID); }}/>
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
        <Show when={!app.linkedMailboxId?.()}><nav class="sidebar__nav sidebar__nav--apps" aria-label={t('mail-nav-apps')}>
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
        </nav></Show>
        </div>
        <div class="sidebar__footer">
        <Show when={folderMode()!=='compact'||narrow()}><QuotaMeter accountId={app.accountId()} refresh={app.mailboxes()}/></Show>
        <div class={sidebarFooterActions}>
        <button type="button" class={`btn btn--ghost sidebar__settings ${a11y.focusable} ${a11y.iconButton}`} aria-label={t('mail-nav-settings')} data-tooltip={t('mail-nav-settings')} onClick={()=>openSettings()}>
          <Icon name="settings"/>
        </button>
        <button type="button" class={`btn btn--ghost sidebar__logout ${a11y.focusable}`} aria-label={t('mail-logout')} title={t('mail-logout')} onClick={() => void app.logout()}>
          <Icon name="signOut"/><span class="sidebar__box-name">{t('mail-logout')}</span>
        </button>
        </div>
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
        <SubTabStrip onClose={id=>void closePane(id)} onCloseMany={ids=>void closePanes(ids)} canClose={id=>id!=='mail'} canDetach={id=>id!=='mail'} isDetached={id=>{paneWindowRevision();return panes().find(pane=>pane.id===id)?.detach?.isDetached()??false;}} onReattach={id=>{panes().find(pane=>pane.id===id)?.detach?.reattach();subTabs.activate(id);}} onDetach={id=>{const pane=panes().find(value=>value.id===id);subTabs.activate(id);pane?.detach?.detach();}} />
        <CollapsibleBar id="mailTopBar" revealPolicy="overlay" class="mail-topbar">
        <div class="workspace-bar" role="toolbar" aria-label={t('mail-workspace-layout')}>
          {/* With the sidebar hidden, its account indicator moves here: exactly one is ever shown. */}
          <Show when={accountIconInTopBar()}><LinkedAccountControl variant="icon"/></Show>
          <button type="button" class={`btn btn--ghost ${a11y.iconButton}`} aria-label={t(folderMode()==='hidden'?'mail-show-folders':'mail-hide-folders')} title={t(folderMode()==='hidden'?'mail-show-folders':'mail-hide-folders')} onClick={()=>setAccountUiPreferences({folderSidebarMode:folderMode()==='hidden'?'expanded':'hidden'})}><Icon name="folder"/></button>
          <button type="button" class={`btn btn--ghost ${a11y.iconButton}`} aria-pressed={folderMode()==='compact'} aria-label={t('mail-compact-folders')} title={t('mail-compact-folders')} onClick={()=>setAccountUiPreferences({folderSidebarMode:folderMode()==='compact'?'expanded':'compact'})}><Icon name="menu"/></button>
          <button type="button" class={`btn btn--ghost ${a11y.iconButton}`} aria-pressed={!accountUiPreferences().messageListCollapsed} aria-label={t('mail-toggle-list')} title={t('mail-toggle-list')} onClick={()=>setAccountUiPreferences({messageListCollapsed:!accountUiPreferences().messageListCollapsed,readerCollapsed:false})}><Icon name="mail"/></button>
          <button type="button" class={`btn btn--ghost ${a11y.iconButton}`} aria-pressed={!accountUiPreferences().readerCollapsed} aria-label={t('mail-toggle-reader')} title={t('mail-toggle-reader')} onClick={()=>setAccountUiPreferences({readerCollapsed:!accountUiPreferences().readerCollapsed,messageListCollapsed:false})}><Icon name="eye"/></button>

          <WorkspaceLayoutControls value={workspaceLayout()} onChange={setWorkspaceLayout}/>
          <button type="button" class={`btn btn--ghost ${a11y.iconButton}`} title={t('mail-workspace-refresh')} aria-label={t('mail-workspace-refresh')} onClick={()=>{if(activePane()?.kind==='folder')activePane()?.refresh?.();else void app.refreshCurrentMailbox();}}><Icon name="refresh" /></button>
        </div>
        </CollapsibleBar>
        <Show when={app.layout()==='ribbon'}><Ribbon onCompose={openNewMessage} onOpenSettings={openSettings}/></Show>
        <div class={a11y.srOnly} role="status" aria-live="polite" aria-atomic="true" data-testid="message-opened-status">{openedNotice()}</div>
        <div class="workspace-body" ref={bodyElement} data-reading-pane={readingPane()} classList={{'workspace-body--split':splitVisible()}}>
          <div class={`mail-pane ${motion.surface}`} hidden={!listVisible()} data-mw-entering={mailEntering()}>
            <SearchBox /><InboxTabs /><EventsViewBar /><MessageList onOpenMessage={id=>void openFromList(id,'activate')} onOpenMessageTab={id=>void openFromList(id,'tab')} onCompose={openReply}/>
          </div>
          <Show when={splitVisible()}><ResizeHandle label={t('mail-resize-list')} orientation={readingPane()==='bottom'?'horizontal':'vertical'} value={listSize()} min={readingPane()==='bottom'?160:240} max={listMax()} onChange={setListDrag} onCommit={value=>{setAccountUiPreferences(readingPane()==='bottom'?{messageListHeight:value}:{messageListWidth:value});setListDrag(null);}}/></Show>
          <div class="workspace-panes" ref={tilesElement} data-layout={effectiveLayout()} data-pair={isSplitLayout(effectiveLayout())} data-companion={companion()!==null} hidden={!readersVisible()}>
            <For each={orderedPanes()}>{pane=><PaneNavigationContext.Provider value={pane.id}><section class="workspace-pane" role="tabpanel" aria-label={subTabs.tabs().find(tab=>tab.id===pane.id)?.title??pane.title}
              hidden={!paneVisible(pane.id)}
              inert={!paneVisible(pane.id)} data-tab-id={pane.id} data-view-slot={views.slot(pane.id)??undefined} data-drop-target={dropView()!==null&&dropView()===views.slot(pane.id)}
              onDragOver={event=>dragView(event,views.slot(pane.id))} onDrop={event=>dragView(event,views.slot(pane.id),true)} onDragLeave={()=>setDropView(null)}
              data-companion-list={companion()===pane.id}
              onFocusIn={()=>{if(companion()!==pane.id)subTabs.activate(pane.id);}} onPointerDown={()=>{if(companion()!==pane.id)subTabs.activate(pane.id);}}>
              <DetachedWorkspacePane title={()=>subTabs.tabs().find(tab=>tab.id===pane.id)?.title??pane.title} account={()=>mailboxNickname(app.linkedMailboxId())??app.me()?.username} register={handle=>{pane.detach=handle;setPaneWindowRevision(value=>value+1);}}>{()=> <>
              <Show when={pane.kind==='messages' && pane.view}>{view=><><Show when={view().error()}><p role="alert">{t('mail-workspace-load-error')}</p></Show><Reader view={view()} active={subTabs.activeId()===pane.id} onClose={()=>void closePane(pane.id)} onCompose={openReply} onOpenAttachment={openAttachmentTab} onExport={messageId=>void openExport(messageId)} onOpenSettings={section=>{openSettings();const pane=panes().find(item=>item.id===`settings:${app.accountId() ?? ''}`);void pane?.navigation?.navigate({section});}}/></>}</Show>
              <Show when={pane.kind==='attachment' && pane.attachment}>{item=><AttachmentTab item={item()}/>}</Show>
              <Show when={pane.kind==='composer'}><Compose embedded registerBeforeClose={(handler,dirty)=>{pane.beforeClose=handler;pane.dirty=dirty;}} onSubjectChange={subject=>subTabs.rename(pane.id,subject.trim()||t('mail-compose'))} onDraftIdChange={id=>{if(pane.draftId!==id){pane.draftId=id;setDraftRevision(value=>value+1);}}} {...(pane.draftId?{resumeDraftId:pane.draftId}:{})} active={subTabs.activeId()===pane.id} onClose={()=>void closePane(pane.id)} {...(pane.initial ? {initial:pane.initial} : {})}/></Show>
              <Show when={pane.kind==='folder'&&pane.reference}><FolderPane mailboxId={pane.reference!} beforeGlobalAction={()=>pane.detach?.reattach()} registerRefresh={refresh=>{pane.refresh=refresh;}} active={subTabs.activeId()===pane.id} companion={companion()===pane.id} onOpen={id=>void openDraftOrMessage(id,pane.id)}/></Show>
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
          <section class="workspace-pane workspace-reading-pane" aria-label={t('mail-reading-pane')} data-testid="reading-pane" hidden={!previewVisible()} inert={!previewVisible()}>
            <Show when={preview()} fallback={<p class="reader__empty">{t('mail-reader-empty')}</p>}>{view=><Reader view={view()} active={previewVisible()} onClose={closePreview} onCompose={openReply} onOpenAttachment={openAttachmentTab} onExport={messageId=>void openExport(messageId)} onOpenSettings={section=>{openSettings();const pane=panes().find(item=>item.id===`settings:${app.accountId() ?? ''}`);void pane?.navigation?.navigate({section});}}/>}</Show>
          </section>
        </div>
        <ConfirmDialog open={dirtyClose()!==null} testId="workspace-tabs-close-dirty"
          title={t('workspace-tabs-close-dirty-title',{count:dirtyClose()?.titles.length??0})} body={t('workspace-tabs-close-dirty-body')}
          confirmLabel={t('workspace-tabs-close-dirty-discard')} cancelLabel={t('workspace-tabs-close-dirty-keep')}
          onCancel={()=>dirtyClose()?.answer(false)} onConfirm={()=>dirtyClose()?.answer(true)}>
          <ul class="workspace-close-dirty"><For each={dirtyClose()?.titles??[]}>{title=><li dir="auto">{title}</li>}</For></ul>
        </ConfirmDialog>
        <AssistPanel config={app.assist.config()} service={app.assist.service} context={assistContext()} onReviewAction={openNewMessage}/>
      </div>

    </div>
  );
}

// The summary header of the folder Properties dialog: the folder's icon, its name
// (renamed in place), the path with a copy button, the type badge, the subscribed /
// starred / pinned switches and four stat tiles. The tiles hold the only skeletons:
// the rest of the dialog never waits for the numbers.
import {For,Show,createMemo,createSignal,type JSX} from 'solid-js';
import {useApp} from '../../state/context.ts';
import {accountUiPreferences,setAccountUiPreferences,type AccountUiPreferences} from '../../api/accountPreferences.ts';
import {createConfiguredClient} from '../../api/transport.ts';
import type {Mailbox} from '../../api/jmap-types.ts';
import {setFoldersSubscribed} from '../../api/folderBulkDelete.ts';
import {canRename} from '../../api/renameFolder.ts';
import {isolate,t} from '../../i18n';
import {Icon} from '../Icon.tsx';
import {FolderRenameField,useRenameContext} from '../RenameFolder.tsx';
import {folderIcon,type PreferenceFolder} from '../../screens/Settings/folderPreferenceModel.ts';
import {toast} from '../toast/toast.ts';
import {formatBytes} from './folderStats.ts';
import type {StatsState} from './useFolderStats.ts';
import {appliedOverride} from './folderOverrides.ts';
import {copyText,dateText,folderTypeKey,pathOf} from './folderDialogUtil.ts';
import * as css from './folderDialog.css.ts';

export function FolderHeader(props:{box:Mailbox;stats:StatsState;titleId:string;renaming:boolean;onRenaming:(on:boolean)=>void}):JSX.Element {
 const app=useApp(),client=createConfiguredClient();
 const context=useRenameContext();
 const prefs=():AccountUiPreferences=>accountUiPreferences();
 const b=()=>props.box;
 const editable=()=>canRename(b(),context());
 const crumbs=createMemo(()=>pathOf(app.mailboxes(),b()));
 const pathText=()=>b().mailwomanPath??crumbs().map(item=>item.name).join('/');
 const starred=()=>prefs().folderStarred?.includes(b().id)===true,pinned=()=>prefs().folderPinned?.includes(b().id)===true;
 const flip=(key:'folderStarred'|'folderPinned')=>{const now=prefs()[key]??[];setAccountUiPreferences({[key]:now.includes(b().id)?now.filter(id=>id!==b().id):[...now,b().id]});};
 const [busy,setBusy]=createSignal(false);
 async function subscribe():Promise<void> {
  const accountId=app.accountId();if(!accountId||busy()||b().isSubscribed===undefined)return;
  setBusy(true);
  try{
   const result=await setFoldersSubscribed(client,accountId,[b().id],!b().isSubscribed,{current:()=>app.accountId()===accountId});
   if(result.failed.length>0)toast.error(t('props-subscribe-failed'));else await app.refreshFolders().catch(()=>{});
  }catch{toast.error(t('props-subscribe-failed'));}
  finally{setBusy(false);}
 }
 const colour=()=>appliedOverride('badgeColour',b().id,prefs().folderProps) as string|undefined;
 const shared=()=>props.stats.stats()?.remote?.namespace==='shared'||props.stats.stats()?.remote?.namespace==='other';
 return <header class={css.header} data-testid="props-header">
  <div class={css.identity}>
   <span class={css.folderGlyph} aria-hidden="true" data-testid="props-glyph" style={colour()?{color:colour()!}:undefined}><Icon name={folderIcon(b() as unknown as PreferenceFolder,prefs())}/></span>
   <div class={css.identityText}>
    <div class={css.nameRow}>
     <Show when={!props.renaming} fallback={<FolderRenameField box={b()} context={context()??{manages:true,maxLength:255,delimiter:null}} onDone={()=>props.onRenaming(false)}/>}>
      <h2 id={props.titleId} class={css.folderName}>{isolate(b().name)}</h2>
      <Show when={editable()}><button type="button" class={css.iconButton} data-testid="props-rename" aria-label={t('props-rename-folder',{name:isolate(b().name)})} title={t('props-act-rename')} onClick={()=>props.onRenaming(true)}><Icon name="edit"/></button></Show>
     </Show>
    </div>
    <nav aria-label={t('props-path')}>
     <ol class={css.breadcrumb} data-testid="props-breadcrumb"><For each={crumbs()}>{item=><li>{isolate(item.name)}</li>}</For>
      <li><button type="button" class={css.iconButton} data-testid="props-copy-path" aria-label={t('props-copy-what',{what:t('props-path')})} title={t('props-copy-what',{what:t('props-path')})} onClick={()=>void copyText(pathText())}><Icon name="copyPath"/></button></li></ol>
    </nav>
    <div class={css.chipRow}>
     <span class={`${css.chip} ${css.chipAccent}`} data-testid="props-type-badge">{t(`props-badge-${folderTypeKey(b())}`)}</span>
     <Show when={shared()}><span class={css.chip}>{t('props-badge-shared')}</span></Show>
     <Show when={b().isSubscribed!==undefined}><button type="button" class={css.toggle} aria-pressed={b().isSubscribed===true} disabled={busy()} data-testid="props-subscribed" onClick={()=>void subscribe()}><Icon name={b().isSubscribed?'check':'plus'}/>{t('props-subscribed')}</button></Show>
     <button type="button" class={css.toggle} aria-pressed={starred()} data-testid="props-starred" onClick={()=>flip('folderStarred')}><Icon name={starred()?'star':'starOutline'}/>{t('props-starred')}</button>
     <button type="button" class={css.toggle} aria-pressed={pinned()} data-testid="props-pinned" onClick={()=>flip('folderPinned')}><Icon name="pin"/>{t('props-pinned')}</button>
    </div>
   </div>
  </div>
  <Tiles box={b()} stats={props.stats}/>
 </header>;
}

function Tile(props:{label:string;testId:string;children:JSX.Element}):JSX.Element {
 return <div class={css.tile} data-testid={props.testId}><p class={css.tileLabel}>{props.label}</p><div class={css.tileValue}>{props.children}</div></div>;
}

function Tiles(props:{box:Mailbox;stats:StatsState}):JSX.Element {
 const s=()=>props.stats,st=()=>s().stats();
 const Pending=()=><Show when={s().failed()} fallback={<Show when={s().idle()} fallback={<span class={css.skeleton} role="status" aria-label={t('props-computing')} data-testid="props-skeleton"/>}>
  <button type="button" class={css.linkButton} data-testid="props-calc-size" onClick={()=>s().compute()}>{t('props-calc-size')}</button></Show>}>
  <button type="button" class={css.linkButton} data-testid="props-tile-retry" onClick={()=>s().retry()}>{t('props-tile-retry')}</button></Show>;
 return <div class={css.tiles} role="group" aria-label={t('props-summary')} aria-busy={s().loading()}>
  <Tile label={t('props-total')} testId="tile-total">{st()?.totalEmails??props.box.totalEmails}</Tile>
  <Tile label={t('props-unread')} testId="tile-unread">{st()?.unreadEmails??props.box.unreadEmails}</Tile>
  <Tile label={t('props-size-server')} testId="tile-size"><Show when={st()} fallback={<Pending/>}>{stats=>stats().approximate?t('props-approx-size',{size:formatBytes(stats().sizeBytes)}):formatBytes(stats().sizeBytes)}</Show></Tile>
  <Tile label={t('props-newest')} testId="tile-newest"><Show when={st()} fallback={<Pending/>}>{stats=>dateText(stats().newest)}</Show></Tile>
 </div>;
}

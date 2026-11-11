// The General, Appearance and Remote location tabs of the folder dialog.
import {For,Show,createMemo,type JSX} from 'solid-js';
import {useApp} from '../../state/context.ts';
import {accountUiPreferences,setAccountUiPreferences,type FolderIcon} from '../../api/accountPreferences.ts';
import type {Mailbox} from '../../api/jmap-types.ts';
import {isolate,loadCatalog,t} from '../../i18n';
import {Icon} from '../Icon.tsx';
import {FolderCountBadge} from '../FolderCountBadge.tsx';
import {FolderIconPicker} from '../../screens/Settings/FolderIconPicker.tsx';
import {folderIcon,type PreferenceFolder} from '../../screens/Settings/folderPreferenceModel.ts';
import {ColorField} from '../color/ColorPicker.tsx';
import {folderColour,folderColourOverride,withOverride} from './folderOverrides.ts';
import {allowedOverrides,remoteForDisplay,remoteVisibility} from './policy.ts';
import {copyText,folderTypeKey,pathOf} from './folderDialogUtil.ts';
import type {StatsState} from './useFolderStats.ts';
import * as css from './folderDialog.css.ts';

function Pair(props:{label:string;children:JSX.Element}):JSX.Element {return <><dt>{props.label}</dt><dd>{props.children}</dd></>;}
function Copy(props:{text:string;label:string}):JSX.Element {
 return <button type="button" class={css.iconButton} aria-label={t('props-copy-what',{what:props.label})} title={t('props-copy-what',{what:props.label})} onClick={()=>void copyText(props.text)}><Icon name="copy"/></button>;
}
const yesNo=(value:boolean|undefined)=>value===undefined?t('props-unknown'):t(value?'props-yes':'props-no');

export function General(props:{box:Mailbox}):JSX.Element {
 const app=useApp();
 const chain=createMemo(()=>pathOf(app.mailboxes(),props.box));
 const parent=()=>chain().length>1?chain()[chain().length-2]:undefined;
 return <section class={css.card} aria-labelledby="props-general-title" data-testid="props-general">
  <h3 id="props-general-title" class={css.cardTitle}>{t('props-card-about')}</h3>
  <dl class={css.grid}>
   <Pair label={t('props-type')}>{t(`props-role-${folderTypeKey(props.box)}`)}</Pair>
   <Pair label={t('props-subscribed')}>{yesNo(props.box.isSubscribed)}</Pair>
   <Pair label={t('props-parent')}>{parent()?isolate(parent()!.name):t('props-top-level')}</Pair>
   <Pair label={t('props-depth')}>{chain().length-1}</Pair>
   <Pair label={t('props-folder-id')}><span class={css.mono}>{props.box.id}</span><Copy text={props.box.id} label={t('props-folder-id')}/></Pair>
   <Pair label={t('props-account')}><span class={css.mono}>{app.accountId()??''}</span><Copy text={app.accountId()??''} label={t('props-account')}/></Pair>
  </dl>
 </section>;
}

void loadCatalog('settings');

const PALETTE=['#d9534f','#e8833a','#c99a00','#3fa34d','#2a9d8f','#3b82f6','#7c5cd6','#d6568f','#6b7280'];

export function Appearance(props:{box:Mailbox}):JSX.Element {
 const prefs=()=>accountUiPreferences();
 const id=()=>props.box.id;
 const icon=()=>prefs().folderIcons?.[id()]??'';
 const colour=()=>prefs().folderProps?.[id()]?.badgeColour??'';
 const canColour=()=>allowedOverrides().includes('badgeColour');
 const automatic=()=>folderIcon({...props.box,id:'__none__'} as unknown as PreferenceFolder,{});
 function setIcon(next:FolderIcon|''):void {
  const rest={...(prefs().folderIcons??{})};delete rest[id()];
  setAccountUiPreferences({folderIcons:next?{...rest,[id()]:next}:rest});
 }
 function setColour(next:string):void {setAccountUiPreferences({folderProps:withOverride(prefs().folderProps,id(),{badgeColour:next||undefined})});}
 const folderTint=()=>folderColour(id(),prefs().folderProps);
 const canTint=()=>allowedOverrides().includes('folderColour')&&!folderColourOverride(undefined).locked;
 function setTint(value:string|null):void {
  if(canTint())setAccountUiPreferences({folderProps:withOverride(prefs().folderProps,id(),{folderColour:value??undefined})});
 }
 const touched=()=>icon()!==''||(canColour()&&colour()!=='')||(canTint()&&Boolean(prefs().folderProps?.[id()]?.folderColour));
 return <div data-testid="props-appearance" style={{display:'contents'}}>
  <section class={css.card} aria-labelledby="props-prev-title">
   <h3 id="props-prev-title" class={css.cardTitle}>{t('props-card-preview')}</h3>
   <p class={css.cardHelp}>{t('props-preview-help')}</p>
   <div class={css.previewRow} data-testid="props-preview">
    <span style={{color:folderTint()}} aria-hidden="true"><Icon name={folderIcon(props.box as unknown as PreferenceFolder,prefs())}/></span>
    <span class={css.previewName} style={{color:folderTint()}}>{isolate(props.box.name)}</span>
    <FolderCountBadge folder={props.box}/>
   </div>
   <div class={css.row}><button type="button" class="btn btn--ghost" data-testid="props-appearance-reset" disabled={!touched()} onClick={()=>{setIcon('');if(canColour())setColour('');setTint(null);}}>{t('props-reset-default')}</button></div>
  </section>
  <section class={css.card} aria-labelledby="props-icon-title">
   <h3 id="props-icon-title" class={css.cardTitle}>{t('props-card-icon')}</h3>
   <FolderIconPicker label={t('props-card-icon')} value={icon() as FolderIcon|''} automaticIcon={automatic()} onChange={setIcon}/>
  </section>
  <Show when={allowedOverrides().includes('folderColour')}><section class={css.card} aria-labelledby="props-folder-colour-title">
   <h3 id="props-folder-colour-title" class={css.cardTitle}>{t('settings-folder-colour')}</h3>
   <ColorField label={t('settings-folder-colour')} value={prefs().folderProps?.[id()]?.folderColour??null} fallback={folderTint()??'#3366cc'} emptyLabel={t('color-picker-default')} disabled={!canTint()} onChange={setTint}/>
  </section></Show>
  <Show when={canColour()}><section class={css.card} aria-labelledby="props-colour-title">
   <h3 id="props-colour-title" class={css.cardTitle}>{t('props-card-colour')}</h3>
   <p class={css.cardHelp}>{t('props-colour-help')}</p>
   <div class={css.swatches} role="radiogroup" aria-label={t('props-card-colour')} data-testid="props-swatches">
    <button type="button" role="radio" aria-checked={colour()===''} class={css.swatch} style={{background:'transparent'}} title={t('props-colour-none')} aria-label={t('props-colour-none')} onClick={()=>setColour('')}/>
    <For each={PALETTE}>{hex=><button type="button" role="radio" aria-checked={colour().toLowerCase()===hex} class={css.swatch} style={{background:hex}} aria-label={hex} title={hex} onClick={()=>setColour(hex)}/>}</For>
    <input type="color" aria-label={t('props-colour-custom')} value={colour()||'#888888'} onChange={event=>setColour(event.currentTarget.value)}/>
   </div>
  </section></Show>
 </div>;
}

export function Remote(props:{box:Mailbox;stats:StatsState}):JSX.Element {
 const s=()=>props.stats;
 const shown=()=>{const stats=s().stats();return stats?remoteForDisplay(stats.remote,remoteVisibility()):null;};
 const Line=(p:{label:string;value:string|number;copy?:boolean})=><Pair label={p.label}><span class={css.mono}>{p.value}</span><Show when={p.copy!==false}><Copy text={String(p.value)} label={p.label}/></Show></Pair>;
 return <div data-testid="props-remote" style={{display:'contents'}}>
  <Show when={remoteVisibility()!=='hidden'} fallback={<section class={css.card}><p class={css.note}>{t('props-remote-hidden')}</p></section>}>
   <Show when={shown()} fallback={<section class={css.card}>
    <Show when={s().failed()} fallback={<Show when={s().idle()} fallback={<p class={css.note} role="status">{s().loading()?t('props-loading'):t('props-remote-unavailable')}</p>}>
     <p class={css.note}>{t('props-size-off')}</p><div class={css.row}><button type="button" class="btn btn--ghost" onClick={()=>s().compute()}>{t('props-load-remote')}</button></div></Show>}>
     <p class={css.problem} role="alert">{t('props-unavailable')}<button type="button" class={css.linkButton} onClick={()=>s().retry()}>{t('props-tile-retry')}</button></p></Show>
   </section>}>{remote=>
    <>
     <section class={css.card} aria-labelledby="props-remote-title">
      <h3 id="props-remote-title" class={css.cardTitle}>{t('props-card-server')}</h3>
      <dl class={css.grid}>
       <Line label={t('props-remote-path')} value={isolate(remote().path)}/>
       <Show when={remote().host}><Line label={t('props-remote-host')} value={remote().host!}/></Show>
       <Show when={remote().namespace}><Pair label={t('props-remote-namespace')}>{t(`props-ns-${remote().namespace}`)}<Show when={remote().namespace!=='personal'}><span class={css.note}>{t('props-ns-shared-note')}</span></Show></Pair></Show>
       <Show when={remote().delimiter}><Line label={t('props-remote-delimiter')} value={remote().delimiter!} copy={false}/></Show>
      </dl>
     </section>
     <details class={css.advanced} data-testid="props-remote-advanced">
      <summary>{t('props-advanced')}</summary>
      <dl class={css.grid} style={{'margin-block-start':'0.5rem'}}>
       <Show when={remote().uidValidity!==undefined}><Line label="UIDVALIDITY" value={remote().uidValidity!}/></Show>
       <Show when={remote().uidNext!==undefined}><Line label="UIDNEXT" value={remote().uidNext!}/></Show>
       <Show when={remote().highestModSeq!==undefined}><Line label="HIGHESTMODSEQ" value={remote().highestModSeq!}/></Show>
      </dl>
     </details>
    </>}</Show>
  </Show>
 </div>;
}

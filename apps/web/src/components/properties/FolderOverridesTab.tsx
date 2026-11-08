// The Counters and badges (per-folder overrides) and Notes tabs of the folder dialog.
// Only the counter display, the compact badge and the badge colour change what the
// app draws today; the other overrides are stored with the folder and shown under
// "More options", labelled as saved for later, so no control promises more than it does.
import {For,Show,createEffect,createMemo,createSignal,createUniqueId,onCleanup,onMount,type JSX} from 'solid-js';
import {accountUiPreferences,setAccountUiPreferences} from '../../api/accountPreferences.ts';
import {folderCountScope,showsFolderCount} from '../../api/countBadges.ts';
import type {Mailbox} from '../../api/jmap-types.ts';
import {t} from '../../i18n';
import {Icon} from '../Icon.tsx';
import {OVERRIDE_DEFS,effectiveOverride,withOverride,type FolderProps,type OverrideDef} from './folderOverrides.ts';
import {allowedOverrides,notesEnabled,notesMaxLength,type OverrideKey} from './policy.ts';
import * as css from './folderDialog.css.ts';

const OPTION_KEYS:Record<string,string>={none:'props-opt-none',unread:'props-opt-unread',total:'props-opt-total',both:'props-opt-both',on:'props-opt-on',off:'props-opt-off',vips:'props-opt-vips',date:'props-opt-date',from:'props-opt-from',subject:'props-opt-subject',size:'props-opt-size'};
type Patch={[K in keyof FolderProps]?:FolderProps[K]|undefined};

/** The overrides the app applies today. */
export const APPLIED_OVERRIDES:readonly OverrideKey[]=['counters','compactBadge','badgeColour'];
const MORE_GROUPS:readonly {id:string;keys:readonly OverrideKey[]}[]=[
 {id:'notifications',keys:['notifications']},
 {id:'reading',keys:['sort','threading','previewLines','markReadDelay']},
 {id:'replying',keys:['replyIdentity']},
 {id:'sync',keys:['unifiedInbox']},
];

export function Overrides(props:{box:Mailbox;onRetention:(()=>void)|undefined}):JSX.Element {
 const prefs=()=>accountUiPreferences().folderProps??{};
 const own=()=>prefs()[props.box.id];
 const allowed=createMemo(()=>allowedOverrides());
 const defsFor=(keys:readonly OverrideKey[])=>OVERRIDE_DEFS.filter(def=>keys.includes(def.key)&&allowed().includes(def.key));
 const applied=()=>defsFor(APPLIED_OVERRIDES),more=createMemo(()=>MORE_GROUPS.map(group=>({...group,defs:defsFor(group.keys)})).filter(group=>group.defs.length>0));
 function set(patch:Patch):void {setAccountUiPreferences({folderProps:withOverride(prefs(),props.box.id,patch)});}
 const eff=(key:OverrideKey)=>effectiveOverride(OVERRIDE_DEFS.find(def=>def.key===key)!,own(),undefined,undefined);
 return <div data-testid="props-overrides" style={{display:'contents'}}>
  <Show when={applied().length>0}>
   <section class={css.card} aria-labelledby="props-ovr-g-counters">
    <h3 id="props-ovr-g-counters" class={css.cardTitle}>{t('props-group-counters')}</h3>
    <p class={css.cardHelp}>{t('props-group-counters-help')}</p>
    <BadgePreview box={props.box} mode={String(eff('counters').value??'unread')} compact={eff('compactBadge').value!==false} colour={allowed().includes('badgeColour')?own()?.badgeColour:undefined}/>
    <div class={css.group}><For each={applied()}>{def=><OverrideRow def={def} props={own()} onSet={set}/>}</For></div>
   </section>
  </Show>
  <Show when={more().length>0}>
   <details class={css.advanced} data-testid="props-ovr-more">
    <summary>{t('props-more-options')}</summary>
    <p class={css.cardHelp} style={{'margin-block':'0.5rem'}}>{t('props-more-options-help')}</p>
    <div class={css.group}><For each={more()}>{group=><section aria-labelledby={`props-ovr-g-${group.id}`} class={css.card}>
     <h3 id={`props-ovr-g-${group.id}`} class={css.cardTitle}>{t(`props-group-${group.id}`)}</h3>
     <p class={css.cardHelp}>{t(`props-group-${group.id}-help`)}</p>
     <For each={group.defs}>{def=><OverrideRow def={def} props={own()} onSet={set} later/>}</For>
    </section>}</For></div>
   </details>
  </Show>
  <Show when={props.onRetention}><button type="button" class="btn btn--ghost" style={{'align-self':'flex-start'}} onClick={()=>props.onRetention!()}><Icon name="archive"/>{t('props-ovr-retention')}</button></Show>
 </div>;
}

/** The badge as the sidebar will draw it with the choices below. */
function BadgePreview(props:{box:Mailbox;mode:string;compact:boolean;colour:string|undefined}):JSX.Element {
 const mode=():import('../../api/countBadges.ts').FolderCountMode=>props.mode==='none'?'none':props.mode==='total'?'total':props.mode==='both'?'both':'unread';
 const shown=()=>showsFolderCount(props.box,mode(),folderCountScope());
 return <div class={css.previewRow} data-testid="ovr-preview" aria-label={t('props-preview-label')} role="group">
  <span class={css.note}>{t('props-preview-label')}</span>
  <Show when={shown()} fallback={<span class={css.chip} data-testid="ovr-preview-none">{t('props-preview-none')}</span>}>
   <span class="sidebar__badge" data-testid="ovr-preview-badge" data-secondary={props.compact&&props.box.role!=='inbox'?'true':undefined} style={props.colour?{'--badge-bg':props.colour,background:props.colour}:undefined}>
    <Show when={mode()==='unread'||mode()==='both'}>{props.box.unreadEmails}</Show><Show when={mode()==='both'}><span class="sidebar__total"> / {props.box.totalEmails}</span></Show><Show when={mode()==='total'}>{props.box.totalEmails}</Show>
   </span>
  </Show>
 </div>;
}

function OverrideRow(props:{def:OverrideDef;props:FolderProps|undefined;onSet:(patch:Patch)=>void;later?:boolean}):JSX.Element {
 const id=`ovr-${createUniqueId()}`;
 const eff=()=>effectiveOverride(props.def,props.props,undefined,undefined);
 const prop=props.def.prop,type=props.def.type;
 const current=()=>props.props?.[prop];
 const reset=()=>props.onSet({[prop]:undefined} as Patch);
 const choose=(value:unknown)=>props.onSet({[prop]:value} as Patch);
 const label=(v:unknown)=>v===undefined||v===''?t('props-none'):typeof v==='boolean'?t(v?'props-opt-on':'props-opt-off'):typeof v==='string'&&OPTION_KEYS[v]?t(OPTION_KEYS[v]!):String(v);
 const chip=()=>{
  const e=eff(),value=label(e.value);
  if(e.locked)return t('props-chip-locked',{value});
  if(e.overridden)return t('props-chip-overridden',{value});
  if(e.value===undefined)return t('props-chip-default-plain');
  return t(e.source==='admin-default'?'props-chip-admin':'props-chip-default',{value});
 };
 const options=():readonly (string|boolean|number)[]=>type.kind==='bool'?[true,false]:type.kind==='enum'?type.options:type.kind==='lines'?[0,1,2,3]:[];
 const optionText=(o:string|boolean|number)=>typeof o==='boolean'?t(o?'props-opt-on':'props-opt-off'):typeof o==='number'?String(o):t(OPTION_KEYS[o]??o);
 return <div class={css.ovr} data-testid={`ovr-${props.def.key}`} data-source={eff().source}>
  <div class={css.ovrHead}>
   <span class={css.ovrLabel} id={`${id}-label`}>{t(`props-ovr-${props.def.key}`)}</span>
   <span class={css.row}>
    <span class={`${css.chip}${eff().overridden?` ${css.chipAccent}`:eff().locked?` ${css.chipLocked}`:''}`} data-testid={`ovr-${props.def.key}-source`}>{chip()}</span>
    <Show when={props.later}><span class={css.chip} data-testid={`ovr-${props.def.key}-later`}>{t('props-saved-later')}</span></Show>
    <Show when={eff().overridden}><button type="button" class={css.linkButton} data-testid={`ovr-${props.def.key}-reset`} onClick={reset}>{t('props-reset')}</button></Show>
   </span>
  </div>
  <p class={css.cardHelp}>{t(`props-ovr-${props.def.key}-help`)}</p>
  <Show when={!eff().locked} fallback={<span class={css.lockedValue} title={t('props-src-admin-locked-tip')} data-testid={`ovr-${props.def.key}-locked`}><Icon name="lock"/>{label(eff().value)}<span class={css.cardHelp}>{t('props-src-admin-locked-tip')}</span></span>}>
   <Show when={type.kind==='enum'||type.kind==='bool'||type.kind==='lines'}>
    <div class={css.segmented} role="radiogroup" aria-labelledby={`${id}-label`}>
     <For each={options()}>{option=><button type="button" role="radio" class={css.segment} aria-checked={current()===option} onClick={()=>choose(option)}>{optionText(option as string)}</button>}</For>
    </div>
   </Show>
   <Show when={type.kind==='colour'}><span class={css.row}><input id={id} type="color" aria-labelledby={`${id}-label`} value={props.props?.badgeColour??'#888888'} onChange={event=>choose(event.currentTarget.value)}/></span></Show>
   <Show when={type.kind==='delay'}><input id={id} class={css.textField} type="number" min="0" max="60000" step="100" aria-labelledby={`${id}-label`} placeholder={t('props-ovr-inherit')} value={props.props?.markReadDelayMs??''} onChange={event=>{const raw=event.currentTarget.value;if(raw==='')reset();else choose(Number(raw));}}/></Show>
   <Show when={type.kind==='identity'}><input id={id} class={css.textField} type="text" maxlength="256" aria-labelledby={`${id}-label`} placeholder={t('props-ovr-inherit')} value={props.props?.replyIdentity??''} onChange={event=>{const raw=event.currentTarget.value.trim();if(raw==='')reset();else choose(raw);}}/></Show>
  </Show>
 </div>;
}

export function Notes(props:{box:Mailbox}):JSX.Element {
 const saved=()=>accountUiPreferences().folderProps?.[props.box.id]?.notes??'';
 const [draft,setDraft]=createSignal(saved());
 const [flash,setFlash]=createSignal(false);
 const max=()=>notesMaxLength();
 const dirty=()=>draft()!==saved();
 const id=`notes-${createUniqueId()}`;
 let area:HTMLTextAreaElement|undefined;
 function grow():void {if(!area)return;area.style.blockSize='auto';area.style.blockSize=`${Math.min(area.scrollHeight+2,320)}px`;}
 onMount(grow);
 createEffect(()=>{draft();queueMicrotask(grow);});
 function save():boolean {
  if(!dirty())return false;
  const all=accountUiPreferences().folderProps??{};
  setAccountUiPreferences({folderProps:withOverride(all,props.box.id,{notes:draft().trim()===''?undefined:draft().slice(0,max())})});
  if(draft().trim()==='')setDraft('');
  setFlash(true);
  return true;
 }
 // Leaving the tab with an unsaved note keeps it.
 onCleanup(()=>{save();});
 return <Show when={notesEnabled()}><section class={css.card} aria-labelledby={`${id}-title`} data-testid="props-notes">
  <h3 id={`${id}-title`} class={css.cardTitle}>{t('props-card-notes')}</h3>
  <p class={css.cardHelp}>{t('props-notes-help')}</p>
  <label for={id} class={css.ovrLabel}>{t('props-notes-label')}</label>
  <textarea id={id} ref={area} class={css.textarea} maxlength={max()} value={draft()} aria-describedby={`${id}-count`} onInput={event=>{setDraft(event.currentTarget.value);setFlash(false);}} onBlur={()=>{save();}}/>
  <div class={css.row} style={{'justify-content':'space-between'}}>
   <span id={`${id}-count`} class={css.note}>{t('props-notes-count',{count:draft().length,max:max()})}</span>
   <span class={css.row}>
    <span class={css.saved} role="status" data-testid="props-notes-saved">{flash()&&!dirty()?t('props-notes-saved-flag'):''}</span>
    <button type="button" class="btn btn--primary" disabled={!dirty()} onClick={()=>{save();}}>{t('props-notes-save')}</button>
   </span>
  </div>
 </section></Show>;
}

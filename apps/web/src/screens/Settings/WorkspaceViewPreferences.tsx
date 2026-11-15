import {For,onMount,onCleanup,type JSX} from 'solid-js';
import {WORKSPACE_VIEWS,type WorkspaceViewId} from '../../state/workspaceViews.ts';
import {useSetting} from './useSetting.ts';
import {SettingRow} from './layout/SettingRow.tsx';
import {SettingsCard} from './layout/SettingsCard.tsx';
import {Icon} from '../../components/Icon.tsx';
import {loadCatalog,t} from '../../i18n';
import * as css from './styles.css.ts';
import * as layout from './workspaceViewPreferences.css.ts';
void loadCatalog('common');void loadCatalog('mail');void loadCatalog('response-templates');
function ViewFields(props:{id:WorkspaceViewId}):JSX.Element{
 const mode=useSetting<string>(`views.${props.id}.visibility`,'visible'),name=useSetting<string>(`views.${props.id}.name`,''),count=useSetting<boolean>(`views.${props.id}.showCount`,true);
 const base=`workspace-view-${props.id}`;
 return <div class={layout.fields}>
  <SettingRow controlId={`${base}-visibility`} label={t('workspace-view-visibility')} help={t(`workspace-view-notable-${props.id}`)} handle={mode}>{description=><select id={`${base}-visibility`} class={css.select} value={mode.value()} disabled={!mode.entry()||mode.locked()||mode.saving()} aria-describedby={description()} onChange={event=>void mode.set(event.currentTarget.value)}><For each={['visible','hidden','notable']}>{value=><option value={value}>{t(`workspace-view-mode-${value}`)}</option>}</For></select>}</SettingRow>
  <SettingRow controlId={`${base}-name`} label={t('workspace-view-name')} handle={name}>{description=><input id={`${base}-name`} class={css.input} value={name.value()} maxlength={128} disabled={!name.entry()||name.locked()||name.saving()} aria-describedby={description()} onChange={event=>void name.set(event.currentTarget.value)}/>}</SettingRow>
  <SettingRow controlId={`${base}-count`} label={t('workspace-view-count')} handle={count}>{description=><input id={`${base}-count`} type="checkbox" checked={count.value()} disabled={!count.entry()||count.locked()||count.saving()} aria-describedby={description()} onChange={event=>void count.set(event.currentTarget.checked)}/>}</SettingRow>
 </div>;
}
export function WorkspaceViewPreferences():JSX.Element{return <SettingsCard id="workspace-views-card" title={t('workspace-view-properties')} description={t('workspace-view-help')}><For each={WORKSPACE_VIEWS}>{view=><details class={layout.group}><summary class={layout.summary}><Icon name={view.icon}/>{t(view.label)}</summary><ViewFields id={view.id}/></details>}</For></SettingsCard>;}
export function WorkspaceViewProperties(props:{id:WorkspaceViewId;onClose:()=>void}):JSX.Element{
 let dialog!:HTMLDialogElement;const view=()=>WORKSPACE_VIEWS.find(view=>view.id===props.id)!;
 onMount(()=>{if(dialog.showModal)dialog.showModal();else dialog.setAttribute('open','');});
 onCleanup(()=>{if(dialog.open&&dialog.close)dialog.close();});
 return <dialog ref={dialog} class={layout.dialog} aria-label={`${t('workspace-view-properties')}: ${t(view().label)}`} onCancel={props.onClose}><header class={layout.header}><h2 class={css.heading}><Icon name={view().icon}/>{t(view().label)}</h2><button type="button" class={css.ghost} aria-label={t('common-close')} onClick={props.onClose}><Icon name="close"/></button></header><p class={css.meta}>{t('workspace-view-help')}</p><ViewFields id={props.id}/></dialog>;
}

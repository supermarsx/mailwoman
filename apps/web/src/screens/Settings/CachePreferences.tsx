import {useApp} from '../../state/context.ts';
import {For,Show,type JSX} from 'solid-js';
import {MAIL_CACHE_FIELDS,DEFAULT_MAIL_CACHE_POLICY} from '../../state/mailCachePolicy.ts';
import {useSetting} from './useSetting.ts';
import {LockNote} from './LockNote.tsx';
import {t} from '../../i18n';
import * as css from './styles.css.ts';
export function CachePreferences():JSX.Element{
 const app=useApp();
 const exclusions=useSetting<string[]>('mail.cache.excludedFolders',[...DEFAULT_MAIL_CACHE_POLICY.excludedFolders],(value):value is string[]=>Array.isArray(value)&&value.every(v=>typeof v==='string'));
 return <section class={css.section} aria-label={t('cache-title')}><h3 class={css.heading}>{t('cache-title')}</h3><p class={css.meta}>{t('cache-help')}</p><button type="button" class={css.ghost} onClick={()=>app.clearMailCache()}>{t('cache-clear')}</button>
 <For each={MAIL_CACHE_FIELDS}>{field=>{
  const setting=useSetting<boolean|number>(`mail.cache.${field[0]}`,field[1],(value):value is boolean|number=>typeof value===typeof field[1]);
  const disabled=()=>!setting.entry()||setting.locked()||setting.saving();
  return <div><label class={css.field}>{t(`cache-${field[0]}`)}<Show when={typeof field[1]==='boolean'} fallback={<input class={css.input} type="number" min={field.length>2?field[2]:undefined} max={field.length>2?field[3]:undefined} value={Number(setting.value())} disabled={disabled()} onChange={event=>{const value=event.currentTarget.valueAsNumber;if(typeof field[2]==='number'&&typeof field[3]==='number'&&Number.isInteger(value)&&value>=field[2]&&value<=field[3])void setting.set(value);}}/>}><input type="checkbox" checked={setting.value()===true} disabled={disabled()} onChange={event=>void setting.set(event.currentTarget.checked)}/></Show></label><LockNote locked={setting.locked()}/><Show when={setting.error()}><p role="alert">{setting.error()}</p></Show><button type="button" class={css.ghost} disabled={disabled()} onClick={()=>void setting.reset()}>{t('reader-inherit')}</button></div>;
 }}</For>
 <label class={css.field}>{t('cache-excludedFolders')}<textarea class={css.input} value={exclusions.value().join('\n')} disabled={!exclusions.entry()||exclusions.locked()||exclusions.saving()} onChange={event=>{const value=event.currentTarget.value.split(/\r?\n/).map(v=>v.trim()).filter(Boolean);void exclusions.set(value);}}/></label><p class={css.meta}>{t('cache-exclusions-help')}</p><LockNote locked={exclusions.locked()}/><Show when={exclusions.error()}><p role="alert">{exclusions.error()}</p></Show><button type="button" class={css.ghost} disabled={!exclusions.entry()||exclusions.locked()||exclusions.saving()} onClick={()=>void exclusions.reset()}>{t('reader-inherit')}</button>
 </section>;
}

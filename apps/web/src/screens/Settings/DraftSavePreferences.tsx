import {Show,type JSX} from 'solid-js';
import {useSetting} from './useSetting.ts';
import {LockNote} from './LockNote.tsx';
import {t} from '../../i18n';
import * as css from './styles.css.ts';
export function DraftSavePreferences():JSX.Element{
 const mode=useSetting<string>('mail.draftSaveMode','realtime'),debounce=useSetting<number>('mail.draftDebounceMs',1000),interval=useSetting<number>('mail.draftIntervalSecs',30);
 return <section class={css.section} aria-label={t('settings-draft-save')}><h3 class={css.heading}>{t('settings-draft-save')}</h3><p class={css.meta}>{t('settings-draft-save-help')}</p>
 <label class={css.field}>{t('settings-draft-save-mode')}<select class={css.input} value={mode.value()} disabled={!mode.entry()||mode.locked()||mode.saving()} onChange={e=>void mode.set(e.currentTarget.value)}><option value="realtime">{t('settings-draft-realtime')}</option><option value="periodic">{t('settings-draft-periodic')}</option><option value="manual">{t('settings-draft-manual')}</option></select></label><LockNote locked={mode.locked()}/>
 <Show when={mode.value()==='realtime'}><label class={css.field}>{t('settings-draft-debounce')}<input class={css.input} type="number" min="500" max="10000" step="100" value={debounce.value()} disabled={!debounce.entry()||debounce.locked()||debounce.saving()} onChange={e=>{const n=e.currentTarget.valueAsNumber;if(Number.isInteger(n)&&n>=500&&n<=10000)void debounce.set(n);}}/></label><LockNote locked={debounce.locked()}/></Show>
 <Show when={mode.value()==='periodic'}><label class={css.field}>{t('settings-draft-interval')}<input class={css.input} type="number" min="5" max="3600" value={interval.value()} disabled={!interval.entry()||interval.locked()||interval.saving()} onChange={e=>{const n=e.currentTarget.valueAsNumber;if(Number.isInteger(n)&&n>=5&&n<=3600)void interval.set(n);}}/></label><LockNote locked={interval.locked()}/></Show>
 <Show when={mode.error()||debounce.error()||interval.error()}><p role="alert">{mode.error()||debounce.error()||interval.error()}</p></Show>
 <button type="button" class={css.button} disabled={mode.locked()||debounce.locked()||interval.locked()||mode.saving()||debounce.saving()||interval.saving()} onClick={()=>{void mode.reset();void debounce.reset();void interval.reset();}}>{t('reader-inherit')}</button></section>;
}

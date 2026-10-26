import {Show,type JSX} from 'solid-js';
import {useSetting} from './useSetting.ts';
import {LockNote} from './LockNote.tsx';
import {t} from '../../i18n';
import * as css from './styles.css.ts';
import * as own from './NextcloudSyncSettings.css.ts';
export function NextcloudSyncSettings():JSX.Element {
 const setting=useSetting<number>('nextcloud.syncIntervalSecs',300,(value):value is number=>typeof value==='number'&&Number.isInteger(value));
 const disabled=()=>!setting.entry()||setting.locked()||setting.saving();
 return <div class={css.field}>
  <label for="nextcloud-sync-interval">{t('nextcloud-pim-interval')}</label>
  <input id="nextcloud-sync-interval" class={`${css.input} ${own.interval}`} type="number" min="0" max="86400" step="1" value={setting.value()} disabled={disabled()} aria-describedby="nextcloud-sync-help" onChange={event=>{if(event.currentTarget.checkValidity())void setting.set(event.currentTarget.valueAsNumber);}}/>
  <p id="nextcloud-sync-help" class={css.meta}>{t('nextcloud-pim-interval-help')}</p><LockNote locked={setting.locked()}/>
  <Show when={setting.error()}><p role="alert">{setting.error()}</p></Show>
  <div class={css.actions}><button class={css.ghost} type="button" disabled={disabled()} onClick={()=>void setting.reset()}>{t('nextcloud-pim-interval-reset')}</button></div>
 </div>;
}

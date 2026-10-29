import {For,Show,type JSX} from 'solid-js';
import {useSetting} from './useSetting.ts';
import {LockNote} from './LockNote.tsx';
import {t} from '../../i18n';
import {FOLDER_COUNT_MODES,FOLDER_COUNT_SCOPES} from '../../api/countBadges.ts';
import * as css from './styles.css.ts';
// Counter bubbles (t39): what a folder count shows, which folders show one, the
// smaller secondary size, and the opt-in bubble beside the mailbox name.
export function FolderCountPreferences():JSX.Element {
 const counts=useSetting<string>('mail.folderCounts','both'),scope=useSetting<string>('mail.folderCountScope','all'),compact=useSetting<boolean>('mail.compactFolderBadges',true),head=useSetting<boolean>('mail.accountUnreadBadge',false);
 const off=(s:{entry:()=>unknown;locked:()=>boolean;saving:()=>boolean})=>!s.entry()||s.locked()||s.saving();
 return <section class={css.section} aria-label={t('settings-folder-counts')}>
  <label class={css.field}><span>{t('settings-folder-counts')}</span><select aria-label={t('settings-folder-counts')} class={css.input} value={counts.value()} disabled={off(counts)} onChange={event=>void counts.set(event.currentTarget.value)}><For each={FOLDER_COUNT_MODES}>{value=><option value={value}>{t(`settings-folder-counts-${value}`)}</option>}</For></select></label><LockNote locked={counts.locked()}/>
  <label class={css.field}><span>{t('settings-folder-count-scope')}</span><select aria-label={t('settings-folder-count-scope')} class={css.input} value={scope.value()} disabled={off(scope)} onChange={event=>void scope.set(event.currentTarget.value)}><For each={FOLDER_COUNT_SCOPES}>{value=><option value={value}>{t(`settings-folder-count-scope-${value}`)}</option>}</For></select><p class={css.meta}>{t('settings-folder-count-scope-help')}</p></label><LockNote locked={scope.locked()}/>
  <label class={css.check}><input type="checkbox" class={css.checkbox} checked={compact.value()} disabled={off(compact)} onChange={event=>void compact.set(event.currentTarget.checked)}/>{t('settings-folder-counts-compact')}</label><LockNote locked={compact.locked()}/>
  <label class={css.check}><input type="checkbox" class={css.checkbox} checked={head.value()} disabled={off(head)} onChange={event=>void head.set(event.currentTarget.checked)}/>{t('settings-account-unread-badge')}</label><LockNote locked={head.locked()}/>
  <Show when={counts.error()||scope.error()||compact.error()||head.error()}><p role="alert">{counts.error()||scope.error()||compact.error()||head.error()}</p></Show>
  <div class={css.actions}><button class={css.ghost} type="button" disabled={off(counts)} onClick={()=>void counts.reset()}>{t('settings-folder-counts-reset')}</button><button class={css.ghost} type="button" disabled={off(compact)} onClick={()=>void compact.reset()}>{t('settings-folder-counts-reset-size')}</button></div>
 </section>;
}

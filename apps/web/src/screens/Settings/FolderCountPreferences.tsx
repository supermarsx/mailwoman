import {For,Show,type JSX} from 'solid-js';
import {useSetting} from './useSetting.ts';
import {LockNote} from './LockNote.tsx';
import {t} from '../../i18n';
import * as css from './styles.css.ts';
export function FolderCountPreferences():JSX.Element {
 const counts=useSetting<string>('mail.folderCounts','both'),compact=useSetting<boolean>('mail.compactFolderBadges',true);
 return <section class={css.section} aria-label={t('settings-folder-counts')}><label class={css.field}><span>{t('settings-folder-counts')}</span><select aria-label={t('settings-folder-counts')} class={css.input} value={counts.value()} disabled={!counts.entry()||counts.locked()||counts.saving()} onChange={event=>void counts.set(event.currentTarget.value)}><For each={['none','unread','total','both']}>{value=><option value={value}>{t(`settings-folder-counts-${value}`)}</option>}</For></select></label><LockNote locked={counts.locked()}/><label class={css.check}><input type="checkbox" class={css.checkbox} checked={compact.value()} disabled={!compact.entry()||compact.locked()||compact.saving()} onChange={event=>void compact.set(event.currentTarget.checked)}/>{t('settings-folder-counts-compact')}</label><LockNote locked={compact.locked()}/><Show when={counts.error()||compact.error()}><p role="alert">{counts.error()||compact.error()}</p></Show><div class={css.actions}><button class={css.button} type="button" disabled={!counts.entry()||counts.locked()||counts.saving()} onClick={()=>void counts.reset()}>{t('settings-folder-counts-reset')}</button><button class={css.button} type="button" disabled={!compact.entry()||compact.locked()||compact.saving()} onClick={()=>void compact.reset()}>{t('settings-folder-counts-reset-size')}</button></div></section>;
}

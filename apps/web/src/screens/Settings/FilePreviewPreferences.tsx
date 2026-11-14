import {createEffect,createMemo,createSignal,Show,type JSX} from 'solid-js';
import {FilePreviewEditor} from '../../components/FilePreviewEditor.tsx';
import {DEFAULT_FILE_PREVIEWS,FILE_PREVIEWS_KEY,validFilePreviews,type FilePreviewConfig} from '../../viewers/filePreviewPreferences.ts';
import {useSetting} from './useSetting.ts';
import {useEditorGuard} from './entityEditor.tsx';
import {SettingsCard} from './layout/SettingsCard.tsx';
import {ControlRow} from './layout/SettingRow.tsx';
import {LockNote} from './LockNote.tsx';
import type {CardSpec} from './layout/rows.tsx';
import {t} from '../../i18n';
import * as css from './styles.css.ts';
export const FILE_PREVIEWS_CARD:CardSpec={id:'file-previews-card',section:'reader',title:'preview-title',help:'preview-help',rows:[{id:'file-previews-editor',label:'preview-title',keywords:'attachment file preview pdf image text txt html code formats tooling toolbar menu hide'}]};
export function FilePreviewPreferences():JSX.Element{
 const setting=useSetting(FILE_PREVIEWS_KEY,DEFAULT_FILE_PREVIEWS,validFilePreviews),[draft,setDraft]=createSignal<FilePreviewConfig>(structuredClone(setting.value()));
 const saved=createMemo(()=>JSON.stringify(setting.value()));createEffect(()=>setDraft(JSON.parse(saved()) as FilePreviewConfig));
 const dirty=()=>JSON.stringify(draft())!==saved(),disabled=()=>!setting.entry()||setting.locked()||setting.saving(),guard=useEditorGuard(dirty,setting.saving);
 return <SettingsCard id={FILE_PREVIEWS_CARD.id} title={t('preview-title')} description={t('preview-help')}><LockNote locked={setting.locked()}/><ControlRow controlId="file-previews-editor" label={t('preview-title')} group>{()=> <FilePreviewEditor value={draft()} onChange={setDraft} disabled={disabled()}/>}</ControlRow><Show when={!validFilePreviews(draft())}><p role="alert">{t('preview-invalid')}</p></Show><Show when={setting.error()}><p role="alert">{setting.error()}</p></Show><div class={css.actions}><button type="button" class={css.button} disabled={disabled()||!dirty()||!validFilePreviews(draft())} onClick={()=>void setting.set(draft())}>{t('preview-save')}</button><button type="button" class={css.ghost} disabled={disabled()} onClick={()=>void setting.reset()}>{t('reader-inherit')}</button></div>{guard.dialog()}</SettingsCard>;
}

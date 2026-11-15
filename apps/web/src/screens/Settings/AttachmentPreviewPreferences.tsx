import {ATTACHMENT_WARNING_TYPES} from '../../components/AttachmentWarningBar.tsx';
import {For,type JSX} from 'solid-js';
import {loadCatalog,t} from '../../i18n';
import {ATTACHMENT_ACTION_STYLE,ATTACHMENT_THUMBNAILS,type AttachmentActionStyle} from '../../viewers/previewPreferences.ts';
import {useSetting} from './useSetting.ts';
import {SettingRow} from './layout/SettingRow.tsx';
import {SettingsCard} from './layout/SettingsCard.tsx';
import type {CardSpec} from './layout/rows.tsx';
import * as css from './styles.css.ts';
void loadCatalog('attachment-preview');
export const ATTACHMENT_PREVIEW_CARD:CardSpec={id:'attachment-preview-card',section:'reader',title:'attachment-preview-title',help:'attachment-preview-help',rows:[{id:'attachment-preview-actions',label:'attachment-preview-actions',help:'attachment-preview-actions-help'},{id:'attachment-preview-thumbnails',label:'attachment-preview-thumbnails',help:'attachment-preview-thumbnails-help',keywords:'images thumbnails pictures attachments preview'}]};
export function AttachmentPreviewPreferences():JSX.Element{
 const actions=useSetting<AttachmentActionStyle>(ATTACHMENT_ACTION_STYLE,'icons', (value):value is AttachmentActionStyle=>value==='icons'||value==='labels'||value==='icons-labels');
 const warningBar=useSetting('mail.attachmentWarnings',true),warningTypes=useSetting<string[]>('mail.attachmentWarningTypes',[...ATTACHMENT_WARNING_TYPES]),required=useSetting('mail.attachmentWarningsRequired',false);
 const thumbs=useSetting(ATTACHMENT_THUMBNAILS,false);
 return <SettingsCard id={ATTACHMENT_PREVIEW_CARD.id} title={t('attachment-preview-title')} description={t('attachment-preview-help')}>
  <SettingRow controlId="attachment-preview-actions" label={t('attachment-preview-actions')} help={t('attachment-preview-actions-help')} handle={actions}>{description=><select id="attachment-preview-actions" class={css.select} value={actions.value()} disabled={!actions.entry()||actions.locked()||actions.saving()} aria-describedby={description()} onChange={event=>void actions.set(event.currentTarget.value as AttachmentActionStyle)}><For each={['icons','labels','icons-labels']}>{style=><option value={style}>{t(`attachment-preview-style-${style}`)}</option>}</For></select>}</SettingRow>
  <SettingRow controlId="attachment-preview-thumbnails" label={t('attachment-preview-thumbnails')} help={t('attachment-preview-thumbnails-help')} handle={thumbs}>{description=><input id="attachment-preview-thumbnails" type="checkbox" checked={thumbs.value()} disabled={!thumbs.entry()||thumbs.locked()||thumbs.saving()} aria-describedby={description()} onChange={event=>void thumbs.set(event.currentTarget.checked)}/>}</SettingRow>
 <SettingRow controlId="attachment-warning-bar" label={t('mail-attachment-warning-title')} help={t('mail-attachment-warning-help')} handle={warningBar}>{description=><input id="attachment-warning-bar" type="checkbox" aria-describedby={description()} checked={required.value()||warningBar.value()} disabled={!warningBar.entry()||warningBar.locked()||warningBar.saving()||required.value()} onChange={event=>void warningBar.set(event.currentTarget.checked)}/>}</SettingRow>
 <fieldset class={css.bareFieldset} disabled={!warningTypes.entry()||warningTypes.locked()||warningTypes.saving()||required.value()}><For each={ATTACHMENT_WARNING_TYPES}>{kind=><label class={css.check}><input type="checkbox" checked={required.value()||warningTypes.value().includes(kind)} onChange={event=>void warningTypes.set(event.currentTarget.checked?[...warningTypes.value(),kind]:warningTypes.value().filter(value=>value!==kind))}/><span>{t(`mail-file-warning-${kind}`)}</span></label>}</For></fieldset>
 </SettingsCard>;
}

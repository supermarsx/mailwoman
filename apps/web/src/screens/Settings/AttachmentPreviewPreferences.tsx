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
 const thumbs=useSetting(ATTACHMENT_THUMBNAILS,false);
 return <SettingsCard id={ATTACHMENT_PREVIEW_CARD.id} title={t('attachment-preview-title')} description={t('attachment-preview-help')}>
  <SettingRow controlId="attachment-preview-actions" label={t('attachment-preview-actions')} help={t('attachment-preview-actions-help')} handle={actions}>{description=><select id="attachment-preview-actions" class={css.select} value={actions.value()} disabled={!actions.entry()||actions.locked()||actions.saving()} aria-describedby={description()} onChange={event=>void actions.set(event.currentTarget.value as AttachmentActionStyle)}><For each={['icons','labels','icons-labels']}>{style=><option value={style}>{t(`attachment-preview-style-${style}`)}</option>}</For></select>}</SettingRow>
  <SettingRow controlId="attachment-preview-thumbnails" label={t('attachment-preview-thumbnails')} help={t('attachment-preview-thumbnails-help')} handle={thumbs}>{description=><input id="attachment-preview-thumbnails" type="checkbox" checked={thumbs.value()} disabled={!thumbs.entry()||thumbs.locked()||thumbs.saving()} aria-describedby={description()} onChange={event=>void thumbs.set(event.currentTarget.checked)}/>}</SettingRow>
 </SettingsCard>;
}

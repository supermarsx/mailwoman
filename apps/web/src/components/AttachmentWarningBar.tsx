import {createMemo,createResource,For,Show} from 'solid-js';
import {settings} from '../api/settings.ts';
import {attachmentDecision,loadAttachmentPolicy} from '../api/attachmentPolicy.ts';
import {inspectFile,type FileWarning} from '../viewers/fileInfo.ts';
import type {AttachmentPart} from '../viewers/attachments.ts';
import {Icon} from './Icon.tsx';
import {loadCatalog,t} from '../i18n';
import * as css from './attachmentWarningBar.css.ts';
void loadCatalog('mail');
export const ATTACHMENT_WARNING_TYPES:readonly FileWarning[]=['executable','macro','disguisedName','typeMismatch','activeDocument'];
export function AttachmentWarningBar(props:{parts:readonly AttachmentPart[]}){
 const required=()=>settings()['mail.attachmentWarningsRequired']?.value===true;
 const allowed=()=>{const value=settings()['mail.attachmentWarningTypes']?.value;return required()||!Array.isArray(value)?ATTACHMENT_WARNING_TYPES:value;};
 const [policy]=createResource(()=>props.parts.length>0,loadAttachmentPolicy);
 const warnings=createMemo(()=>props.parts.flatMap(part=>{
  const name=part.name||t('mail-attachment-unnamed'),mime=part.type||'application/octet-stream';
  const types=inspectFile(name,mime).warnings.filter(kind=>allowed().includes(kind));
  const adminWarning=policy.state==='ready'&&attachmentDecision(policy.latest!,{name,mime,size:part.size}).warning;
  return types.length||adminWarning?[{name,types,adminWarning}]:[];
 }));
 return <Show when={(required()||settings()['mail.attachmentWarnings']?.value!==false)&&warnings().length}><section class={css.bar} role="note" aria-label={t('mail-attachment-warning-title')}><Icon name="alert"/><div class={css.body}><strong>{t('mail-attachment-warning-title')}</strong><p>{t('mail-attachment-warning-help')}</p><ul><For each={warnings()}>{item=><li><bdi>{item.name}</bdi><span><For each={item.types}>{kind=><span>{t(`mail-file-warning-${kind}`)}</span>}</For><Show when={item.adminWarning}><span>{t('mail-attachment-policy-warning')}</span></Show></span></li>}</For></ul></div></section></Show>;
}

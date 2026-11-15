import {For,createUniqueId,type JSX} from 'solid-js';
import {DEFAULT_FILE_PREVIEWS,PREVIEW_KINDS,PREVIEW_TOOLS,type FilePreviewConfig,type PreviewPlacement} from '../viewers/filePreviewPreferences.ts';
import {Icon,type IconName} from './Icon.tsx';
import {loadCatalog,t} from '../i18n';
import * as css from '../screens/Settings/styles.css.ts';
import * as layout from './filePreviewEditor.css.ts';
void loadCatalog('file-previews');for(const catalog of ['image-viewer','text-viewer','attachment-preview','reader','reader-actions'])void loadCatalog(catalog);
const groups=[['common','preview-common','settings'],['image','preview-kind-image','image'],['pdf','preview-kind-pdf','filePdf'],['text','preview-kind-code','fileCode'],['html','preview-kind-html','fileCode'],['media','preview-media','fileVideo']] as const satisfies readonly (readonly [string,string,IconName])[];
export function FilePreviewEditor(props:{value:FilePreviewConfig;onChange:(value:FilePreviewConfig)=>void;disabled?:boolean|undefined;id?:string;describedBy?:string|undefined}):JSX.Element{
 const prefix=createUniqueId(),patch=(value:Partial<FilePreviewConfig>)=>props.onChange({...props.value,...value});
 const extensions=(field:'imageFormats'|'textExtensions')=><label class={css.field}>{t(`preview-${field}`)}<input class={css.input} value={props.value[field].join(', ')} onChange={event=>patch({[field]:event.currentTarget.value.toLowerCase().split(/[\s,]+/).map(value=>value.replace(/^\./,'')).filter(Boolean)})}/><span class={css.meta}>{t(`preview-${field}-help`)}</span></label>;
 const setTool=(id:string,value:string)=>{const tools={...props.value.tools};if(value==='default')delete tools[id];else tools[id]=value as PreviewPlacement;patch({tools});};
 return <fieldset class={`${css.bareFieldset} ${layout.editor}`} disabled={props.disabled} id={props.id} aria-describedby={props.describedBy}>
  <section class={layout.group}>
   <label class={css.check}><input type="checkbox" checked={props.value.enabled} onChange={event=>patch({enabled:event.currentTarget.checked})}/><span>{t('preview-enabled')}</span></label><p class={css.meta}>{t('preview-enabled-help')}</p>
   <label class={css.field}>{t('preview-max-size')}<input class={css.input} type="number" min={1} max={256} step={1} value={props.value.maxBytes/1048576} onChange={event=>{if(event.currentTarget.checkValidity())patch({maxBytes:Math.round(event.currentTarget.valueAsNumber*1048576)});}}/><span class={css.meta}>{t('preview-max-size-help')}</span></label>
  </section>
  <section class={layout.group} aria-labelledby={`${prefix}-types`}><h3 class={layout.heading} id={`${prefix}-types`}>{t('preview-formats-title')}</h3><div class={layout.formats}><For each={PREVIEW_KINDS}>{kind=><div class={layout.format}><label class={css.check}><input type="checkbox" checked={props.value.types[kind]!==false} onChange={event=>patch({types:{...props.value.types,[kind]:event.currentTarget.checked}})}/><span>{t(`preview-kind-${kind}`)}</span></label><p class={css.meta}>{t(`preview-supported-${kind}`)}</p></div>}</For></div></section>
  <div><h3 class={layout.heading}>{t('preview-tools-title')}</h3><p class={css.meta}>{t('preview-tools-help')}</p></div>
  <For each={groups}>{([group,label,icon])=><section class={layout.group} aria-labelledby={`${prefix}-${group}`}><h4 class={layout.heading} id={`${prefix}-${group}`}><Icon name={icon}/>{t(label)}{group==='pdf'?' (PDF.js)':''}</h4>
   {group==='image'?<><label class={css.check}><input type="checkbox" checked={props.value.convertImages} onChange={event=>patch({convertImages:event.currentTarget.checked})}/><span>{t('preview-convert-images')}</span></label><p class={css.meta}>{t('preview-convert-images-help')}</p><details><summary class={layout.advanced}>{t('preview-advanced')}</summary><div class={layout.advancedBody}>{extensions('imageFormats')}</div></details></>:null}
   {group==='text'?<details><summary class={layout.advanced}>{t('preview-advanced')}</summary><div class={layout.advancedBody}>{extensions('textExtensions')}</div></details>:null}
   <For each={PREVIEW_TOOLS.filter(tool=>group==='common'?!tool.id.includes('.'):tool.id.startsWith(`${group}.`))}>{tool=><label class={layout.tool} for={`${prefix}-${tool.id}`}><span>{t(tool.label)}</span><select id={`${prefix}-${tool.id}`} class={css.select} value={props.value.tools[tool.id]??'default'} onChange={event=>setTool(tool.id,event.currentTarget.value)}><option value="default">{t('preview-default-layout')}</option><For each={['toolbar','menu','hidden']}>{placement=><option value={placement}>{t(`reader-actions-option-${placement}`)}</option>}</For></select></label>}</For>
  </section>}</For>
  <div class={css.actions}><button type="button" class={css.ghost} onClick={()=>props.onChange(structuredClone(DEFAULT_FILE_PREVIEWS))}>{t('preview-restore')}</button></div>
 </fieldset>;
}

import {createResource,onCleanup,onMount,For,Show,type JSX} from 'solid-js';
import {loadPolicyDocument,loadProcessingDocument,type ProcessingDocument} from '../../api/privacy.ts';
import {withBase} from '../../api/basePath.ts';
import {themedPolicyUrl} from '../../theme/publicPolicyTheme.ts';
import {loadCatalog,t,formatLocale} from '../../i18n';
import {Icon} from '../../components/Icon.tsx';
import {resourceValue} from '../../components/resourceValue.ts';
import * as css from './styles.css.ts';
import * as layout from './privacyData.css.ts';
export type PrivacyDocumentKind='policy'|'processing';
export function PrivacyDocumentPage(props:{kind:PrivacyDocumentKind;source:string;onBack:()=>void}):JSX.Element{
 onMount(()=>void loadCatalog('your-data'));
 let controller:AbortController|undefined;onCleanup(()=>controller?.abort());
 const [content,{refetch}]=createResource(()=>({kind:props.kind,source:props.source,lang:formatLocale().startsWith('pt')?'pt-PT':'en'}),async request=>{
  controller?.abort();controller=new AbortController();return request.kind==='policy'?{policy:await loadPolicyDocument(request.source,request.lang,controller.signal)}:{processing:await loadProcessingDocument(request.lang,controller.signal)};
 });
 const value=()=>resourceValue(content);
 return <div class={layout.document} data-testid={`privacy-document-${props.kind}`}>
  <div class={css.actions}><button type="button" class={`${css.ghost} ${layout.action}`} onClick={props.onBack}><Icon name="chevronLeft"/>{t('common-back')}</button><h2 class={css.heading}>{t(props.kind==='policy'?'yd-link-policy':'yd-link-processing')}</h2></div>
  <Show when={props.kind==='policy'}><a class={`${css.ghost} ${layout.action}`} href={themedPolicyUrl(withBase(props.source))} target="_blank" rel="noopener noreferrer"><Icon name="openTab"/>{t('common-open')}</a></Show>
  <Show when={content.loading&&!value()}><p role="status" class={css.meta}>{t('common-loading')}</p></Show>
  <Show when={content.error}><section class={css.section}><p role="alert">{t('yd-unavailable')}</p><button type="button" class={css.ghost} onClick={()=>void refetch()}>{t('common-retry')}</button></section></Show>
  <Show when={value()?.policy}>{policy=><><p class={css.meta}>{t('yd-policy-version',{version:policy().version,date:policy().date})}</p><For each={policy().sections}>{section=><section class={css.section}><h3 class={css.subheading}>{section.title}</h3><p class={layout.reading}>{section.body}</p></section>}</For></>}</Show>
  <Show when={value()?.processing}>{processing=><ProcessingRecords value={processing()}/>}</Show>
 </div>;
}
function ProcessingRecords(props:{value:ProcessingDocument}):JSX.Element{
 return <><section class={css.section}><h3 class={css.subheading}>{t('yd-doc-controller')}</h3><dl class={layout.facts}><dt>{t('yd-doc-controller')}</dt><dd>{[props.value.controller.name,props.value.controller.address].filter(Boolean).join(' · ')||'—'}</dd><dt>{t('yd-doc-contact')}</dt><dd>{[props.value.contact.name,props.value.contact.email,props.value.contact.url].filter(Boolean).join(' · ')||'—'}</dd><dt>{t('yd-doc-authority')}</dt><dd>{props.value.supervisoryAuthority||'—'}</dd></dl></section>
 <Show when={props.value.activities.length} fallback={<p class={css.meta}>{t('yd-doc-empty')}</p>}><For each={props.value.activities}>{record=><section class={css.section}><h3 class={css.subheading}>{record.name}</h3><p class={layout.reading}>{record.purpose}</p><dl class={layout.facts}><For each={[[t('yd-doc-data'),record.categories],[t('yd-doc-basis'),record.legalBasis],[t('yd-col-retention'),record.retention],[t('yd-col-processors'),record.recipients]]}>{pair=><><dt>{pair[0]}</dt><dd>{pair[1]||'—'}</dd></>}</For></dl></section>}</For></Show>
 <Show when={props.value.recipients.some(recipient=>recipient.enabled)}><section class={css.section}><h3 class={css.subheading}>{t('yd-col-processors')}</h3><ul><For each={props.value.recipients.filter(recipient=>recipient.enabled)}>{recipient=><li>{recipient.name}</li>}</For></ul></section></Show></>;
}

import {createResource,createSignal,Show,type JSX} from 'solid-js';
import type {ViewerProps} from '../contracts/viewer.ts';
import {loadText} from './TextViewer.tsx';
import {sanitizeBody} from '../sanitize/index.ts';
import {escapeHtml,SANDBOX_TOKENS} from './sandbox.ts';
import {ViewerToolbar} from './ViewerToolbar.tsx';
import {previewToolEnabled} from './filePreviewPreferences.ts';
import TextViewer from './TextViewer.tsx';
import {t,loadCatalog} from '../i18n';
export function HtmlViewer(props:ViewerProps):JSX.Element{
 void loadCatalog('file-previews');const [rendered,setRendered]=createSignal(true);
 const [document]=createResource(()=>props.blobUrl,async url=>{const result=await loadText(url,props.mime);if(result.kind!=='text')return '';
  const safe=await sanitizeBody({op:'readerSafe',html:result.text,blockTrackers:true,cleanLinks:true});
  const dom=new DOMParser().parseFromString(safe,'text/html');for(const link of dom.querySelectorAll('a'))link.removeAttribute('href');for(const element of dom.querySelectorAll('[src],[srcset],[background]')){if(!/^data:image\/(?:png|jpeg|gif|webp|avif);base64,/i.test(element.getAttribute('src')??''))element.removeAttribute('src');element.removeAttribute('srcset');element.removeAttribute('background');}
  return `<!doctype html><html><head><meta charset="utf-8"><meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src data:; style-src 'unsafe-inline'; base-uri 'none'; form-action 'none';"><meta name="referrer" content="no-referrer"><title>${escapeHtml(props.name)}</title><style>body{margin:1rem;overflow-wrap:anywhere}img{max-width:100%;height:auto}</style></head><body>${dom.body.innerHTML}</body></html>`;
 });
 return <div class="mw-text"><div class="mw-text__bar"><ViewerToolbar scope={props.blobUrl} actions={[{id:'html.render',label:t('preview-tool-html-render'),icon:'eye',pressed:rendered(),run:()=>{setRendered(true);}},{id:'html.source',label:t('preview-tool-html-source'),icon:'fileCode',pressed:!rendered(),run:()=>{setRendered(false);}}]}/></div>
  <Show when={rendered()&&previewToolEnabled('html.render')} fallback={<Show when={previewToolEnabled('html.source')} fallback={<p>{t('common-viewer-no-preview')}</p>}><TextViewer {...props}/></Show>}><Show when={!document.loading} fallback={<p role="status">{t('common-loading')}</p>}><Show when={!document.error} fallback={<p role="alert">{t('text-viewer-failed')}</p>}><iframe class="mw-viewer__frame" title={props.name} sandbox={SANDBOX_TOKENS} referrerPolicy="no-referrer" srcdoc={document()??''}/></Show></Show></Show>
 </div>;
}
export default HtmlViewer;

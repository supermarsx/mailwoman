import { createEffect, createSignal, onCleanup, onMount, Show, type JSX } from 'solid-js';
import { withBase } from '../api/basePath.ts';
import { t } from '../i18n/index.ts';
/** The wrapper runs only its fixed bridge; actual mail always uses sandbox="". */
export function DirectReaderFrame(props: { html: string; title: string; documentKey?:string; onDisplayed?: (html:string,key:string|undefined)=>void }): JSX.Element {
  let frame: HTMLIFrameElement | undefined;
  const token = Array.from(crypto.getRandomValues(new Uint8Array(24)), byte => byte.toString(16).padStart(2, '0')).join('');
  const [ready, setReady] = createSignal(false);
  const [failed, setFailed] = createSignal(false);
  const fragment = new URLSearchParams({ token, origin: location.origin });
  let timeout: ReturnType<typeof setTimeout> | undefined;
  let revision=0;let sentHtml='';let sentKey:string|undefined;
  onMount(() => {
    const receive = (event: MessageEvent): void => {
      if (event.source !== frame?.contentWindow || event.origin !== 'null' || event.data?.token !== token) return;
      if(event.data?.type==='mw-reader-displayed'){
        if(event.data.revision===revision&&revision>0&&sentHtml===props.html&&sentKey===props.documentKey)props.onDisplayed?.(sentHtml,sentKey);
        return;
      }
      if(event.data?.type!=='mw-reader-ready')return;
      clearTimeout(timeout); setReady(true); setFailed(false);
    };
    window.addEventListener('message', receive);
    timeout = setTimeout(() => setFailed(true), 10_000);
    onCleanup(() => { clearTimeout(timeout); window.removeEventListener('message', receive); });
  });
  createEffect(() => { const html = props.html,key=props.documentKey; if (ready()) {sentHtml=html;sentKey=key;frame?.contentWindow?.postMessage({ type: 'mw-reader-frame', token, html, revision:++revision }, '*');} });
  return <><Show when={failed()}><p role="alert">{t('privacy-frame-failed')}</p></Show><iframe ref={frame} class="reader__frame" title={props.title} sandbox="allow-scripts" referrerPolicy="no-referrer" src={`${withBase('/api/rc/frame')}#${fragment}`} /></>;
}

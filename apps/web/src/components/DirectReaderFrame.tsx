import { batch, createEffect, createSignal, onCleanup, onMount, Show, type JSX } from 'solid-js';
import { withBase } from '../api/basePath.ts';
import { t } from '../i18n/index.ts';
/** The wrapper runs only its fixed bridge; actual mail always uses sandbox="". */
export function DirectReaderFrame(props: { html: string; title: string; documentKey?:string; onDisplayed?: (html:string,key:string|undefined)=>void }): JSX.Element {
  let frame: HTMLIFrameElement | undefined;
  const newToken = ():string => Array.from(crypto.getRandomValues(new Uint8Array(24)), byte => byte.toString(16).padStart(2, '0')).join('');
  const [token,setToken] = createSignal(newToken());
  const [compatible,setCompatible] = createSignal(false);
  const [ready, setReady] = createSignal(false);
  const [failed, setFailed] = createSignal(false);
  const fragment = ():URLSearchParams => new URLSearchParams({ token:token(), origin: location.origin });
  let timeout: ReturnType<typeof setTimeout> | undefined;
  let fallback: ReturnType<typeof setTimeout> | undefined;
  let loadedFallback: ReturnType<typeof setTimeout> | undefined;
  let revision=0;let sentHtml='';let sentKey:string|undefined;
  function useCompatibleFrame():void {
    if(ready()||compatible()||failed())return;
    clearTimeout(fallback);clearTimeout(loadedFallback);
    batch(()=>{setToken(newToken());setCompatible(true);});
  }
  function loaded():void {
    clearTimeout(loadedFallback);
    // Blocked frames may fire load. Only the correlated bridge message is ready.
    loadedFallback=setTimeout(useCompatibleFrame,250);
  }
  onMount(() => {
    const receive = (event: MessageEvent): void => {
      if (failed() || event.source !== frame?.contentWindow || event.origin !== 'null' || event.data?.token !== token()) return;
      if(event.data?.type==='mw-reader-displayed'){
        if(event.data.revision===revision&&revision>0&&sentHtml===props.html&&sentKey===props.documentKey)props.onDisplayed?.(sentHtml,sentKey);
        return;
      }
      if(event.data?.type!=='mw-reader-ready')return;
      clearTimeout(timeout);clearTimeout(fallback);clearTimeout(loadedFallback); setReady(true); setFailed(false);
    };
    window.addEventListener('message', receive);
    fallback = setTimeout(useCompatibleFrame,1500);
    timeout = setTimeout(() => setFailed(true), 10_000);
    onCleanup(() => { clearTimeout(timeout);clearTimeout(fallback);clearTimeout(loadedFallback); window.removeEventListener('message', receive); });
  });
  createEffect(() => { const html = props.html,key=props.documentKey; if (ready()) {sentHtml=html;sentKey=key;frame?.contentWindow?.postMessage({ type: 'mw-reader-frame', token:token(), html, revision:++revision }, '*');} });
  return <><Show when={failed()}><p role="alert">{t('privacy-frame-failed')}</p></Show><Show when={compatible()&&ready()}><p class="meta" role="status">{t('privacy-frame-compatible')}</p></Show><iframe ref={frame} class="reader__frame" title={props.title} sandbox="allow-scripts" referrerPolicy="no-referrer" onLoad={loaded} src={`${withBase('/api/rc/frame')}${compatible()?'?coep=require-corp':''}#${fragment()}`} /></>;
}

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
  // Each bridge document announces itself once; a new one needs the body again.
  const [bridges, setBridges] = createSignal(0);
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
    // Moved into a detached window, the frame reloads a new bridge whose ready
    // went to a window not listened on yet: listen there and ask again.
    if(listen(frame?.ownerDocument.defaultView))frame?.contentWindow?.postMessage({type:'mw-reader-ping',token:token()},'*');
    clearTimeout(loadedFallback);
    // Blocked frames may fire load. Only the correlated bridge message is ready.
    loadedFallback=setTimeout(useCompatibleFrame,250);
  }
  const hosts = new Set<Window>();
  let receive: ((event: MessageEvent) => void) | undefined;
  /** True when `host` is newly listened on. */
  function listen(host: Window | null | undefined): boolean {
    if (!host || !receive || hosts.has(host)) return false;
    hosts.add(host);
    host.addEventListener('message', receive);
    return true;
  }
  onMount(() => {
    receive = (event: MessageEvent): void => {
      if (failed() || event.source !== frame?.contentWindow || event.origin !== 'null' || event.data?.token !== token()) return;
      if(event.data?.type==='mw-reader-displayed'){
        if(event.data.revision===revision&&revision>0&&sentHtml===props.html&&sentKey===props.documentKey)props.onDisplayed?.(sentHtml,sentKey);
        return;
      }
      if(event.data?.type!=='mw-reader-ready')return;
      clearTimeout(timeout);clearTimeout(fallback);clearTimeout(loadedFallback); batch(()=>{setReady(true); setFailed(false); setBridges(n=>n+1);});
    };
    // A detached workspace window shows this frame in its own document while
    // the code runs in the main window; the element is created here and moved
    // there, and the bridge posts to the window that holds it. Listen on that
    // window too, from the frame's first load.
    listen(window);
    fallback = setTimeout(useCompatibleFrame,1500);
    timeout = setTimeout(() => setFailed(true), 10_000);
    onCleanup(() => { clearTimeout(timeout);clearTimeout(fallback);clearTimeout(loadedFallback); for (const host of hosts) if (receive) host.removeEventListener('message', receive); hosts.clear(); });
  });
  createEffect(() => { const html = props.html,key=props.documentKey; void bridges(); if (ready()) {sentHtml=html;sentKey=key;frame?.contentWindow?.postMessage({ type: 'mw-reader-frame', token:token(), html, revision:++revision }, '*');} });
  return <><Show when={failed()}><p role="alert">{t('privacy-frame-failed')}</p></Show><Show when={compatible()&&ready()}><p class="meta" role="status">{t('privacy-frame-compatible')}</p></Show><iframe ref={frame} class="reader__frame" title={props.title} sandbox="allow-scripts" referrerPolicy="no-referrer" onLoad={loaded} src={`${withBase('/api/rc/frame')}${compatible()?'?coep=require-corp':''}#${fragment()}`} /></>;
}

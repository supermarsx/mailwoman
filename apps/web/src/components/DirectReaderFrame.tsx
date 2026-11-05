import { batch, createEffect, createSignal, onCleanup, onMount, Show, untrack, type JSX } from 'solid-js';
import { remoteServerBase } from '../api/remote-content.ts';
import { diffReaderDocument, patchPrefixes } from '../viewers/readerPatch.ts';
import { withBase } from '../api/basePath.ts';
import {isNativeAuth,nativeServerBase} from '../api/serverAddress.ts';
import {readerAddressContext,type ReaderAddressContext} from '../viewers/readerAddressContext.ts';
import { t } from '../i18n/index.ts';
import { ReaderZoomBox } from './ReaderZoomBox.tsx';
import {MessageLoading} from './MessageLoading.tsx';
import './readerPane.css.ts';
export interface ReaderImageProgress {loaded:number;failed:number;pending:number;total:number}
function imageProgress(value:Record<string,unknown>):ReaderImageProgress|null {
  const {loaded,failed,pending,total}=value;
  if(![loaded,failed,pending,total].every(count=>typeof count==='number'&&Number.isSafeInteger(count)&&count>=0&&count<=1_000_000))return null;
  return (loaded as number)+(failed as number)+(pending as number)===total ? {loaded:loaded as number,failed:failed as number,pending:pending as number,total:total as number} : null;
}
/** Only fixed hash-authorized handlers run; both frames retain opaque origins.
 * The Reader remounts this wrapper when message-link consent changes. */
export function DirectReaderFrame(props: { html: string; linksEnabled?:boolean; title: string; documentKey?:string; loadingFallback?:JSX.Element; onDisplayed?: (html:string,key:string|undefined)=>void; onAddressContext?:(value:ReaderAddressContext)=>void; onImageProgress?:(value:ReaderImageProgress)=>void }): JSX.Element {
  let frame: HTMLIFrameElement | undefined;
  const newToken = ():string => Array.from(crypto.getRandomValues(new Uint8Array(24)), byte => byte.toString(16).padStart(2, '0')).join('');
  const [token,setToken] = createSignal(newToken());
  const [compatible,setCompatible] = createSignal(false);
  const [ready, setReady] = createSignal(false);
  // Each bridge document announces itself once; a new one needs the body again.
  const [bridges, setBridges] = createSignal(0);
  const [failed, setFailed] = createSignal(false);
  const [presentedRevision,setPresentedRevision]=createSignal(0);
  const fragment = ():URLSearchParams => new URLSearchParams({ token:token(), origin: location.origin });
  const source = ():string => {
    const native = isNativeAuth(), base = native ? nativeServerBase() : null;
    if(native&&!base)return 'about:blank';
    const query = new URLSearchParams();
    if(compatible()||native)query.set('coep','require-corp');
    if(native)query.set('parentOrigin',location.origin);
    return `${base ? base+'/api/rc/frame' : withBase('/api/rc/frame')}${query.size?'?'+query:''}#${fragment()}`;
  };
  let timeout: ReturnType<typeof setTimeout> | undefined;
  let fallback: ReturnType<typeof setTimeout> | undefined;
  let loadedFallback: ReturnType<typeof setTimeout> | undefined;
  let revision=0;let sentHtml='';let sentKey:string|undefined;let sentLinks=false;let sentBridge=0;let patchId=0;
  // A same-message refresh keeps the shown document visible until its replacement is up (or patched in place).
  const [holding,setHolding]=createSignal(false);
  // Bumped when `sentHtml` moves without a new document (a patch), so the view re-evaluates `settled`.
  const [sentTick,setSentTick]=createSignal(0);
  let paintHost:Window|null=null;let paint:number|undefined;
  const settled=():boolean=>{void sentTick();const visible=presentedRevision();return ready()&&revision>0&&visible===revision&&sentHtml===props.html&&sentKey===props.documentKey;};
  const presented=():boolean=>settled()||(holding()&&ready()&&!failed()&&sentKey===props.documentKey);
  function cancelPaint():void {if(paint!==undefined)paintHost?.cancelAnimationFrame(paint);paint=undefined;paintHost=null;}
  function present():void {
    if(settled()||paint!==undefined)return;
    const current=revision,html=sentHtml,key=sentKey,host=frame?.ownerDocument.defaultView;
    if(!host)return;
    paintHost=host;
    paint=host.requestAnimationFrame(()=>{paint=host.requestAnimationFrame(()=>{
      paint=undefined;paintHost=null;
      if(!failed()&&ready()&&revision===current&&html===props.html&&key===props.documentKey){batch(()=>{setPresentedRevision(current);setHolding(false);});props.onDisplayed?.(html,key);}
    });});
  }
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
      if(event.data?.type==='mw-reader-address-context'){
        if(ready()&&frame&&event.data.revision===revision&&revision>0&&sentHtml===props.html&&sentKey===props.documentKey){const context=readerAddressContext(event.data,frame);if(context)props.onAddressContext?.(context);}
        return;
      }
      if(event.data?.type==='mw-reader-image-progress'){
        if(ready()&&event.data.revision===revision&&revision>0&&sentHtml===props.html&&sentKey===props.documentKey){const progress=imageProgress(event.data);if(progress)props.onImageProgress?.(progress);}
        return;
      }
      if(event.data?.type==='mw-reader-patched'){
        // The document refused the patch (or it could not be delivered): redo it as a full, flicker-free refresh.
        if(event.data.ok===false&&event.data.revision===revision&&event.data.id===patchId&&ready())sendFrame(true);
        return;
      }
      if(event.data?.type==='mw-reader-displayed'){
        if(event.data.revision===revision&&revision>0&&sentHtml===props.html&&sentKey===props.documentKey)present();
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
    onCleanup(() => {cancelPaint();clearTimeout(timeout);clearTimeout(fallback);clearTimeout(loadedFallback); for (const host of hosts) if (receive) host.removeEventListener('message', receive); hosts.clear(); });
  });
  function sendFrame(swap:boolean):void {
    const html=props.html,key=props.documentKey,linksEnabled=props.linksEnabled===true;
    sentHtml=html;sentKey=key;sentLinks=linksEnabled;
    cancelPaint();
    if(swap)setHolding(true);else{setHolding(false);setPresentedRevision(0);}
    frame?.contentWindow?.postMessage({ type: 'mw-reader-frame', token:token(), html, linksEnabled, swap, revision:++revision }, '*');
  }
  createEffect(() => {
    const html = props.html,key=props.documentKey,linksEnabled=props.linksEnabled===true,bridge=bridges();
    if (!ready()) return;
    untrack(() => {
      const sameBridge=bridge===sentBridge;sentBridge=bridge;
      const shown=sameBridge&&revision>0&&sentKey===key&&(holding()||presentedRevision()===revision);
      if(!shown){sendFrame(false);return;}
      if(html===sentHtml&&linksEnabled===sentLinks)return;
      // Image toggles only change image references: patch the document that is on screen.
      if(!holding()&&linksEnabled===sentLinks){
        const ops=diffReaderDocument(sentHtml,html,patchPrefixes(remoteServerBase()));
        if(ops){
          sentHtml=html;setSentTick(n=>n+1);
          if(ops.length>0)frame?.contentWindow?.postMessage({ type:'mw-reader-patch', token:token(), revision, id:++patchId, ops }, '*');
          return;
        }
      }
      sendFrame(true);
    });
  });
  return <><Show when={failed()}><p role="alert">{t('privacy-frame-failed')}</p></Show><Show when={compatible()&&ready()}><p class="meta" role="status">{t('privacy-frame-compatible')}</p></Show><ReaderZoomBox><iframe ref={frame} class="reader__frame" classList={{'reader__frame--pending':!presented()}} title={props.title} aria-hidden={!presented()} inert={!presented()} tabIndex={presented()?undefined:-1} sandbox="allow-scripts allow-popups allow-popups-to-escape-sandbox" referrerPolicy="no-referrer" onLoad={loaded} src={source()} /><Show when={!presented()&&!failed()}><div class="reader__frame-loading" data-testid="reader-frame-loading" aria-busy="true">{props.loadingFallback??<MessageLoading variant="skeleton" delayMs={0}/>}</div></Show></ReaderZoomBox></>;
}

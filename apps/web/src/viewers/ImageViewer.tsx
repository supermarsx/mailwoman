import {convertImageData} from './imageConversion.ts';
import {ViewerToolbar} from './ViewerToolbar.tsx';
import {previewToolEnabled} from './filePreviewPreferences.ts';
import {createMemo,createResource,createSignal,onCleanup,onMount,Show,type JSX} from 'solid-js';
import type {ViewerProps} from '../contracts/viewer.ts';
import {blobUrlToDataUrl,mediaFrameDoc,SANDBOX_TOKENS} from './sandbox.ts';
import {type IconName} from '../components/Icon.tsx';
import {mailFeature} from '../state/mailFeatures.ts';
import {loadCatalog,t} from '../i18n/index.ts';
import './image-viewer.css';
type Size={width:number;height:number};
type Mode='fit'|'width'|'actual'|'custom';
export function imageScale(size:Size,viewport:Size,rotation:number,mode:Mode,zoom:number):number{
 const turned=Math.abs(rotation%180)===90,w=turned?size.height:size.width,h=turned?size.width:size.height;
 if(mode==='actual')return 1;
 if(mode==='custom')return Math.max(.05,Math.min(8,zoom));
 if(!w||!h||!viewport.width||!viewport.height)return 1;
 return mode==='width'?Math.min(1,viewport.width/w):Math.min(1,viewport.width/w,viewport.height/h);
}
/** Image bytes stay inside an opaque, script-free frame; controls belong to the app. */
export function imageMime(mime:string,name:string):string{if(mime.startsWith('image/'))return mime;const ext=name.split('.').pop()?.toLowerCase()??'';return ({jpg:'image/jpeg',jpeg:'image/jpeg',jfif:'image/jpeg',svg:'image/svg+xml',tif:'image/tiff',tiff:'image/tiff',ico:'image/x-icon',cur:'image/x-icon'} as Record<string,string>)[ext]??`image/${ext}`;}
export function ImageViewer(props:ViewerProps):JSX.Element{
 void loadCatalog('image-viewer');
 let root:HTMLDivElement|undefined,viewport:HTMLDivElement|undefined;
 const [bounds,setBounds]=createSignal<Size>({width:0,height:0});
 const [mode,setMode]=createSignal<Mode>('fit'),[zoom,setZoom]=createSignal(1),[rotation,setRotation]=createSignal(0),[flipH,setFlipH]=createSignal(false),[flipV,setFlipV]=createSignal(false);
 const [status,setStatus]=createSignal(''),[copying,setCopying]=createSignal(false),[fullscreen,setFullscreen]=createSignal(false);
 const [grayscale,setGrayscale]=createSignal(false),[inverted,setInverted]=createSignal(false);
 const colorFilter=()=>`saturate(${grayscale()?0:1}) invert(${inverted()?1:0})`;
 const [converted,setConverted]=createSignal('');
 const [image,setImage]=createSignal<HTMLImageElement>(),[size,setSize]=createSignal<Size>({width:0,height:0});
 let generation=0;
 function reset():void{setMode('fit');setZoom(1);setRotation(0);setFlipH(false);setFlipV(false);setGrayscale(false);setInverted(false);if(viewport){viewport.scrollTop=0;viewport.scrollLeft=0;}}
 const [source]=createResource(()=>({url:props.blobUrl,mime:props.mime}),async({url,mime})=>{
  const captured=++generation;
  setImage(undefined);setConverted('');setSize({width:0,height:0});reset();setStatus('');
  const data=await blobUrlToDataUrl(url,imageMime(mime,props.name));
  if(captured!==generation)return;
  // An image decoding context never executes SVG scripts or exposes its DOM.
  const decoded=new Image();
  decoded.onload=()=>{if(captured===generation){setImage(decoded);setSize({width:decoded.naturalWidth,height:decoded.naturalHeight});}};
  decoded.onerror=async()=>{
   if(captured!==generation)return;setStatus(t('common-loading'));
   try{const png=await convertImageData(data);if(captured!==generation)return;const converted=new Image();converted.onload=()=>{if(captured===generation){setImage(converted);setSize({width:converted.naturalWidth,height:converted.naturalHeight});setConverted(mediaFrameDoc('image',png));setStatus('');}};converted.onerror=()=>{if(captured===generation)setStatus(t('preview-image-unsupported'));};converted.src=png;}
   catch{if(captured===generation)setStatus(t('preview-image-unsupported'));}
  };
  decoded.src=data;
  return mediaFrameDoc('image',data);
 });
 onCleanup(()=>{generation++;const current=image();if(current){current.onload=null;current.onerror=null;}});
 const scale=createMemo(()=>imageScale(size(),bounds(),rotation(),mode(),zoom()));
 const turned=()=>Math.abs(rotation()%180)===90;
 const width=()=>size().width||bounds().width,height=()=>size().height||bounds().height;
 const canvasWidth=()=>Math.max(bounds().width,(turned()?height():width())*scale());
 const canvasHeight=()=>Math.max(bounds().height,(turned()?width():height())*scale());
 function changeZoom(factor:number):void{setZoom(Math.max(.05,Math.min(8,scale()*factor)));setMode('custom');}
 const canCopy=()=>previewToolEnabled('image.copy')&&!!image()&&mailFeature('allowDownload')&&typeof ClipboardItem!=='undefined'&&typeof navigator.clipboard?.write==='function'&&width()*height()<=64_000_000&&Math.max(width(),height())<=16384;
 async function copy():Promise<void>{
  const decoded=image();if(!decoded||!canCopy()||copying())return;
  const captured=generation,angle=rotation(),horizontal=flipH(),vertical=flipV(),swapped=turned();
  const canvas=document.createElement('canvas');canvas.width=swapped?decoded.naturalHeight:decoded.naturalWidth;canvas.height=swapped?decoded.naturalWidth:decoded.naturalHeight;
  const context=canvas.getContext('2d');if(!context){setStatus(t('image-viewer-copy-failed'));return;}
  context.filter=colorFilter();context.translate(canvas.width/2,canvas.height/2);context.rotate(angle*Math.PI/180);context.scale(horizontal?-1:1,vertical?-1:1);context.drawImage(decoded,-decoded.naturalWidth/2,-decoded.naturalHeight/2);
  setCopying(true);setStatus('');
  try{
   const png=new Promise<Blob>((resolve,reject)=>canvas.toBlob(blob=>blob?resolve(blob):reject(new Error('Image conversion failed')),'image/png'));
   await navigator.clipboard.write([new ClipboardItem({'image/png':png})]);
   if(captured===generation)setStatus(t('image-viewer-copied'));
  }catch{if(captured===generation)setStatus(t('image-viewer-copy-failed'));}
  finally{setCopying(false);canvas.width=0;canvas.height=0;}
 }
 async function toggleFullscreen():Promise<void>{try{const doc=root?.ownerDocument;if(doc&&doc.fullscreenElement===root)await doc.exitFullscreen();else await root?.requestFullscreen();}catch{setStatus(t('image-viewer-fullscreen-failed'));}}
 onMount(()=>{
  if(!viewport||!root)return;
  const measure=()=>{if(viewport)setBounds({width:viewport.clientWidth,height:viewport.clientHeight});};measure();
  if(typeof ResizeObserver!=='undefined'){const observer=new ResizeObserver(measure);observer.observe(viewport);onCleanup(()=>observer.disconnect());}
  const wheel=(event:WheelEvent)=>{if((event.ctrlKey||event.metaKey)&&previewToolEnabled(event.deltaY<0?'image.zoom-in':'image.zoom-out')){event.preventDefault();changeZoom(event.deltaY<0?1.2:1/1.2);}};
  viewport.addEventListener('wheel',wheel,{passive:false});onCleanup(()=>viewport?.removeEventListener('wheel',wheel));
  const doc=root.ownerDocument,changed=()=>setFullscreen(doc.fullscreenElement===root);doc.addEventListener('fullscreenchange',changed);onCleanup(()=>doc.removeEventListener('fullscreenchange',changed));
 });
 let pan:{id:number;x:number;y:number;left:number;top:number}|undefined;
 const controls:Array<{id:string;icon:IconName;run:()=>void;pressed?:()=>boolean;disabled?:()=>boolean}>=[
  {id:'zoom-out',icon:'minus',run:()=>changeZoom(1/1.25),disabled:()=>scale()<=.05},
  {id:'zoom-in',icon:'plus',run:()=>changeZoom(1.25),disabled:()=>scale()>=8},
  {id:'fit',icon:'imageFit',run:()=>setMode('fit'),pressed:()=>mode()==='fit'},
  {id:'width',icon:'imageWidth',run:()=>setMode('width'),pressed:()=>mode()==='width'},
  {id:'actual',icon:'imageActual',run:()=>setMode('actual'),pressed:()=>mode()==='actual'},
  {id:'rotate-left',icon:'rotateCcw',run:()=>setRotation(value=>(value+270)%360)},
  {id:'rotate-right',icon:'rotateCw',run:()=>setRotation(value=>(value+90)%360)},
  {id:'flip-horizontal',icon:'flipH',run:()=>setFlipH(value=>!value),pressed:flipH},
  {id:'flip-vertical',icon:'flipV',run:()=>setFlipV(value=>!value),pressed:flipV},
  {id:'grayscale',icon:'grayscale',run:()=>setGrayscale(value=>!value),pressed:grayscale},
  {id:'invert',icon:'invert',run:()=>setInverted(value=>!value),pressed:inverted},
  {id:'reset',icon:'restore',run:reset},
 ];
 return <div ref={root} class="image-preview">
  <div class="image-preview__toolbar" role="group" aria-label={t('image-viewer-controls')}>
   <ViewerToolbar scope={props.blobUrl} actions={[...controls.map(control=>({id:`image.${control.id}`,label:t(`image-viewer-${control.id}`),icon:control.icon,pressed:control.pressed?.(),disabled:control.disabled?.()||!image(),run:control.run})),...(mailFeature('allowDownload')?[{id:'image.copy',label:t('image-viewer-copy'),icon:'copy' as const,disabled:!canCopy()||copying(),run:copy}]:[]),...(typeof document.documentElement.requestFullscreen==='function'?[{id:'image.fullscreen',label:t('image-viewer-fullscreen'),icon:'fullscreen' as const,pressed:fullscreen(),run:toggleFullscreen}]:[])]}/>
   <output aria-label={t('image-viewer-zoom')}>{Math.round(scale()*100)}%</output>
  </div>
  <div ref={viewport} class="image-preview__viewport" tabindex="0" aria-label={props.name}
   onKeyDown={event=>{if(event.ctrlKey||event.metaKey||event.altKey)return;const key=event.key;const tool=key==='+'||key==='='?'image.zoom-in':key==='-'?'image.zoom-out':key==='0'?'image.fit':key==='1'?'image.actual':key.toLowerCase()==='r'?'image.rotate-right':undefined;if(tool&&!previewToolEnabled(tool))return;if(key==='+'||key==='=')changeZoom(1.25);else if(key==='-')changeZoom(1/1.25);else if(key==='0')setMode('fit');else if(key==='1')setMode('actual');else if(key.toLowerCase()==='r')setRotation(value=>(value+90)%360);else return;event.preventDefault();}}
   onPointerDown={event=>{if(event.pointerType!=='mouse'||event.button!==0)return;const node=event.currentTarget;pan={id:event.pointerId,x:event.clientX,y:event.clientY,left:node.scrollLeft,top:node.scrollTop};node.setPointerCapture?.(event.pointerId);}}
   onPointerMove={event=>{if(pan?.id!==event.pointerId)return;event.currentTarget.scrollLeft=pan.left+pan.x-event.clientX;event.currentTarget.scrollTop=pan.top+pan.y-event.clientY;}}
   onPointerUp={()=>{pan=undefined;}} onPointerCancel={()=>{pan=undefined;}} onLostPointerCapture={()=>{pan=undefined;}}>
   <Show when={!source.error} fallback={<p role="alert">{t('image-viewer-failed')}</p>}>
    <Show when={source()} fallback={<p class="mw-viewer__loading">{t('common-viewer-loading-image')}</p>}>{doc=><div class="image-preview__canvas" style={{width:`${canvasWidth()}px`,height:`${canvasHeight()}px`}}><iframe class="mw-viewer__frame mw-viewer__frame--image" title={props.name} sandbox={SANDBOX_TOKENS} scrolling="no" srcdoc={converted()||doc()} style={{filter:colorFilter(),width:`${width()*scale()}px`,height:`${height()*scale()}px`,transform:`translate(-50%, -50%) rotate(${rotation()}deg) scale(${flipH()?-1:1}, ${flipV()?-1:1})`}}/></div>}</Show>
   </Show>
  </div>
  <Show when={status()}><p class="image-preview__status" role="status">{status()}</p></Show>
 </div>;
}
export default ImageViewer;

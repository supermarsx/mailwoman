import {Show,createEffect,createSignal,createUniqueId,onCleanup,onMount,type JSX} from 'solid-js';
import {Portal} from 'solid-js/web';
import {withBase} from '../../../api/basePath.ts';
import type {BrandKind} from '../../../api/admin/branding.ts';
import {Icon} from '../../../components/Icon.tsx';
import {t} from '../../../i18n';
import {tooltip} from '../../../components/tooltip.css.ts';
import * as css from './currentValue.css.ts';

/** One image a branding slot shows: an uploaded asset, or the built-in default. */
export interface BrandImageView {url?:string|undefined;custom:boolean;draft?:boolean}

/** Bundled defaults served from the web root; the logo and background have none. */
const DEFAULTS:Partial<Record<BrandKind,{path:string;type:string}>>={
 icon_master:{path:'/favicon.svg',type:'SVG'},
 pwa_192:{path:'/icons/icon-192.png',type:'PNG'},
 pwa_512:{path:'/icons/icon-512.png',type:'PNG'},
 pwa_maskable:{path:'/icons/icon-maskable-512.png',type:'PNG'},
};
export function defaultImage(kind:BrandKind):BrandImageView{const d=DEFAULTS[kind];return {url:d?withBase(d.path):undefined,custom:false};}
function imageType(kind:BrandKind,image:BrandImageView):string|undefined{
 if(!image.url)return undefined;
 // Every upload is re-encoded to PNG by the server before it is stored.
 return image.custom?'PNG':DEFAULTS[kind]?.type;
}

/** Natural pixel size, read off-DOM so it is known before the tooltip opens. */
function useImageSize(url:()=>string|undefined):()=>string|undefined{
 const [size,setSize]=createSignal<string>();
 createEffect(()=>{
  const src=url();setSize(undefined);if(!src)return;
  const image=new Image();let live=true;
  image.onload=()=>{if(live&&image.naturalWidth)setSize(t('admin-branding-current-pixels',{width:image.naturalWidth,height:image.naturalHeight}));};
  image.onerror=()=>{if(live)setSize(t('admin-branding-current-unavailable'));};
  image.src=src;onCleanup(()=>{live=false;image.onload=null;image.onerror=null;});
 });
 return size;
}

function Checkerboard(props:{class:string;url?:string|undefined;children?:JSX.Element}):JSX.Element{
 const pattern=`brand-checker-${createUniqueId()}`;
 return <span class={`${css.checker} ${props.class}`}><svg class={css.checkerPattern} aria-hidden="true"><defs><pattern id={pattern} width="8" height="8" patternUnits="userSpaceOnUse"><rect width="4" height="4"/><rect x="4" y="4" width="4" height="4"/></pattern></defs><rect width="100%" height="100%" fill={`url(#${pattern})`}/></svg><Show when={props.url} fallback={props.children}>{url=><img src={url()} alt="" class={css.checkerImage} decoding="async"/>}</Show></span>;
}

/**
 * A focusable trigger with a themed, non-modal tooltip. Hover and keyboard focus
 * open it; a tap toggles it on touch screens; Escape, blur or a tap elsewhere
 * close it. The bubble stays in the DOM (hidden) so `aria-describedby` always
 * resolves. The global TooltipLayer is opted out so only this tooltip shows.
 */
export function CurrentTip(props:{label:string;class:string;trigger:JSX.Element;children:JSX.Element}):JSX.Element{
 const id=`brand-current-${createUniqueId()}`;
 const [open,setOpen]=createSignal(false);
 let button:HTMLButtonElement|undefined;let bubble:HTMLDivElement|undefined;
 let hideTimer:ReturnType<typeof setTimeout>|undefined;let touchWasOpen:boolean|undefined;let frame=0;
 const clearHide=()=>clearTimeout(hideTimer);
 const scheduleHide=()=>{clearHide();hideTimer=setTimeout(()=>{if(document.activeElement!==button)setOpen(false);},120);};
 const place=()=>{
  if(!open()||!button||!bubble)return;
  if(!button.isConnected){setOpen(false);return;}
  const anchor=button.getBoundingClientRect();const box=bubble.getBoundingClientRect();const view=window.visualViewport;
  const left=view?.offsetLeft??0;const top=view?.offsetTop??0;const width=view?.width??window.innerWidth;const height=view?.height??window.innerHeight;
  const x=Math.max(left+8,Math.min(anchor.left,left+width-box.width-8));
  const below=anchor.bottom+8;const y=below+box.height<=top+height-8?below:Math.max(top+8,anchor.top-box.height-8);
  // CSSOM writes, not a style attribute, so the strict style-src still holds.
  bubble.style.left=`${Math.round(x)}px`;bubble.style.top=`${Math.round(y)}px`;
 };
 const schedulePlace=()=>{cancelAnimationFrame(frame);frame=requestAnimationFrame(place);};
 createEffect(()=>{if(open())schedulePlace();});
 onMount(()=>{
  const outside=(event:PointerEvent)=>{const target=event.target as Node|null;if(open()&&target&&!button?.contains(target)&&!bubble?.contains(target))setOpen(false);};
  document.addEventListener('pointerdown',outside,true);document.addEventListener('scroll',schedulePlace,true);window.addEventListener('resize',schedulePlace);
  onCleanup(()=>{clearHide();cancelAnimationFrame(frame);document.removeEventListener('pointerdown',outside,true);document.removeEventListener('scroll',schedulePlace,true);window.removeEventListener('resize',schedulePlace);});
 });
 return <><button ref={button} type="button" class={props.class} aria-label={props.label} aria-describedby={id} data-tooltip="off" data-brand-current
  onPointerEnter={e=>{if(e.pointerType!=='touch'){clearHide();setOpen(true);}}}
  onPointerLeave={e=>{if(e.pointerType!=='touch')scheduleHide();}}
  onPointerDown={e=>{touchWasOpen=e.pointerType==='touch'?open():undefined;}}
  onFocus={()=>setOpen(true)} onBlur={()=>setOpen(false)}
  onClick={()=>{if(touchWasOpen!==undefined)setOpen(!touchWasOpen);else setOpen(true);touchWasOpen=undefined;}}
  onKeyDown={e=>{if(e.key==='Escape'&&open()){e.preventDefault();e.stopPropagation();setOpen(false);}}}>{props.trigger}</button>
  <Portal><div ref={bubble} id={id} role="tooltip" class={`${tooltip} ${css.bubble}`} hidden={!open()} onPointerEnter={clearHide} onPointerLeave={scheduleHide}>{props.children}</div></Portal></>;
}

const scopeNote=()=><p class={css.scope}>{t('admin-branding-current-scope')}</p>;

function ImageFacts(props:{kind:BrandKind;heading:string;image:BrandImageView}):JSX.Element{
 const size=useImageSize(()=>props.image.url);
 return <figure class={css.figure}><figcaption class={css.caption}>{props.heading}</figcaption>
  <Checkerboard class={css.preview} url={props.image.url}><span class={css.emptyMark}>{t('admin-branding-current-no-image')}</span></Checkerboard>
  <dl class={css.facts}>
   <Show when={props.image.url} fallback={<><dt>{t('admin-branding-current-shows')}</dt><dd>{t(`admin-branding-current-default-${props.kind==='background'?'background':'logo'}`)}</dd></>}>
    <dt>{t('admin-branding-current-size')}</dt><dd>{size()??t('admin-branding-current-measuring')}</dd>
    <dt>{t('admin-branding-current-type')}</dt><dd>{imageType(props.kind,props.image)}</dd>
   </Show>
   <dt>{t('admin-branding-current-origin')}</dt><dd>{t(props.image.custom?'admin-branding-current-custom':'admin-branding-current-default')}</dd>
   <dt>{t('admin-branding-current-source')}</dt><dd>{t(props.image.draft?'admin-branding-current-draft':'admin-branding-current-published')}</dd>
  </dl></figure>;
}

/** Inline thumbnail of the slot's published image (and the pending one), with details on hover, focus or tap. */
export function BrandImageCurrent(props:{kind:BrandKind;field:string;current:BrandImageView;next?:BrandImageView|undefined}):JSX.Element{
 const state=()=>t(props.current.custom?'admin-branding-current-custom':'admin-branding-current-default');
 return <div class={css.row}>
  <CurrentTip class={css.trigger} label={t(props.next?'admin-branding-current-label-pending':'admin-branding-current-label',{field:props.field,state:state()})}
   trigger={<><Checkerboard class={css.thumb} url={props.current.url}/><span class={css.triggerText}>{state()}</span>
    <Show when={props.next}>{next=><><span class={css.arrow} aria-hidden="true">→</span><Checkerboard class={css.thumb} url={next().url}/><span class={css.triggerText}>{t('admin-branding-current-new')}</span></>}</Show></>}>
   <div class={css.columns}>
    <ImageFacts kind={props.kind} heading={t('admin-branding-current-heading')} image={props.current}/>
    <Show when={props.next}>{next=><ImageFacts kind={props.kind} heading={t('admin-branding-current-new')} image={next()}/>}</Show>
   </div>
   {scopeNote()}
  </CurrentTip>
 </div>;
}

/** ⓘ beside a text field: the published value, and the unsaved edit when there is one. */
export function BrandTextCurrent(props:{field:string;published:string|null;draft:string|null}):JSX.Element{
 const pending=()=>(props.draft??'')!==(props.published??'');
 const shown=(value:string|null)=>value?.trim()?value:t('admin-branding-current-empty');
 return <CurrentTip class={css.infoTrigger} label={t('admin-branding-current-text-label',{field:props.field})} trigger={<Icon name="info"/>}>
  <dl class={css.facts}>
   <dt>{t('admin-branding-current-heading')}</dt><dd>{shown(props.published)}</dd>
   <dt>{t('admin-branding-current-source')}</dt><dd>{t('admin-branding-current-published')}</dd>
   <Show when={pending()}><dt>{t('admin-branding-current-new')}</dt><dd>{shown(props.draft)}</dd></Show>
  </dl>
  {scopeNote()}
 </CurrentTip>;
}

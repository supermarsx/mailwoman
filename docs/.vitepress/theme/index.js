// The default theme without its bundled Inter font: the manual uses the app's
// own self-hosted faces (Jost, JetBrains Mono), declared in style.css.
import DefaultTheme from 'vitepress/theme-without-fonts';
import {h,nextTick,onMounted,watch} from 'vue';
import {useData,useRoute,withBase} from 'vitepress';
import './style.css';

// Moved pages: a stub with `redirect: /new/path` in its front matter sends the
// browser on. Readers without script get the stub's own "moved" link and a
// <meta refresh> (config.mjs transformHead).
const Redirect={
 setup(){
  const {frontmatter}=useData();
  const go=()=>{
   const to=frontmatter.value.redirect;
   if(typeof to==='string'&&to.startsWith('/')&&!to.startsWith('//'))location.replace(withBase(to));
  };
  onMounted(go);
  watch(frontmatter,go);
  return ()=>null;
 },
};

// Screenshots open at full size on click (or Enter): a plain overlay, no
// dependency. Esc, a click or the close button dismisses it and focus returns.
const zoomable='.vp-doc img.mw-shot, .vp-doc img.mw-crop';
function openFull(img){
 const opener=document.activeElement;
 const box=document.createElement('div');
 box.className='mw-lightbox';box.setAttribute('role','dialog');box.setAttribute('aria-modal','true');
 box.setAttribute('aria-label',img.alt||'Screenshot at full size');
 const close=document.createElement('button');
 close.type='button';close.className='mw-lightbox-close';close.textContent='Close';
 const full=document.createElement('img');
 full.src=img.currentSrc||img.src;full.alt=img.alt;
 const done=()=>{document.removeEventListener('keydown',key,true);box.remove();document.body.classList.remove('mw-lightbox-open');opener?.focus?.();};
 const key=e=>{if(e.key==='Escape'){e.preventDefault();done();}else if(e.key==='Tab'){e.preventDefault();close.focus();}};
 box.addEventListener('click',done);close.addEventListener('click',done);
 document.addEventListener('keydown',key,true);
 box.append(close,full);document.body.append(box);document.body.classList.add('mw-lightbox-open');close.focus();
}
const Zoom={
 setup(){
  const route=useRoute();
  const mark=()=>nextTick(()=>document.querySelectorAll(zoomable).forEach(img=>{
   img.tabIndex=0;img.setAttribute('role','button');
   img.title=img.title||'Open at full size';
  }));
  const click=e=>{const img=e.target?.closest?.(zoomable);if(img)openFull(img);};
  const press=e=>{if((e.key==='Enter'||e.key===' ')&&e.target?.matches?.(zoomable)){e.preventDefault();openFull(e.target);}};
  onMounted(()=>{mark();document.addEventListener('click',click);document.addEventListener('keydown',press);});
  watch(()=>route.path,mark);
  return ()=>null;
 },
};

export default {
 extends:DefaultTheme,
 Layout:()=>h(DefaultTheme.Layout,null,{'layout-bottom':()=>[h(Redirect),h(Zoom)]}),
};

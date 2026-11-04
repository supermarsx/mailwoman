import font0 from '@fontsource-variable/manrope/files/manrope-cyrillic-ext-wght-normal.woff2?url';
import font1 from '@fontsource-variable/manrope/files/manrope-cyrillic-wght-normal.woff2?url';
import font2 from '@fontsource-variable/manrope/files/manrope-greek-wght-normal.woff2?url';
import font3 from '@fontsource-variable/manrope/files/manrope-vietnamese-wght-normal.woff2?url';
import font4 from '@fontsource-variable/manrope/files/manrope-latin-ext-wght-normal.woff2?url';
import font5 from '@fontsource-variable/manrope/files/manrope-latin-wght-normal.woff2?url';

const subsets=[
 {url:font0,unicodeRange:'U+0460-052F,U+1C80-1C8A,U+20B4,U+2DE0-2DFF,U+A640-A69F,U+FE2E-FE2F'},
 {url:font1,unicodeRange:'U+0301,U+0400-045F,U+0490-0491,U+04B0-04B1,U+2116'},
 {url:font2,unicodeRange:'U+0370-0377,U+037A-037F,U+0384-038A,U+038C,U+038E-03A1,U+03A3-03FF'},
 {url:font3,unicodeRange:'U+0102-0103,U+0110-0111,U+0128-0129,U+0168-0169,U+01A0-01A1,U+01AF-01B0,U+0300-0301,U+0303-0304,U+0308-0309,U+0323,U+0329,U+1EA0-1EF9,U+20AB'},
 {url:font4,unicodeRange:'U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF'},
 {url:font5,unicodeRange:'U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD'}
];
const buffers=new Map<string,Promise<ArrayBuffer>>();
const installed=new WeakMap<Document,Promise<void>>();
const manrope=(family:string)=>family.split(',').some(value=>['manrope','manrope variable'].includes(value.trim().replace(/^["']|["']$/g,'').toLowerCase()));
function used(root:HTMLElement):boolean {
 const doc=root.ownerDocument,view=doc.defaultView;
 if(!view)return false;
 const matches=(element:Element)=>manrope(view.getComputedStyle(element).fontFamily);
 if(matches(root))return true;
 for(const element of root.querySelectorAll<HTMLElement>('[style],font[face]'))if((manrope(element.style.fontFamily)||manrope(element.getAttribute('face')??''))&&matches(element))return true;
 for(const sheet of doc.adoptedStyleSheets??[]){
  let rules:CSSRuleList;try{rules=sheet.cssRules;}catch{continue;}
  for(const rule of Array.from(rules)){
   const style=rule as CSSStyleRule;if(!style.style||!manrope(style.style.fontFamily))continue;
   let target:Element|null;try{target=root.matches(style.selectorText)?root:root.querySelector(style.selectorText);}catch{continue;}
   if(target&&matches(target))return true;
  }
 }
 return false;
}
/** Native editor preview only. Font bytes stay outside captured/sent HTML and
 * child CSP remains unchanged: only the parent fetches fixed bundled assets. */
export function ensureManropePreview(root:HTMLElement):Promise<void> {
 const doc=root.ownerDocument,view=doc.defaultView as (Window&{FontFace:typeof FontFace})|null;
 if(!view?.FontFace||!doc.fonts)return Promise.resolve();
 const existing=installed.get(doc);if(existing)return existing;
 if(!used(root))return Promise.resolve();
 const work=Promise.all(subsets.map(async subset=>{
  const url=new URL(subset.url,doc.baseURI);if(url.origin!==new URL(doc.baseURI).origin)throw new Error('Font asset must be same origin');
  let bytes=buffers.get(url.href);
  // This is an imported, same-origin static asset, not a mailbox API. The API
  // router deliberately rejects /assets while a linked mailbox is selected.
  // eslint-disable-next-line mw/no-global-fetch -- Fixed bundled font GET; same-origin checked above, redirects refused.
  if(!bytes){bytes=fetch(url.href,{credentials:'same-origin',redirect:'error'}).then(response=>{if(!response.ok)throw new Error('Font unavailable');return response.arrayBuffer();}).catch(error=>{buffers.delete(url.href);throw error;});buffers.set(url.href,bytes);}
  const data=await bytes;
  for(const family of ['Manrope Variable','Manrope']){
   const face=await new view.FontFace(family,data,{weight:'200 800',style:'normal',display:'swap',unicodeRange:subset.unicodeRange}).load();
   if(root.isConnected&&root.ownerDocument===doc)doc.fonts.add(face);
  }
 })).then(()=>undefined).catch(()=>{installed.delete(doc);});
 installed.set(doc,work);return work;
}

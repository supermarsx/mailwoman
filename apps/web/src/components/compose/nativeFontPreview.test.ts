import {afterEach,expect,it,vi} from 'vitest';
import {ensureManropePreview} from './nativeFontPreview.ts';
afterEach(()=>{document.body.replaceChildren();vi.unstubAllGlobals();});
function nativeDocument(family:string){const frame=document.createElement('iframe');document.body.append(frame);const doc=frame.contentDocument!,root=doc.createElement('div');root.style.fontFamily=family;root.textContent='Font preview';doc.body.append(root);const faces:{family:string;unicodeRange:string}[]=[];class Face{family:string;unicodeRange:string;constructor(name:string,_data:ArrayBuffer,options:FontFaceDescriptors){this.family=name;this.unicodeRange=options.unicodeRange??'';}async load(){return this;}}
 Object.defineProperty(frame.contentWindow!,'FontFace',{value:Face,configurable:true});Object.defineProperty(doc,'fonts',{value:{add:(face:{family:string;unicodeRange:string})=>faces.push(face)},configurable:true});return {doc,root,faces};}
it('does not fetch preview fonts until Manrope is actually used and installs all subsets without changing markup',async()=>{
 const fetcher=vi.fn(async()=>new Response(new Uint8Array([1,2,3])));vi.stubGlobal('fetch',fetcher);const first=nativeDocument('Arial');const original=first.root.outerHTML;
 await ensureManropePreview(first.root);expect(fetcher).not.toHaveBeenCalled();expect(first.faces).toHaveLength(0);
 first.root.style.fontFamily='"Manrope Variable", Arial, sans-serif';const authored=first.root.outerHTML;await ensureManropePreview(first.root);expect(fetcher).toHaveBeenCalledTimes(6);expect(first.faces).toHaveLength(12);expect(new Set(first.faces.map(face=>face.family))).toEqual(new Set(['Manrope Variable','Manrope']));expect(first.faces.every(face=>face.unicodeRange.startsWith('U+'))).toBe(true);expect(first.root.outerHTML).toBe(authored);expect(original).not.toContain('@font-face');
 const second=nativeDocument('Manrope, Arial');await ensureManropePreview(second.root);expect(fetcher).toHaveBeenCalledTimes(6);expect(second.faces).toHaveLength(12);
 for(const [url,options] of fetcher.mock.calls as unknown as [string,RequestInit][]) {expect(new URL(url).origin).toBe(location.origin);expect(options).toEqual({credentials:'same-origin',redirect:'error'});}
});

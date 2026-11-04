import {afterEach,beforeAll,expect,it,vi} from 'vitest';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {initSync} from '../../wasm/mw-sanitize/mw_sanitize.js';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {NativeHtmlVisualEditor,type NativeHtmlVisualApi} from './NativeHtmlVisualEditor.tsx';

afterEach(()=>{cleanup();vi.restoreAllMocks();});
beforeAll(()=>initSync({module:readFileSync(resolve(process.cwd(),'src/wasm/mw-sanitize/mw_sanitize_bg.wasm'))}));
function mount(html:string,upload?:(files:File[])=>Promise<{cid:string;alt:string}[]>,reply=false){
 let api!:NativeHtmlVisualApi;
 const change=vi.fn();
 render(()=><NativeHtmlVisualEditor html={html} revision={0} reply={reply} owned={()=>new Set(['logo'])} {...(upload?{upload}:{})} onInput={change} onReady={value=>api=value}/>);
 const frame=screen.getByTitle('Visual editor') as HTMLIFrameElement;
 const doc=frame.contentDocument!,view=frame.contentWindow as Window&{CSSStyleSheet:typeof CSSStyleSheet};
 if(!view.CSSStyleSheet.prototype.replaceSync)Object.defineProperty(view.CSSStyleSheet.prototype,'replaceSync',{value:()=>{},configurable:true});
 doc.documentElement.setAttribute('data-mw-native-document','true');fireEvent.load(frame);
 return {api,frame,doc,change};
}
it('uses all shared ribbon groups and preserves CID background images alongside new shading and undo',async()=>{
 const {api,doc}=mount('<table width="480"><tbody><tr><td style="background:url(cid:logo) #fff">Native layout</td></tr></tbody></table>');
 for(const id of ['clipboard','font','paragraph','editing','view','insert','table-insert','table','table-shading'])expect(screen.getByTestId(`rt-group-${id}`)).toBeInTheDocument();
 const cell=doc.querySelector('td')!;cell.style.backgroundColor='rgb(255, 0, 0)';fireEvent.input(cell);
 const captured=await api.capture();
 const parsed=new DOMParser().parseFromString(captured,'text/html');
 expect(parsed.querySelector('table')?.getAttribute('width')).toBe('480');
 expect(parsed.querySelector('td')?.style.backgroundImage).toContain('cid:logo');
 expect(parsed.querySelector('td')?.style.backgroundColor).toBe('rgb(255, 0, 0)');
 expect(api.historyDepth()).toEqual({undo:1,redo:0});
 api.undo();expect(api.historyDepth()).toEqual({undo:0,redo:1});
 expect((await api.capture())).not.toContain('255, 0, 0');
 api.redo();expect((await api.capture())).toContain('255, 0, 0');
});
it('strips editor-only caret markers from captured HTML without dropping typed text',async()=>{
 const {api,doc}=mount('<p>Text</p>');
 const span=doc.createElement('span');span.dataset['mwNativeCaret']='true';span.style.fontSize='18pt';span.textContent='\u200bTyped';doc.querySelector('p')!.append(span);fireEvent.input(span);
 const captured=await api.capture();expect(captured).toContain('Typed');expect(captured).toContain('18pt');expect(captured).not.toContain('\u200b');expect(captured).not.toContain('data-mw-native-caret');
});
it('focuses and bookmarks the blank reply answer without sending its view-only spacing',async()=>{
 const {api,doc}=mount('<p></p><p>Alice wrote:</p><blockquote><table cellpadding="7"><tr><td><img src="cid:logo">Original</td></tr></table></blockquote>',undefined,true);
 api.focus();const answer=doc.querySelector('p')!;
 expect(answer).toHaveAttribute('data-mw-native-answer','true');
 expect(doc.getSelection()!.getRangeAt(0).startContainer).toBe(answer);
 expect(api.bookmark()).toEqual({start:0,end:0,answer:true});
 answer.textContent='My response';fireEvent.input(answer);
 const captured=await api.capture();expect(captured).toContain('<p>My response</p>');expect(captured).toContain('cellpadding="7"');expect(captured).toContain('cid:logo');expect(captured).not.toMatch(/data-mw-native-answer|min-block-size|6em/);
 const original=doc.querySelector('td')!.lastChild!,range=doc.createRange();range.setStart(original,2);range.collapse(true);doc.getSelection()!.removeAllRanges();doc.getSelection()!.addRange(range);fireEvent(doc,new Event('selectionchange'));
 api.focus();expect(doc.getSelection()!.getRangeAt(0).startContainer).toBe(original);expect(doc.getSelection()!.getRangeAt(0).startOffset).toBe(2);
});
it('refreshes the view-only app surface without rewriting authored backgrounds or outgoing HTML',async()=>{
 const {api,frame,doc,change}=mount('<table style="background-color:#eeddaa"><tbody><tr><td>Authored surface</td></tr></tbody></table>');
 const view=frame.contentWindow as Window&{CSSStyleSheet:typeof CSSStyleSheet};
 const replace=vi.spyOn(view.CSSStyleSheet.prototype,'replaceSync');
 frame.style.backgroundColor='rgb(20, 30, 40)';frame.style.color='rgb(230, 240, 250)';
 document.documentElement.setAttribute('data-native-theme-test','dark');
 await waitFor(()=>expect(replace.mock.calls.at(-1)?.[0]).toContain('background:rgb(20, 30, 40)'));
 expect(frame).toHaveAttribute('data-surface','theme');expect(change).not.toHaveBeenCalled();
 doc.querySelector('td')!.append(' edited');fireEvent.input(doc.querySelector('td')!);
 const captured=await api.capture();
 expect(captured).toContain('background-color: rgb(238, 221, 170)');
 expect(captured).not.toContain('20, 30, 40');expect(captured).not.toContain('230, 240, 250');
 frame.style.backgroundColor='rgb(250, 245, 240)';document.documentElement.setAttribute('data-native-theme-test','light');
 await waitFor(()=>expect(replace.mock.calls.at(-1)?.[0]).toContain('background:rgb(250, 245, 240)'));
 expect(await api.capture()).toBe(captured);
 document.documentElement.removeAttribute('data-native-theme-test');
});
it('keeps a paragraph selection and its font state after undo and redo at the preceding table boundary',async()=>{
 const {api,doc}=mount('<table><tbody><tr><td>Native signature</td></tr></tbody></table><p>Editable paragraph</p>');
 const paragraph=doc.querySelector('p')!,range=doc.createRange();range.selectNodeContents(paragraph);doc.getSelection()!.removeAllRanges();doc.getSelection()!.addRange(range);fireEvent(doc,new Event('selectionchange'));
 fireEvent.change(screen.getByLabelText('Font size'),{target:{value:'18pt'}});
 expect(api.historyDepth().undo).toBe(1);api.undo();api.redo();
 const selection=doc.getSelection()!;
 expect(selection.toString()).toBe('Editable paragraph');
 expect(selection.getRangeAt(0).startContainer.parentElement?.closest('p')).not.toBeNull();
 expect(screen.getByLabelText('Font size')).toHaveValue('18pt');
 expect((await api.capture())).toContain('18pt');
});
it('does not replace an old selection after an image upload when the message has since changed',async()=>{
 let complete!:(images:{cid:string;alt:string}[])=>void;
 const upload=vi.fn(()=>new Promise<{cid:string;alt:string}[]>(resolve=>{complete=resolve;}));
 const {api,doc}=mount('<p>Keep this message</p>',upload);
 const paragraph=doc.querySelector('p')!,range=doc.createRange();range.setStart(paragraph.firstChild!,0);range.setEnd(paragraph.firstChild!,4);doc.getSelection()!.addRange(range);fireEvent(doc,new Event('selectionchange'));
 fireEvent.change(screen.getByLabelText('Insert an image'),{target:{files:[new File(['png'],'logo.png',{type:'image/png'})]}});
 await waitFor(()=>expect(upload).toHaveBeenCalledOnce());
 paragraph.append(' Later edit');fireEvent.input(paragraph);complete([{cid:'logo',alt:'Image'}]);
 const captured=await api.capture();expect(captured).toContain('Keep this message Later edit');expect(captured).not.toContain('<img');expect(screen.getByRole('status')).toHaveTextContent('The selection changed');
});

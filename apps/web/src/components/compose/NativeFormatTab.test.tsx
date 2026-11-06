import {afterEach,beforeAll,beforeEach,expect,it,vi} from 'vitest';

const state=vi.hoisted(()=>({map:{} as Record<string,{value:unknown;locked?:boolean}>}));
vi.mock('../../api/settings.ts',()=>({settings:()=>state.map,loadSettings:vi.fn(async()=>undefined)}));

import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {initSync} from '../../wasm/mw-sanitize/mw_sanitize.js';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {NativeHtmlVisualEditor,type NativeHtmlVisualApi} from './NativeHtmlVisualEditor.tsx';
import {loadCatalog} from '../../i18n/index.ts';

beforeAll(async()=>{initSync({module:readFileSync(resolve(process.cwd(),'src/wasm/mw-sanitize/mw_sanitize_bg.wasm'))});await loadCatalog('compose-format');await loadCatalog('mail');});
beforeEach(()=>{state.map={};});
afterEach(()=>{cleanup();vi.restoreAllMocks();});
function mount(html:string){
 let api!:NativeHtmlVisualApi;
 render(()=><NativeHtmlVisualEditor html={html} revision={0} owned={()=>new Set()} onInput={()=>undefined} onReady={value=>api=value}/>);
 const frame=screen.getByTitle('Visual editor') as HTMLIFrameElement;
 const doc=frame.contentDocument!,view=frame.contentWindow as Window&{CSSStyleSheet:typeof CSSStyleSheet};
 if(!view.CSSStyleSheet.prototype.replaceSync)Object.defineProperty(view.CSSStyleSheet.prototype,'replaceSync',{value:()=>{},configurable:true});
 doc.documentElement.setAttribute('data-mw-native-document','true');fireEvent.load(frame);
 return {api,doc};
}
function selectAll(doc:Document){
 const range=doc.createRange();range.selectNodeContents(doc.getElementById('mw-native-editor')!);
 doc.getSelection()!.removeAllRanges();doc.getSelection()!.addRange(range);fireEvent(doc,new Event('selectionchange'));
}

it('has the Format groups in order with labelled painter buttons',()=>{
 mount('<p>Hi</p>');
 const ids=Array.from(document.querySelectorAll('[data-ribbon-group]')).map(g=>g.getAttribute('data-testid')).slice(0,6);
 expect(ids).toEqual(['rt-group-clipboard','rt-group-painter','rt-group-font','rt-group-paragraph','rt-group-styles','rt-group-editing']);
 expect(screen.getByTestId('rt-copy-format')).toHaveTextContent('Copy formatting');
 expect(screen.getByTestId('rt-apply-format')).toHaveTextContent('Apply formatting');
});
it('makes a numbered list from the dropdown and sends no editor attributes',async()=>{
 const {api,doc}=mount('<p>One</p><p>Two</p>');
 selectAll(doc);
 fireEvent.click(screen.getByTestId('rt-ordered-types'));
 fireEvent.click(await screen.findByTestId('list-style-paren'));
 await waitFor(()=>expect(doc.querySelector('ol')?.getAttribute('data-mw-list')).toBe('paren'));
 const sent=await api.capture();
 expect(sent).toContain('<span>1) </span>One'.replace(' ','&nbsp;'));
 expect(sent).not.toMatch(/data-mw-list|data-mw-checked|class=/);
 // The editor reopens its own output as the same list type.
 fireEvent.click(screen.getByTestId('rt-ordered-types'));
 fireEvent.click(await screen.findByTestId('list-style-lower-roman'));
 await waitFor(()=>expect(doc.querySelector('ol')?.getAttribute('type')).toBe('i'));
});
it('grows the font with the button and the shortcut, and the shortcut is inert when the feature is off',async()=>{
 const {doc}=mount('<p>Hello</p>');
 selectAll(doc);
 fireEvent.click(screen.getByTestId('rt-font-grow'));
 await waitFor(()=>expect(doc.querySelector<HTMLElement>('#mw-native-editor span')?.style.fontSize).toBe('12pt'));
 doc.dispatchEvent(new (doc.defaultView as unknown as typeof globalThis).KeyboardEvent('keydown',{key:'>',ctrlKey:true,shiftKey:true,bubbles:true,cancelable:true}));
 await waitFor(()=>expect(Array.from(doc.querySelectorAll<HTMLElement>('#mw-native-editor span')).some(s=>s.style.fontSize==='14pt')).toBe(true));
});
it('hides switched-off controls and leaves their shortcuts inert',async()=>{
 for(const id of ['fontSizeStep','indent','clearFormatting','painter','lists'])state.map[`compose.feature.format.${id}`]={value:false};
 const {doc}=mount('<p style="margin-left:40px"><b>Hello</b></p>');
 for(const id of ['rt-font-grow','rt-indent','rt-clear-formatting','rt-copy-format','rt-bullet'])expect(screen.queryByTestId(id),id).toBeNull();
 selectAll(doc);
 doc.dispatchEvent(new (doc.defaultView as unknown as typeof globalThis).KeyboardEvent('keydown',{key:' ',ctrlKey:true,bubbles:true,cancelable:true}));
 expect(doc.querySelector('b')).not.toBeNull();
});
it('copies formatting into the chip and clears it with the menu',async()=>{
 const {doc}=mount('<p><b style="color:#336699">Fancy</b></p>');
 selectAll(doc);
 fireEvent.click(screen.getByTestId('rt-copy-format'));
 await waitFor(()=>expect(screen.getByTestId('rt-format-chip')).toHaveTextContent('bold'));
 fireEvent.click(screen.getByTestId('rt-clear-formatting-options'));
 fireEvent.click(await screen.findByTestId('clear-text'));
 await waitFor(()=>expect(doc.querySelector('b')).toBeNull());
});

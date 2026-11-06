import {beforeEach,describe,expect,it,vi} from 'vitest';

const state=vi.hoisted(()=>({map:{} as Record<string,{value:unknown;locked?:boolean}>,writes:[] as [string,unknown][]}));
vi.mock('../../api/settings.ts',()=>({
 settings:()=>state.map,
 loadSettings:vi.fn(async()=>undefined),
 writeSetting:vi.fn(async(key:string,value:unknown)=>{state.writes.push([key,value]);state.map[key]={value};return {value};}),
 resetSetting:vi.fn(async()=>({})),
 SettingsApiError:class extends Error{},
}));

import {afterEach,beforeAll} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor,within} from '@solidjs/testing-library';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {initSync} from '../../wasm/mw-sanitize/mw_sanitize.js';
import {RichTextEditor} from './RichTextEditor.tsx';
import {NativeHtmlVisualEditor,type NativeHtmlVisualApi} from './NativeHtmlVisualEditor.tsx';
import {loadCatalog} from '../../i18n/index.ts';

beforeAll(async()=>{
 initSync({module:readFileSync(resolve(process.cwd(),'src/wasm/mw-sanitize/mw_sanitize_bg.wasm'))});
 Object.defineProperty(Range.prototype,'getClientRects',{configurable:true,value:()=>[]});
 Object.defineProperty(Range.prototype,'getBoundingClientRect',{configurable:true,value:()=>({left:0,top:0,right:0,bottom:0,width:0,height:0,x:0,y:0,toJSON:()=>({})})});
 await loadCatalog('compose-format');await loadCatalog('compose-insert');await loadCatalog('mail');await loadCatalog('settings-policy');
});
beforeEach(()=>{state.map={};state.writes=[];});
afterEach(cleanup);

function mount(html:string){
 let out='';
 render(()=><RichTextEditor initialHtml={html} ariaLabel="Body" onChange={value=>{out=value;}}/>);
 return {html:()=>out,surface:()=>screen.getByTestId('compose-richtext')};
}
const selectAll=()=>fireEvent.click(screen.getByRole('button',{name:'Select all'}));
const flag=(id:string,value:boolean)=>{state.map[`compose.feature.${id}`]={value};};

describe('Styles gallery (rich editor)',()=>{
 it('shows live-preview tiles and applies a style from one',async()=>{
  const m=mount('<p>Hello</p>');
  const tiles=screen.getAllByTestId(/^style-tile-/);
  expect(tiles.map(t=>t.getAttribute('data-testid'))).toEqual(['style-tile-normal','style-tile-no-spacing','style-tile-heading1','style-tile-heading2']);
  const h1=screen.getByTestId('style-tile-heading1');
  expect((h1.querySelector('.cfmt-tile__sample') as HTMLElement).style.fontWeight).toBe('bold');
  expect(screen.getByTestId('style-tile-normal')).toHaveAttribute('aria-pressed','true');
  selectAll();
  fireEvent.click(h1);
  await waitFor(()=>expect(m.html()).toContain('<h1'));
  expect(m.html()).toContain('font-size: 20pt');
  expect(screen.getByTestId('style-tile-heading1')).toHaveAttribute('aria-pressed','true');
 });
 it('lists every style in the all-styles dropdown with previews and switches style set',async()=>{
  const m=mount('<p>Hello</p>');
  selectAll();
  fireEvent.click(screen.getByTestId('rt-all-styles'));
  const menu=await screen.findByRole('menu',{name:'All styles'});
  const rows=within(menu).getAllByRole('menuitemradio').filter(r=>r.getAttribute('data-testid')?.startsWith('style-')&&!r.getAttribute('data-testid')!.startsWith('style-set-'));
  expect(rows.map(r=>r.getAttribute('data-testid'))).toEqual(['normal','no-spacing','heading1','heading2','heading3','heading4','heading5','heading6','title','subtitle','strong','emphasis','intense-emphasis','quote','intense-quote','list-paragraph','task-item','code-inline','code','caption'].map(id=>`style-${id}`));
  expect(rows[2]!.textContent).toContain('Heading 1');
  fireEvent.click(screen.getByTestId('style-set-compact'));
  expect(state.writes).toContainEqual(['compose.styles.defaultSet','compact']);
  fireEvent.click(screen.getByTestId('style-heading1'));
  await waitFor(()=>expect(m.html()).toContain('font-size: 16pt'));
 });
 it('creates a style from the selection, applies it, keeps it per user and stops at the limit',async()=>{
  state.map['compose.styles.maxCustom']={value:1};
  const m=mount('<h3 style="margin-top: 12pt;"><strong><span style="font-size: 15pt;">Look</span></strong></h3><p>Other</p>');
  selectAll();
  fireEvent.click(screen.getByTestId('style-create'));
  fireEvent.input(await screen.findByTestId('style-name'),{target:{value:'Callout'}});
  fireEvent.click(screen.getByTestId('style-create-apply'));
  await waitFor(()=>expect(state.writes.some(([k])=>k==='compose.styles.custom')).toBe(true));
  const stored=JSON.parse(String(state.writes.find(([k])=>k==='compose.styles.custom')![1]));
  expect(stored[0]).toMatchObject({id:'c-callout',name:'Callout',block:'h3',marks:{fontSize:'15pt',bold:true}});
  fireEvent.click(screen.getByTestId('rt-all-styles'));
  expect(await screen.findByTestId('style-c-callout')).toHaveTextContent('Callout');
  fireEvent.click(screen.getByTestId('style-create'));
  fireEvent.input(await screen.findByTestId('style-name'),{target:{value:'Second'}});
  fireEvent.click(screen.getByTestId('style-create-apply'));
  expect(await screen.findByTestId('styles-message')).toHaveTextContent('You can keep 1 custom styles.');
  void m;
 });
 it('updates a style to match the selection (a built-in becomes an override that does not count)',async()=>{
  state.map['compose.styles.maxCustom']={value:0};
  const m=mount('<p><strong>Bold look</strong></p>');
  selectAll();
  fireEvent.click(screen.getByTestId('style-update'));
  await waitFor(()=>expect(state.writes.some(([k])=>k==='compose.styles.custom')).toBe(true));
  expect(JSON.parse(String(state.writes.find(([k])=>k==='compose.styles.custom')![1]))[0]).toMatchObject({id:'normal',marks:{bold:true}});
  void m;
 });
 it('resets to Normal',async()=>{
  const m=mount('<h2><strong>Head</strong></h2>');
  selectAll();
  fireEvent.click(screen.getByTestId('style-reset'));
  await waitFor(()=>expect(m.html()).toBe('<p>Head</p>'));
 });
 it('answers Ctrl+Alt+1 to 6, Ctrl+Alt+0 and Ctrl+Shift+N, and is inert with the gallery off',async()=>{
  const m=mount('<p>Hello</p>');
  fireEvent.keyDown(m.surface(),{key:'2',ctrlKey:true,altKey:true});
  await waitFor(()=>expect(m.html()).toContain('<h2'));
  fireEvent.keyDown(m.surface(),{key:'6',ctrlKey:true,altKey:true});
  await waitFor(()=>expect(m.html()).toContain('<h6'));
  fireEvent.keyDown(m.surface(),{key:'N',keyCode:78,ctrlKey:true,shiftKey:true});
  await waitFor(()=>expect(m.html()).toBe('<p>Hello</p>'));
 });
 it('hides the gallery and ignores its shortcuts when the feature is off, and hides custom styles when only those are off',()=>{
  flag('format.customStyles',false);
  mount('<p>Hello</p>');
  expect(screen.queryByTestId('style-create')).toBeNull();expect(screen.queryByTestId('style-update')).toBeNull();
  expect(screen.getByTestId('style-reset')).toBeInTheDocument();
  cleanup();
  flag('format.stylesRibbon',false);
  const m=mount('<p>Hello</p>');
  expect(screen.queryByTestId('rt-group-styles')).toBeNull();
  fireEvent.keyDown(m.surface(),{key:'5',ctrlKey:true,altKey:true});
  expect(m.html()).toBe('<p>Hello</p>');
 });
});

describe('Font dialog and shortcuts (rich editor)',()=>{
 it('opens with Ctrl+D, previews live, applies only what changed, and sets the default font',async()=>{
  const m=mount('<p>Hello</p>');
  selectAll();
  fireEvent.keyDown(m.surface(),{key:'d',ctrlKey:true});
  const dialog=await screen.findByRole('dialog',{name:'Font…'});
  fireEvent.input(within(dialog).getByTestId('font-size'),{target:{value:'16'}});
  expect((within(dialog).getByTestId('font-preview') as HTMLElement).style.fontSize).toBe('21px');
  fireEvent.click(within(dialog).getByTestId('font-bold'));
  fireEvent.click(within(dialog).getByTestId('font-effect-allCaps'));
  fireEvent.change(within(dialog).getByTestId('font-underline'),{target:{value:'wavy'}});
  fireEvent.change(within(dialog).getByTestId('font-spacing'),{target:{value:'expanded'}});
  expect((within(dialog).getByTestId('font-preview') as HTMLElement).style.textTransform).toBe('uppercase');
  fireEvent.click(within(dialog).getByTestId('font-set-default'));
  await waitFor(()=>expect(state.writes).toContainEqual(['compose.layout.defaultFont',expect.stringMatching(/^\|21\|/)]));
  fireEvent.click(within(dialog).getByTestId('font-dialog-apply'));
  await waitFor(()=>expect(m.html()).toContain('font-size: 16pt'));
  const out=m.html();
  expect(out).toContain('<strong>');expect(out).toContain('text-transform: uppercase');expect(out).toContain('letter-spacing: 0.1em');expect(out).toContain('text-decoration-style: wavy');
  expect(screen.queryByRole('dialog')).toBeNull();
 });
 it('opens from its button, closes with Escape, and Ctrl+D is inert when the feature is off',async()=>{
  const m=mount('<p>Hello</p>');
  fireEvent.click(screen.getByTestId('rt-font-dialog'));
  expect(await screen.findByRole('dialog',{name:'Font…'})).toBeInTheDocument();
  fireEvent.keyDown(screen.getByRole('dialog'),{key:'Escape'});
  await waitFor(()=>expect(screen.queryByRole('dialog')).toBeNull());
  cleanup();
  flag('format.fontDialog',false);
  const off=mount('<p>Hello</p>');
  expect(screen.queryByTestId('rt-font-dialog')).toBeNull();
  fireEvent.keyDown(off.surface(),{key:'d',ctrlKey:true});
  expect(screen.queryByRole('dialog')).toBeNull();
  void m;
 });
 it('clears formatting with Ctrl+\\ (and not when the feature is off)',async()=>{
  const m=mount('<p><strong>Bold</strong> text</p>');
  selectAll();
  fireEvent.keyDown(m.surface(),{key:'\\',ctrlKey:true});
  await waitFor(()=>expect(m.html()).toBe('<p>Bold text</p>'));
  cleanup();
  flag('format.clearFormatting',false);
  const off=mount('<p><strong>Bold</strong> text</p>');
  selectAll();
  fireEvent.keyDown(off.surface(),{key:'\\',ctrlKey:true});
  expect(off.html()).toContain('<strong>');
 });
});

function nativeMount(html:string){
 let api!:NativeHtmlVisualApi;
 render(()=><NativeHtmlVisualEditor html={html} revision={0} owned={()=>new Set()} onInput={()=>undefined} onReady={value=>api=value}/>);
 const frame=screen.getByTitle('Visual editor') as HTMLIFrameElement;
 const doc=frame.contentDocument!,view=frame.contentWindow as Window&{CSSStyleSheet:typeof CSSStyleSheet};
 if(!view.CSSStyleSheet.prototype.replaceSync)Object.defineProperty(view.CSSStyleSheet.prototype,'replaceSync',{value:()=>{},configurable:true});
 doc.documentElement.setAttribute('data-mw-native-document','true');fireEvent.load(frame);
 const range=doc.createRange();range.selectNodeContents(doc.getElementById('mw-native-editor')!);
 doc.getSelection()!.removeAllRanges();doc.getSelection()!.addRange(range);fireEvent(doc,new Event('selectionchange'));
 const key=(init:KeyboardEventInit)=>doc.dispatchEvent(new (doc.defaultView as unknown as typeof globalThis).KeyboardEvent('keydown',{bubbles:true,cancelable:true,...init}));
 return {api,doc,key};
}

describe('Styles, Font dialog and shortcuts (native editor)',()=>{
 it('applies a style from a tile and a shortcut, and sends no editor attributes',async()=>{
  const {api,doc,key}=nativeMount('<p>Hello</p>');
  fireEvent.click(screen.getByTestId('style-tile-heading1'));
  await waitFor(()=>expect(doc.querySelector('h1')).not.toBeNull());
  const sent=await api.capture();
  expect(sent).toContain('<h1');expect(sent).toContain('font-size: 20pt');expect(sent).not.toMatch(/data-mw/);
  key({key:'N',keyCode:78,ctrlKey:true,shiftKey:true});
  await waitFor(()=>expect(doc.querySelector('h1')).toBeNull());
  key({key:'3',ctrlKey:true,altKey:true});
  await waitFor(()=>expect(doc.querySelector('h3')).not.toBeNull());
 });
 it('opens the Font dialog with Ctrl+D and applies it; Ctrl+\\ clears formatting',async()=>{
  const {doc,key}=nativeMount('<p><b>Hello</b></p>');
  key({key:'d',ctrlKey:true});
  const dialog=await screen.findByTestId('font-dialog');
  fireEvent.input(within(dialog).getByTestId('font-size'),{target:{value:'18'}});
  fireEvent.click(within(dialog).getByTestId('font-dialog-apply'));
  await waitFor(()=>expect(Array.from(doc.querySelectorAll<HTMLElement>('#mw-native-editor span')).some(s=>s.style.fontSize==='18pt')).toBe(true));
  key({key:'\\',ctrlKey:true});
  await waitFor(()=>expect(doc.querySelector('b')).toBeNull());
  expect(doc.querySelector('#mw-native-editor span')).toBeNull();
 });
});

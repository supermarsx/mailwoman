import {beforeEach,describe,expect,it,vi} from 'vitest';

const state=vi.hoisted(()=>({map:{} as Record<string,{value:unknown;locked?:boolean}>}));
vi.mock('../../api/settings.ts',()=>({settings:()=>state.map,loadSettings:vi.fn(async()=>undefined)}));

import {cleanup,fireEvent,render,screen,waitFor,within} from '@solidjs/testing-library';
import {afterEach,beforeAll} from 'vitest';
import {RichTextEditor} from './RichTextEditor.tsx';
import {loadCatalog} from '../../i18n/index.ts';

beforeAll(async()=>{
 Object.defineProperty(Range.prototype,'getClientRects',{configurable:true,value:()=>[]});
 Object.defineProperty(Range.prototype,'getBoundingClientRect',{configurable:true,value:()=>({left:0,top:0,right:0,bottom:0,width:0,height:0,x:0,y:0,toJSON:()=>({})})});
 await loadCatalog('compose-format');await loadCatalog('mail');
});
beforeEach(()=>{state.map={};try{localStorage.clear();}catch{/* ignore */}});
afterEach(cleanup);

function mount(html:string){
 let out='';
 render(()=><RichTextEditor initialHtml={html} ariaLabel="Body" onChange={value=>{out=value;}}/>);
 return {html:()=>out,surface:()=>screen.getByTestId('compose-richtext')};
}
const selectAll=()=>fireEvent.click(screen.getByRole('button',{name:'Select all'}));
const flag=(id:string,value:boolean)=>{state.map[`compose.feature.${id}`]={value};};

describe('Format tab (rich editor)',()=>{
 it('orders the groups Clipboard, Format Painter, Font, Paragraph, Styles, Editing and labels the painter buttons with text',()=>{
  mount('<p>Hi</p>');
  const ids=Array.from(document.querySelectorAll('[data-ribbon-group]')).map(g=>g.getAttribute('data-testid'));
  expect(ids.slice(0,6)).toEqual(['rt-group-clipboard','rt-group-painter','rt-group-font','rt-group-paragraph','rt-group-styles','rt-group-editing']);
  const copy=screen.getByTestId('rt-copy-format'),apply=screen.getByTestId('rt-apply-format');
  expect(copy).toHaveTextContent('Copy formatting');expect(apply).toHaveTextContent('Apply formatting');
  expect(copy.getAttribute('data-tooltip')).toMatch(/Copy the formatting of the selected text/);
  expect(screen.getByTestId('rt-format-chip')).toHaveTextContent('Nothing copied');
 });
 it('gives every new control an accessible name',()=>{
  mount('<p>Hi</p>');
  for(const id of ['rt-font-grow','rt-font-shrink','rt-bullet','rt-bullet-types','rt-ordered','rt-ordered-types','rt-outdent','rt-indent','rt-clear-formatting','rt-clear-formatting-options','rt-line-spacing','rt-dir-ltr','rt-dir-rtl','rt-align-left','rt-paste-options']){
   const button=screen.getByTestId(id);
   expect(button.getAttribute('aria-label')?.length,id).toBeGreaterThan(0);
  }
  expect(screen.getByTestId('rt-font-grow')).toHaveAttribute('aria-label','Increase font size');
 });
 it('steps the font size with the buttons and the shortcut, and Shift steps by 1 pt',async()=>{
  const m=mount('<p>Hello</p>');
  selectAll();
  fireEvent.click(screen.getByTestId('rt-font-grow'));
  await waitFor(()=>expect(m.html()).toContain('font-size: 12pt'));
  fireEvent.keyDown(m.surface(),{key:'>',ctrlKey:true,shiftKey:true});
  await waitFor(()=>expect(m.html()).toContain('font-size: 14pt'));
  fireEvent.keyDown(m.surface(),{key:'<',ctrlKey:true,shiftKey:true});
  await waitFor(()=>expect(m.html()).toContain('font-size: 12pt'));
  fireEvent.click(screen.getByTestId('rt-font-shrink'),{shiftKey:true});
  await waitFor(()=>expect(m.html()).toContain('font-size: 11pt'));
 });
 it('opens the bullet dropdown with previews, moves with the arrow keys, applies and remembers a type',async()=>{
  const m=mount('<p>One</p><p>Two</p>');
  selectAll();
  fireEvent.click(screen.getByTestId('rt-bullet-types'));
  const menu=await screen.findByRole('menu',{name:'Bullet types'});
  const items=within(menu).getAllByRole('menuitemradio');
  expect(items.map(i=>i.getAttribute('data-style'))).toEqual(['disc','circle','square','dash','arrow','tick','check']);
  expect(items[3]!.querySelector('.cfmt-preview')).not.toBeNull();
  await waitFor(()=>expect(document.activeElement).toBe(items[0]));
  fireEvent.keyDown(menu,{key:'ArrowDown'});
  expect(document.activeElement).toBe(items[1]);
  fireEvent.click(items[2]!);
  await waitFor(()=>expect(m.html()).toContain('<ul style="list-style-type: square;">'));
  expect(localStorage.getItem('mw.compose.list.bullet')).toBe('square');
 });
 it('numbers with a chosen style and offers restart, continue and set start value',async()=>{
  const m=mount('<ol><li><p>One</p></li><li><p>Two</p></li></ol>');
  selectAll();
  fireEvent.click(screen.getByTestId('rt-ordered-types'));
  const menu=await screen.findByRole('menu',{name:'Numbering types'});
  expect(within(menu).getAllByRole('menuitemradio')).toHaveLength(9);
  fireEvent.click(screen.getByTestId('list-style-upper-roman'));
  await waitFor(()=>expect(m.html()).toContain('type="I"'));
  fireEvent.click(screen.getByTestId('rt-ordered-types'));
  fireEvent.input(await screen.findByTestId('list-start-input'),{target:{value:'4'}});
  fireEvent.click(screen.getByTestId('list-start-apply'));
  await waitFor(()=>expect(m.html()).toContain('start="4"'));
 });
 it('indents and outdents paragraphs by 40 px and clears formatting with the menu',async()=>{
  const m=mount('<p style="text-align:center"><strong>Bold</strong> text</p>');
  selectAll();
  fireEvent.click(screen.getByTestId('rt-indent'));
  await waitFor(()=>expect(m.html()).toContain('margin-left: 40px'));
  fireEvent.click(screen.getByTestId('rt-outdent'));
  await waitFor(()=>expect(m.html()).not.toContain('margin-left'));
  fireEvent.click(screen.getByTestId('rt-clear-formatting-options'));
  const menu=await screen.findByRole('menu',{name:'Clear formatting options'});
  expect(within(menu).getAllByRole('menuitem').map(i=>i.textContent)).toEqual(['Clear formatting','Clear text formatting only','Clear paragraph formatting','Clear everything including links']);
  fireEvent.click(screen.getByTestId('clear-text'));
  await waitFor(()=>expect(m.html()).not.toContain('<strong>'));
  expect(m.html()).toContain('text-align: center');
  fireEvent.keyDown(m.surface(),{key:' ',ctrlKey:true});
  await waitFor(()=>expect(m.html()).not.toContain('text-align'));
 });
 it('sets line spacing, custom spacing and direction',async()=>{
  const m=mount('<p>Body</p>');
  selectAll();
  fireEvent.click(screen.getByTestId('rt-line-spacing'));
  fireEvent.click(await screen.findByTestId('line-1.5'));
  await waitFor(()=>expect(m.html()).toContain('line-height: 1.5'));
  fireEvent.click(screen.getByTestId('rt-line-spacing'));
  fireEvent.input(await screen.findByTestId('line-custom-input'),{target:{value:'2.25'}});
  fireEvent.click(screen.getByTestId('line-custom-apply'));
  await waitFor(()=>expect(m.html()).toContain('line-height: 2.25'));
  fireEvent.click(screen.getByTestId('rt-dir-rtl'));
  await waitFor(()=>expect(m.html()).toContain('dir="rtl"'));
 });
 it('copies formatting into a chip, applies it, and keeps painting in sticky mode until Esc',async()=>{
  const m=mount('<p><strong><span style="font-size:14pt">Fancy</span></strong></p>');
  selectAll();
  fireEvent.click(screen.getByTestId('rt-copy-format'));
  expect(screen.getByTestId('rt-format-chip')).toHaveTextContent('14 · bold');
  expect(screen.getByTestId('rt-apply-format')).not.toBeDisabled();
  fireEvent.dblClick(screen.getByTestId('rt-copy-format'));
  expect(screen.getByTestId('rt-copy-format')).toHaveAttribute('aria-pressed','true');
  expect(screen.getByTestId('rt-format-chip')).toHaveTextContent('Esc to stop');
  fireEvent.keyDown(m.surface(),{key:'Escape'});
  expect(screen.getByTestId('rt-copy-format')).toHaveAttribute('aria-pressed','false');
 });
 it('Shift+Tab lifts a list item out; Tab on the first item leaves the list alone',async()=>{
  const m=mount('<ul><li><p>One</p><ul><li><p>Two</p></li></ul></li></ul>');
  const surface=m.surface();
  surface.focus();
  fireEvent.keyDown(surface,{key:'Tab'});
  expect(m.html()).toBe('<ul><li><p>One</p><ul><li><p>Two</p></li></ul></li></ul>');
  fireEvent.keyDown(surface,{key:'Tab',shiftKey:true});
  await waitFor(()=>expect(m.html()).toBe('<p>One</p><ul><li><p>Two</p></li></ul>'));
 });
});

describe('text effects and sort (rich editor)',()=>{
 it('lists effects with a client support hint and applies them, with Ctrl+= and Ctrl++ for sub and superscript',async()=>{
  const m=mount('<p>Hello</p>');
  selectAll();
  fireEvent.click(screen.getByTestId('rt-effects'));
  const menu=await screen.findByRole('menu',{name:'Text effects'});
  const items=within(menu).getAllByRole('menuitemcheckbox');
  expect(items.length).toBeGreaterThan(15);
  expect(screen.getByTestId('effect-superscript')).toHaveAttribute('data-support','most');
  expect(screen.getByTestId('effect-doubleStrike')).toHaveTextContent('Some apps ignore this');
  expect(screen.getByTestId('effect-superscript')).toHaveTextContent('Works in most mail apps');
  fireEvent.click(screen.getByTestId('effect-allCaps'));
  await waitFor(()=>expect(m.html()).toContain('text-transform: uppercase'));
  fireEvent.keyDown(m.surface(),{key:'=',ctrlKey:true});
  await waitFor(()=>expect(m.html()).toContain('<sub>'));
  fireEvent.keyDown(m.surface(),{key:'+',ctrlKey:true});
  await waitFor(()=>expect(m.html()).toContain('<sup>'));
  expect(m.html()).not.toContain('<sub>');
 });
 it('sorts the selected paragraphs from the Sort form and announces the result',async()=>{
  const m=mount('<p>pear</p><p>fig</p><p>apple</p>');
  selectAll();
  fireEvent.click(screen.getByTestId('rt-sort'));
  fireEvent.change(await screen.findByTestId('sort-order'),{target:{value:'desc'}});
  fireEvent.click(screen.getByTestId('sort-run'));
  await waitFor(()=>expect(m.html()).toBe('<p>pear</p><p>fig</p><p>apple</p>'));
  fireEvent.click(screen.getByTestId('rt-sort'));
  fireEvent.change(await screen.findByTestId('sort-order'),{target:{value:'asc'}});
  fireEvent.click(screen.getByTestId('sort-run'));
  await waitFor(()=>expect(m.html()).toBe('<p>apple</p><p>fig</p><p>pear</p>'));
  fireEvent.click(screen.getByTestId('rt-sort'));
  expect(await screen.findByTestId('sort-message')).toHaveTextContent('3 items sorted.');
  expect(screen.getByTestId('sort-message')).toHaveAttribute('aria-live','polite');
 });
 it('says so when there is nothing to sort',async()=>{
  mount('<p>one</p>');
  fireEvent.click(screen.getByTestId('rt-sort'));
  fireEvent.click(await screen.findByTestId('sort-run'));
  expect(screen.getByTestId('sort-message')).toHaveTextContent('Nothing to sort');
 });
});

describe('Format tab feature flags',()=>{
 it('hides the effects, the strikethrough button, the sort control and the styles gallery when switched off',()=>{
  for(const id of ['strikethrough','doubleStrikethrough','superscript','subscript','smallCaps','allCaps','underlineStyle','ligatures','textScale','textOutline','stylesRibbon','sort'])flag(`format.${id}`,false);
  mount('<p>Hi</p>');
  expect(screen.queryByTestId('rt-strike')).toBeNull();
  expect(screen.queryByTestId('rt-sort')).toBeNull();
  expect(screen.queryByTestId('rt-group-styles')).toBeNull();
  fireEvent.click(screen.getByTestId('rt-effects'));
  expect(screen.queryByTestId('effect-allCaps')).toBeNull();
  expect(screen.queryByTestId('effect-superscript')).toBeNull();
 });
 it('leaves Ctrl+= inert when subscript is off',async()=>{
  flag('format.subscript',false);
  const m=mount('<p>Hello</p>');
  selectAll();
  fireEvent.keyDown(m.surface(),{key:'=',ctrlKey:true});
  expect(m.html()).toBe('<p>Hello</p>');
 });
 it('renders nothing for a switched-off feature and makes its shortcut inert',async()=>{
  for(const id of ['fontSizeStep','lists','indent','clearFormatting','painter','lineSpacing','direction','pasteOptions'])flag(`format.${id}`,false);
  const m=mount('<p style="margin-left:40px"><strong>Hello</strong></p>');
  for(const id of ['rt-font-grow','rt-bullet','rt-ordered','rt-indent','rt-outdent','rt-clear-formatting','rt-line-spacing','rt-dir-rtl','rt-copy-format','rt-paste-options'])expect(screen.queryByTestId(id),id).toBeNull();
  expect(screen.queryByTestId('rt-group-painter')).toBeNull();
  selectAll();
  fireEvent.keyDown(m.surface(),{key:'>',ctrlKey:true,shiftKey:true});
  fireEvent.keyDown(m.surface(),{key:' ',ctrlKey:true});
  expect(m.html()).toBe('<p style="margin-left: 40px;"><strong>Hello</strong></p>');
 });
 it('hides the dropdown arrow but keeps the main list buttons when only list types are off',()=>{
  flag('format.listTypes',false);
  mount('<p>Hi</p>');
  expect(screen.getByTestId('rt-bullet')).toBeInTheDocument();
  expect(screen.queryByTestId('rt-bullet-types')).toBeNull();
  expect(screen.queryByTestId('rt-ordered-types')).toBeNull();
 });
});

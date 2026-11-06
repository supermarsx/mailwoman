import {describe,expect,it} from 'vitest';
import {EditorState,TextSelection,type Command} from 'prosemirror-state';
import {docFromHtml,htmlFromDoc,richSchema} from './richtext.ts';
import {applyStyleDef,captureStyle,currentStyleId} from './richFormatCommands.ts';
import {effectStyle} from './effects.ts';
import {allStyles,builtinStyle,BUILTIN_STYLE_IDS,newStyleId,parseCustomStyles,resolveStyle,saveCustomStyle,serializeCustomStyles,STYLE_SETS,type StyleDef} from './styleModel.ts';

function stateOf(html:string,from:string,to?:string):EditorState {
 const doc=docFromHtml(html);let start=-1,end=-1;
 doc.descendants((node,pos)=>{if(!node.isText)return;const a=node.text!.indexOf(from);if(a>=0&&start<0)start=pos+a+(to===undefined?from.length:0);const b=to===undefined?-1:node.text!.indexOf(to);if(b>=0&&end<0)end=pos+b+to!.length;});
 const state=EditorState.create({doc,schema:richSchema});
 return state.apply(state.tr.setSelection(TextSelection.create(doc,start,end<0?start:end)));
}
function run(state:EditorState,...commands:Command[]):EditorState {
 let current=state;
 for(const command of commands){let next=current;const ok=command(current,tr=>{next=current.apply(tr);});if(!ok)throw new Error('command did not apply');current=next;}
 return current;
}
const html=(s:EditorState)=>htmlFromDoc(s.doc);
const style=(id:string,set:(typeof STYLE_SETS)[number]='default')=>builtinStyle(set,id)!;

describe('built-in styles',()=>{
 it('defines every listed style in every set',()=>{
  for(const set of STYLE_SETS)for(const id of BUILTIN_STYLE_IDS)expect(builtinStyle(set,id),`${set}/${id}`).toBeDefined();
  expect(allStyles('default',[]).map(s=>s.id)).toEqual([...BUILTIN_STYLE_IDS]);
 });
 it('the sets look different',()=>{
  expect(style('heading1','compact').marks.fontSize).toBe('16pt');
  expect(style('heading1','modern').marks.color).toBe('#1f4e79');
  expect(style('heading1','classic').marks.fontFamily).toContain('Georgia');
  expect(style('heading1','default').marks.fontSize).toBe('20pt');
 });
});

describe('apply a style (rich editor)',()=>{
 it('makes a heading with a size, bold and spacing as tags and inline CSS',()=>{
  const out=html(run(stateOf('<p>Title here</p>','Title'),applyStyleDef(style('heading1'))));
  expect(out).toBe('<h1 style="margin-top: 12pt; margin-bottom: 6pt;"><strong><span style="font-size: 20pt;">Title here</span></strong></h1>');
  expect(out).not.toMatch(/data-mw|class=/);
 });
 it('replaces the earlier look when another style is applied, and Normal resets everything',()=>{
  let s=run(stateOf('<p>Hello</p>','Hello'),applyStyleDef(style('heading2')),applyStyleDef(style('title')));
  expect(html(s)).toContain('font-size: 28pt');expect(html(s)).not.toContain('<h2');expect(html(s)).not.toContain('20pt');expect(html(s)).not.toContain('<strong>');
  s=run(s,applyStyleDef(style('normal')));
  expect(html(s)).toBe('<p>Hello</p>');
  expect(currentStyleId(s)).toBe('normal');
 });
 it('makes quotes, intense quotes, code blocks and list paragraphs',()=>{
  expect(html(run(stateOf('<p>Said</p>','Said'),applyStyleDef(style('quote'))))).toBe('<blockquote><p><em>Said</em></p></blockquote>');
  expect(html(run(stateOf('<p>Said</p>','Said'),applyStyleDef(style('intense-quote'))))).toContain('text-align: center');
  expect(html(run(stateOf('<p>x = 1</p>','x'),applyStyleDef(style('code'))))).toBe('<pre><code>x = 1</code></pre>');
  expect(html(run(stateOf('<p>Item</p>','Item'),applyStyleDef(style('list-paragraph'))))).toBe('<p style="margin-left: 40px;">Item</p>');
  const leave=run(stateOf('<blockquote><p>Said</p></blockquote>','Said'),applyStyleDef(style('normal')));
  expect(html(leave)).toBe('<p>Said</p>');
 });
 it('makes a task item as a checklist',()=>{
  const out=html(run(stateOf('<p>Buy milk</p>','Buy'),applyStyleDef(style('task-item'))));
  expect(out).toContain('☐&nbsp;</span>Buy milk');
 });
 it('applies inline styles to the selection only, or as a pending style at a caret',()=>{
  expect(html(run(stateOf('<p>Say hello now</p>','hello','hello'),applyStyleDef(style('strong'))))).toBe('<p>Say <strong>hello</strong> now</p>');
  expect(html(run(stateOf('<p>Say hello now</p>','hello','hello'),applyStyleDef(style('intense-emphasis'))))).toContain('color: rgb(31, 78, 121)');
  const code=html(run(stateOf('<p>Run ls now</p>','ls','ls'),applyStyleDef(style('code-inline'))));
  expect(code).toContain('Courier New');expect(code).toContain('background-color: rgb(240, 240, 240)');
  const caret=run(stateOf('<p>Hi</p>','Hi'),applyStyleDef(style('emphasis')));
  expect(caret.storedMarks?.some(m=>m.type.name==='em')).toBe(true);
 });
 it('applies to every selected paragraph in one step',()=>{
  const s0=stateOf('<p>One</p><p>Two</p>','One','Two');
  let steps=0;applyStyleDef(style('caption'))(s0,()=>{steps+=1;});
  expect(steps).toBe(1);
  const out=html(run(s0,applyStyleDef(style('caption'))));
  expect(out.match(/font-size: 9pt/g)).toHaveLength(2);
 });
});

describe('custom styles',()=>{
 it('captures a selection as a style and applies it elsewhere',()=>{
  const source=run(stateOf('<p>Look</p>','Look','Look'),applyStyleDef(style('heading3','modern')));
  const captured=captureStyle(source);
  expect(captured.block).toBe('h3');expect(captured.marks).toMatchObject({fontSize:'15pt',bold:true,color:'#1f4e79'});
  const custom:StyleDef={id:'c-callout',name:'Callout',...captured};
  const out=html(run(stateOf('<p>Other</p>','Other'),applyStyleDef(custom)));
  expect(out).toContain('<h3');expect(out).toContain('font-size: 15pt');expect(out).toContain('color: rgb(31, 78, 121)');
 });
 it('stores and reads custom styles, dropping what is unreadable, and enforces the limit',()=>{
  const a:StyleDef={id:'c-a',name:'A',block:'p',attrs:{indent:2},marks:{bold:true,color:'#ff0000'}};
  const text=serializeCustomStyles([a]);
  expect(parseCustomStyles(text)).toEqual([a]);
  expect(parseCustomStyles('not json')).toEqual([]);
  expect(parseCustomStyles(JSON.stringify([{id:'Bad Id',name:'x',block:'p'},{id:'c-ok',name:'Ok',block:'p',attrs:{indent:99},marks:{color:'red',fontSize:'huge'}},{id:'c-x',block:'p'}]))).toEqual([{id:'c-ok',name:'Ok',block:'p',attrs:{},marks:{}}]);
  expect(saveCustomStyle([a],{...a,id:'c-b',name:'B'},1)).toBeNull();
  expect(saveCustomStyle([a],{...a,id:'c-b',name:'B'},2)).toHaveLength(2);
  expect(saveCustomStyle([a],{...a,marks:{italic:true}},1)![0]!.marks).toEqual({italic:true});
  expect(newStyleId('My Callout!',['c-my-callout'])).toBe('c-my-callout-2');
 });
 it('lets a custom style override a built-in one (update style to match selection)',()=>{
  const override:StyleDef={id:'heading1',block:'h1',attrs:{},marks:{fontSize:'30pt'}};
  expect(resolveStyle('default',[override],'heading1')!.marks.fontSize).toBe('30pt');
  expect(allStyles('default',[override]).length).toBe(BUILTIN_STYLE_IDS.length);
  expect(saveCustomStyle([],override,0)).toEqual([override]);
 });
});

// jsdom's CSS engine drops `font-variant: no-common-ligatures`, so this only runs where the
// engine keeps it (a real browser; it was checked by hand in Chrome, and the declaration
// itself is covered by the `effectStyle` test).
const keepsLigatureStyle=(()=>{const probe=document.createElement('span');probe.style.setProperty('font-variant','no-common-ligatures');return probe.style.fontVariant!=='';})();
describe('ligatures',()=>{
 it.skipIf(!keepsLigatureStyle)('keeps the ligature style through the rich schema',()=>{
  const out=htmlFromDoc(docFromHtml('<p><span style="font-variant: no-common-ligatures">office</span></p>'));
  expect(out).toContain('font-variant: no-common-ligatures');
 });
 it('maps the ligature effect to the declaration the sanitiser allows',()=>{
  expect(JSON.stringify(effectStyle('ligatures','none'))).toBe('[["font-variant","no-common-ligatures"]]');
 });
});

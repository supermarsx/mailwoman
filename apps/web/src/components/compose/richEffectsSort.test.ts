import {describe,expect,it} from 'vitest';
import {EditorState,TextSelection,type Command} from 'prosemirror-state';
import {docFromHtml,htmlFromDoc,richSchema} from './richtext.ts';
import {effectActive,sortSelection,toggleEffect} from './richFormatCommands.ts';
import {DEFAULT_SORT,sortItems,type SortOptions} from './sortModel.ts';
import {effectStyle,EFFECTS} from './effects.ts';

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
const sortOf=(over:Partial<SortOptions>):SortOptions=>({...DEFAULT_SORT,...over});

describe('font effects',()=>{
 it('writes each effect as plain inline CSS or a tag, and reads it back',()=>{
  const cases:[Parameters<typeof toggleEffect>[0],string,string][]=[
   ['superscript','','<sup>'],['subscript','','<sub>'],['smallCaps','','font-variant: small-caps'],['allCaps','','text-transform: uppercase'],
   ['doubleStrike','','text-decoration-style: double'],['underlineStyle','wavy','text-decoration-style: wavy'],   ['scale','150','font-stretch: 150%'],['spacing','expanded','letter-spacing: 0.1em'],['outline','#ff0000','text-shadow'],['shadow','','text-shadow'],
  ];
  for(const [kind,value,expected] of cases){
   const s=run(stateOf('<p>Hello</p>','Hello','Hello'),toggleEffect(kind,value));
   const out=html(s);
   expect(out,kind).toContain(expected);expect(out,kind).not.toMatch(/data-mw|class=/);
   const reopened=EditorState.create({doc:docFromHtml(out),schema:richSchema});
   const probe=reopened.apply(reopened.tr.setSelection(TextSelection.create(reopened.doc,2,4)));
   expect(effectActive(probe,kind),`${kind} reopened`).toBe(true);
  }
 });
 it('toggles off, replaces super with sub, and stacks different effects',()=>{
  let s=run(stateOf('<p>Hello</p>','Hello','Hello'),toggleEffect('superscript'));
  expect(effectActive(s,'superscript')).toBe(true);
  s=run(s,toggleEffect('subscript'));
  expect(effectActive(s,'superscript')).toBe(false);expect(html(s)).toContain('<sub>');expect(html(s)).not.toContain('<sup>');
  s=run(s,toggleEffect('allCaps'),toggleEffect('smallCaps'));
  expect(html(s)).toContain('uppercase');expect(html(s)).toContain('small-caps');
  s=run(s,toggleEffect('subscript'),toggleEffect('allCaps'),toggleEffect('smallCaps'));
  expect(html(s)).toBe('<p>Hello</p>');
 });
 it('applies to a caret as a pending style',()=>{
  const s=run(stateOf('<p>Hi</p>','Hi'),toggleEffect('allCaps'));
  expect(effectActive(s,'allCaps')).toBe(true);
 });
 it('lists a client support level for every effect and only emits allowed values',()=>{
  for(const effect of EFFECTS)expect(['most','some']).toContain(effect.support);
  // jsdom drops font-variant: no-common-ligatures, so the ligature declaration is checked here, not through the editor.
  expect(effectStyle('ligatures','none')).toEqual([['font-variant','no-common-ligatures']]);
  expect(effectStyle('scale','500')).toEqual([]);expect(effectStyle('outline','red')).toEqual([]);expect(effectStyle('underlineStyle','zigzag')).toEqual([]);
 });
});

describe('sortItems',()=>{
 const text=(s:string)=>s;
 it('sorts text naturally, case-insensitively, in both directions',()=>{
  expect(sortItems(['item10','Item2','item1'],text,sortOf({})).items).toEqual(['item1','Item2','item10']);
  expect(sortItems(['b','A','c'],text,sortOf({order:'desc'})).items).toEqual(['c','b','A']);
  expect(sortItems(['item10','item2'],text,sortOf({natural:false})).items).toEqual(['item10','item2']);
 });
 it('is case sensitive on request and can ignore leading articles',()=>{
  expect(sortItems(['The Zoo','An Apple','Banana'],text,sortOf({ignoreArticles:true})).items).toEqual(['An Apple','Banana','The Zoo']);
  expect(sortItems(['b','B','a'],text,sortOf({caseSensitive:true})).items).toHaveLength(3);
 });
 it('sorts numbers and dates, leaving unreadable ones last in their order',()=>{
  expect(sortItems(['10 apples','2 pears','n/a','1,5 kg','zzz'],text,sortOf({type:'number'})).items).toEqual(['1,5 kg','2 pears','10 apples','n/a','zzz']);
  expect(sortItems(['n/a','2024-03-01','2023-12-31','15/01/2024'],text,sortOf({type:'date'})).items).toEqual(['2023-12-31','15/01/2024','2024-03-01','n/a']);
  expect(sortItems(['n/a','2','1'],text,sortOf({type:'number',order:'desc'})).items).toEqual(['2','1','n/a']);
 });
 it('removes duplicates, reverses, shuffles with a given random and de-duplicates alone',()=>{
  expect(sortItems(['a','B','b','a'],text,sortOf({removeDuplicates:true})).items).toEqual(['a','B']);
  expect(sortItems(['a','b','a'],text,sortOf({mode:'unique'})).items).toEqual(['a','b']);
  expect(sortItems(['1','2','3'],text,sortOf({mode:'reverse'})).items).toEqual(['3','2','1']);
  const shuffled=sortItems(['1','2','3','4'],text,sortOf({mode:'shuffle'}),()=>0).items;
  expect([...shuffled].sort()).toEqual(['1','2','3','4']);expect(shuffled).not.toEqual(['1','2','3','4']);
 });
});

describe('sort the selection',()=>{
 it('sorts selected paragraphs in one undoable step',()=>{
  const s0=stateOf('<p>pear</p><p>apple</p><p>fig</p>','pear','fig');
  let steps=0;
  const ok=sortSelection(sortOf({}))(s0,tr=>{steps+=1;expect(tr.docChanged).toBe(true);});
  expect(ok).toBe(true);
  expect(steps).toBe(1);
  expect(html(run(s0,sortSelection(sortOf({}))))).toBe('<p>apple</p><p>fig</p><p>pear</p>');
  expect(html(run(s0,sortSelection(sortOf({order:'desc'}))))).toBe('<p>pear</p><p>fig</p><p>apple</p>');
 });
 it('sorts list items among their siblings and keeps nested lists attached',()=>{
  const list='<ul><li><p>pear</p><ul><li><p>sub</p></li></ul></li><li><p>apple</p></li></ul>';
  const out=html(run(stateOf(list,'pear'),sortSelection(sortOf({}))));
  expect(out).toBe('<ul><li><p>apple</p></li><li><p>pear</p><ul><li><p>sub</p></li></ul></li></ul>');
 });
 it('sorts table rows by a column, numerically, keeping the header row',()=>{
  const table='<table><tbody><tr><td><p>Name</p></td><td><p>Qty</p></td></tr><tr><td><p>b</p></td><td><p>10</p></td></tr><tr><td><p>a</p></td><td><p>9</p></td></tr><tr><td><p>c</p></td><td><p>100</p></td></tr></tbody></table>';
  const order=(s:EditorState)=>Array.from(new DOMParser().parseFromString(html(s),'text/html').querySelectorAll('tr')).map(r=>r.querySelector('td')!.textContent);
  expect(order(run(stateOf(table,'b'),sortSelection(sortOf({column:1,type:'number'}))))).toEqual(['Name','a','b','c']);
  expect(order(run(stateOf(table,'b'),sortSelection(sortOf({column:0,order:'desc'}))))).toEqual(['Name','c','b','a']);
  expect(order(run(stateOf(table,'b'),sortSelection(sortOf({column:0,header:false}))))).toEqual(['a','b','c','Name'].map(x=>x));
 });
 it('removes duplicate paragraphs and reports the outcome',()=>{
  let outcome:unknown;
  const s2=stateOf('<p>x</p><p>y</p><p>x</p>','x','x');
  const all=s2.apply(s2.tr.setSelection(TextSelection.create(s2.doc,1,s2.doc.content.size-1)));
  const next=run(all,sortSelection(sortOf({mode:'unique'}),o=>{outcome=o;}));
  expect(html(next)).toBe('<p>x</p><p>y</p>');
  expect(outcome).toEqual({kind:'blocks',count:2,removed:1});
 });
 it('does nothing for a caret in a single paragraph',()=>{
  expect(sortSelection(sortOf({}))(stateOf('<p>one</p>','one'))).toBe(false);
 });
});

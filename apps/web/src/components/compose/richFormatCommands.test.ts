import {describe,expect,it} from 'vitest';
import {EditorState,TextSelection,type Command} from 'prosemirror-state';
import {docFromHtml,htmlFromDoc,richSchema} from './richtext.ts';
import {applyListStyle,applyPaint,capturePaint,changeIndent,clearFormatting,currentFontSize,currentListType,setDirection,setLineSpacing,setListStart,sinkItem,stepFontSizeCommand,toggleChecked} from './richFormatCommands.ts';

/** A state on `html` with the selection covering `from`..`to` text (or the caret at the end of `from`). */
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
const html=(state:EditorState)=>htmlFromDoc(state.doc);

describe('font size steps',()=>{
 it('steps the selected text along the ramp, in both directions',()=>{
  let s=stateOf('<p>Hello world</p>','Hello','Hello');
  s=run(s,stepFontSizeCommand(1));expect(currentFontSize(s)).toBe(12);
  s=run(s,stepFontSizeCommand(1));expect(currentFontSize(s)).toBe(14);
  s=run(s,stepFontSizeCommand(-1),stepFontSizeCommand(-1),stepFontSizeCommand(-1));expect(currentFontSize(s)).toBe(10.5);
  expect(html(s)).toContain('font-size: 10.5pt');
 });
 it('steps by 1 pt with shift and applies to the caret as a pending style',()=>{
  let s=stateOf('<p>Hello</p>','Hello');
  s=run(s,stepFontSizeCommand(1,true));
  expect(currentFontSize(s)).toBe(12);
  expect(s.storedMarks?.some(m=>m.type.name==='fontSize'&&m.attrs['value']==='12pt')).toBe(true);
 });
 it('stops at the top of the ramp',()=>{
  const s=stateOf('<p><span style="font-size: 72pt">Big</span></p>','Big','Big');
  expect(currentFontSize(s)).toBe(72);
  expect(currentFontSize(run(s,stepFontSizeCommand(1)))).toBe(72);
 });
});

describe('list types',()=>{
 it('makes a bullet list of a chosen shape, changes it, and turns it off',()=>{
  let s=stateOf('<p>One</p><p>Two</p>','One','Two');
  s=run(s,applyListStyle('bullet','square'));
  expect(html(s)).toBe('<ul style="list-style-type: square;"><li><p>One</p></li><li><p>Two</p></li></ul>');
  expect(currentListType(s)).toEqual({kind:'bullet',style:'square'});
  s=run(s,applyListStyle('bullet','circle'));
  expect(html(s)).toContain('list-style-type: circle');
  s=run(s,applyListStyle('bullet','circle'));
  expect(html(s)).toBe('<p>One</p><p>Two</p>');
 });
 it('switches kinds and numbering styles with the legacy type attribute',()=>{
  let s=stateOf('<ul><li><p>One</p></li><li><p>Two</p></li></ul>','One');
  s=run(s,applyListStyle('ordered','lower-alpha'));
  expect(html(s)).toBe('<ol style="list-style-type: lower-alpha;" type="a"><li><p>One</p></li><li><p>Two</p></li></ol>');
  s=run(s,applyListStyle('ordered','upper-roman'));
  expect(html(s)).toContain('type="I"');
  s=run(s,applyListStyle('ordered','decimal'));
  expect(html(s)).toBe('<ol><li><p>One</p></li><li><p>Two</p></li></ol>');
 });
 it('sends dash, arrow, paren and zero lists as marker text without editor attributes',()=>{
  const out=html(run(stateOf('<p>One</p><p>Two</p>','One','Two'),applyListStyle('bullet','arrow')));
  expect(out).toContain('list-style-type: none');
  expect(out).toContain('<span>→&nbsp;</span>One');
  expect(out).not.toMatch(/data-mw|class=/);
  const paren=html(run(stateOf('<p>One</p><p>Two</p>','One','Two'),applyListStyle('ordered','paren')));
  expect(paren).toContain('<span>1)&nbsp;</span>One');expect(paren).toContain('<span>2)&nbsp;</span>Two');
  expect(html(run(stateOf('<p>One</p>','One'),applyListStyle('ordered','zero')))).toContain('01.');
 });
 it('round-trips a marker list through its sent form',()=>{
  const sent=html(run(stateOf('<p>One</p><p>Two</p>','One','Two'),applyListStyle('ordered','paren')));
  const doc=docFromHtml(sent);
  expect(htmlFromDoc(doc)).toBe(sent);
  expect(doc.firstChild!.type.name).toBe('ordered_list');expect(doc.firstChild!.attrs['listStyle']).toBe('paren');
 });
 it('makes a checklist whose boxes toggle and render as glyph text',()=>{
  let s=run(stateOf('<p>Milk</p><p>Eggs</p>','Milk','Eggs'),applyListStyle('bullet','check'));
  expect(html(s)).toContain('☐&nbsp;</span>Milk');
  s=s.apply(s.tr.setSelection(TextSelection.create(s.doc,4)));
  s=run(s,toggleChecked());
  const out=html(s);expect(out).toContain('☑&nbsp;</span>Milk');expect(out).toContain('☐&nbsp;</span>Eggs');
  expect(html(run(s,applyListStyle('bullet','disc')))).toBe('<ul><li><p>Milk</p></li><li><p>Eggs</p></li></ul>');
 });
 it('nests with a fresh look per level and offers outline and legal types',()=>{
  const s=run(stateOf('<ol><li><p>One</p></li><li><p>Two</p></li></ol>','Two'),sinkItem);
  expect(html(s)).toContain('<ol style="list-style-type: lower-alpha;" type="a">');
  const outline=run(stateOf('<ol><li><p>One</p><ol><li><p>Sub</p><ol><li><p>Deep</p></li></ol></li></ol></li></ol>','One'),applyListStyle('ordered','outline'));
  expect(html(outline)).toMatch(/<ol><li><p>One<\/p><ol style="list-style-type: lower-alpha;" type="a"><li><p>Sub<\/p><ol style="list-style-type: lower-roman;" type="i">/);
  let legal=run(stateOf('<p>A</p><p>B</p>','A','B'),applyListStyle('ordered','legal'));
  legal=legal.apply(legal.tr.setSelection(TextSelection.create(legal.doc,legal.doc.content.size-3)));
  const out=html(run(legal,sinkItem));
  expect(out).toContain('<span>1&nbsp;</span>A');expect(out).toContain('<span>1.1&nbsp;</span>B');
 });
 it('restarts, continues and sets the start value of numbering',()=>{
  const base='<ol><li><p>One</p></li><li><p>Two</p></li><li><p>Three</p></li></ol>';
  expect(html(run(stateOf(base,'Three'),setListStart('restart')))).toBe('<ol><li><p>One</p></li><li><p>Two</p></li></ol><ol><li><p>Three</p></li></ol>');
  expect(html(run(stateOf(base,'Two'),setListStart(5)))).toContain('start="5"');
  const split=stateOf('<ol><li><p>One</p></li><li><p>Two</p></li></ol><ol><li><p>Three</p></li></ol>','Three');
  expect(html(run(split,setListStart('continue')))).toContain('start="3"');
  expect(setListStart('continue')(stateOf(base,'One'))).toBe(false);
  expect(html(run(stateOf('<ol start="4"><li><p>One</p></li></ol>','One'),setListStart('restart')))).toBe('<ol><li><p>One</p></li></ol>');
 });
});

describe('indent, spacing and direction',()=>{
 it('indents a paragraph in 40 px steps and back',()=>{
  let s=stateOf('<p>Body</p>','Body');
  s=run(s,changeIndent(1),changeIndent(1));
  expect(html(s)).toBe('<p style="margin-left: 80px;">Body</p>');
  s=run(s,changeIndent(-1));expect(html(s)).toBe('<p style="margin-left: 40px;">Body</p>');
  s=run(s,changeIndent(-1));expect(html(s)).toBe('<p>Body</p>');
  expect(changeIndent(-1)(s)).toBe(false);
 });
 it('drops an imported margin when outdented',()=>{
  expect(html(run(stateOf('<p style="margin-left:40px">Body</p>','Body'),changeIndent(-1)))).toBe('<p>Body</p>');
 });
 it('nests and lifts list items',()=>{
  let s=stateOf('<ul><li><p>One</p></li><li><p>Two</p></li></ul>','Two');
  s=run(s,changeIndent(1));expect(html(s)).toContain('<ul><li><p>One</p><ul><li><p>Two</p></li></ul></li></ul>');
  s=run(s,changeIndent(-1));expect(html(s)).toBe('<ul><li><p>One</p></li><li><p>Two</p></li></ul>');
 });
 it('sets line spacing and direction',()=>{
  let s=run(stateOf('<p>Body</p>','Body'),setLineSpacing('1.5'),setDirection('rtl'));
  expect(html(s)).toBe('<p style="line-height: 1.5;" dir="rtl">Body</p>');
  s=run(s,setLineSpacing('2.25'));expect(html(s)).toContain('line-height: 2.25');
  s=run(s,setLineSpacing(null));expect(html(s)).toBe('<p dir="rtl">Body</p>');
 });
});

describe('clear formatting',()=>{
 const rich='<p style="text-align:center"><b>Bo</b><i>ld</i> <span style="color:#ff0000">red</span> <a href="https://x.test/">link</a></p>';
 it('text: removes marks and colour but keeps links and the paragraph look',()=>{
  expect(html(run(stateOf(rich,'Bo','link'),clearFormatting('text')))).toBe('<p style="text-align: center;">Bold red <a href="https://x.test/">link</a></p>');
 });
 it('paragraph: resets alignment, headings and lists but keeps marks',()=>{
  expect(html(run(stateOf('<h2 style="text-align:right"><b>Head</b></h2>','Head'),clearFormatting('paragraph')))).toBe('<p><strong>Head</strong></p>');
  expect(html(run(stateOf('<ul><li><p>One</p></li></ul>','One'),clearFormatting('paragraph')))).toBe('<p>One</p>');
 });
 it('all keeps links; everything removes them too',()=>{
  expect(html(run(stateOf(rich,'Bo','link'),clearFormatting('all')))).toBe('<p>Bold red <a href="https://x.test/">link</a></p>');
  expect(html(run(stateOf(rich,'Bo','link'),clearFormatting('everything')))).toBe('<p>Bold red link</p>');
 });
 it('at a caret it clears the pending style',()=>{
  const s=run(stateOf('<p>Hi</p>','Hi'),stepFontSizeCommand(1),clearFormatting('text'));
  expect(s.storedMarks?.length??0).toBe(0);
 });
});

describe('format painter',()=>{
 it('copies the look and applies it elsewhere',()=>{
  const source=stateOf('<p style="text-align:center"><strong><span style="color:#336699;font-size:14pt">Fancy</span></strong></p><p>Plain text</p>','Fancy','Fancy');
  const paint=capturePaint(source);
  expect(paint.summary).toMatchObject({bold:true,size:14,color:'#336699'});
  const out=html(run(stateOf('<p>Plain text</p>','Plain','text'),applyPaint(paint)));
  expect(out).toContain('font-size: 14pt');expect(out).toContain('<strong>');expect(out).toContain('text-align: center');
 });
 it('paints a caret as the pending style',()=>{
  const paint=capturePaint(stateOf('<p><b>Bold</b></p>','Bold','Bold'));
  const s=run(stateOf('<p>Other</p>','Other'),applyPaint(paint));
  expect(s.storedMarks?.some(m=>m.type.name==='strong')).toBe(true);
 });
});

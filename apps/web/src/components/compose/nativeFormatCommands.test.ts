import {afterEach,describe,expect,it,vi} from 'vitest';
import {createNativeHtmlCommands} from './nativeHtmlCommands.ts';
import {createNativeFormat} from './nativeFormatCommands.ts';
import {materializeLists,dematerializeLists} from './listMarkers.ts';

let host:HTMLElement|undefined;
afterEach(()=>{host?.remove();host=undefined;});
/** An editor with `html`, the selection over `from`..`to` text (a caret at the end of `from` when `to` is omitted). */
function setup(html:string,from:string,to?:string){
 host=document.createElement('div');host.contentEditable='true';host.innerHTML=html;document.body.append(host);
 const change=vi.fn();
 const commands=createNativeHtmlCommands({root:()=>host,change,selection:()=>undefined});
 const find=(text:string)=>{const walker=document.createTreeWalker(host!,4);let node:Node|null;while((node=walker.nextNode())){const at=(node.textContent??'').indexOf(text);if(at>=0)return {node,at};}throw new Error('no '+text);};
 const range=document.createRange();const a=find(from);
 if(to===undefined){range.setStart(a.node,a.at+from.length);range.collapse(true);}
 else{range.setStart(a.node,a.at);const b=find(to);range.setEnd(b.node,b.at+to.length);}
 commands.setRange(range);
 return {commands,format:createNativeFormat(commands),change};
}
/** What a recipient gets: marker lists as text, no editor attributes. */
function sent():string{const clone=host!.cloneNode(true) as HTMLElement;materializeLists(clone);return clone.innerHTML;}

describe('native font size steps',()=>{
 it('steps each selected run along the ramp and keeps the selection',()=>{
  const {format}=setup('<p>Hello <span style="font-size:14pt">big</span></p>','Hello','big');
  expect(format.stepFont(1)).toBe(true);
  const sizes=Array.from(host!.querySelectorAll<HTMLElement>('span')).map(s=>s.style.fontSize);
  expect(sizes).toContain('12pt');expect(sizes).toContain('16pt');
 });
 it('steps by 1 pt when fine and leaves a pending style at a caret',()=>{
  const {format}=setup('<p>Hi</p>','Hi');
  format.stepFont(1,true);
  expect(host!.querySelector<HTMLElement>('[data-mw-native-caret]')?.style.fontSize).toBe('12pt');
 });
});

describe('native lists',()=>{
 it('turns paragraphs into a list of a chosen shape, changes it and turns it off',()=>{
  const {format}=setup('<p>One</p><p>Two</p>','One','Two');
  format.applyList('bullet','square');
  expect(host!.innerHTML).toBe('<ul style="list-style-type: square;"><li>One</li><li>Two</li></ul>');
  expect(format.listType()).toEqual({kind:'bullet',style:'square'});
  format.applyList('ordered','lower-alpha');
  expect(host!.innerHTML).toBe('<ol style="list-style-type: lower-alpha;" type="a"><li>One</li><li>Two</li></ol>');
  format.applyList('ordered','lower-alpha');
  expect(host!.innerHTML).toBe('<p>One</p><p>Two</p>');
 });
 it('keeps default lists as bare tags',()=>{
  const {format}=setup('<p>One</p>','One');
  format.applyList('ordered','decimal');
  expect(host!.innerHTML).toBe('<ol><li>One</li></ol>');
 });
 it('sends marker lists as text and reads them back',()=>{
  const {format}=setup('<p>One</p><p>Two</p>','One','Two');
  format.applyList('ordered','paren');
  const out=sent();
  expect(out).toContain('<span>1)&nbsp;</span>One');expect(out).toContain('<span>2)&nbsp;</span>Two');
  expect(out).not.toMatch(/data-mw|class=/);
  const probe=document.createElement('div');probe.innerHTML=out;dematerializeLists(probe);
  expect(probe.querySelector('ol')?.getAttribute('data-mw-list')).toBe('paren');
  expect(probe.textContent).toBe('OneTwo');
 });
 it('makes a checklist with toggling boxes',()=>{
  const {format}=setup('<p>Milk</p><p>Eggs</p>','Milk','Eggs');
  format.applyList('bullet','check');
  expect(sent()).toContain('☐&nbsp;</span>Milk');
  host!.querySelector('li')!.querySelector('li');
  format.toggleChecked(host!.querySelector('li')!);
  expect(sent()).toContain('☑&nbsp;</span>Milk');
  expect(sent()).toContain('☐&nbsp;</span>Eggs');
 });
 it('nests with Tab-style indent, restyles the level, and lifts back',()=>{
  const {format}=setup('<ol><li>One</li><li>Two</li></ol>','Two');
  expect(format.indent(1)).toBe(true);
  expect(host!.innerHTML).toBe('<ol><li>One<ol style="list-style-type: lower-alpha;" type="a"><li>Two</li></ol></li></ol>');
  expect(format.indent(-1)).toBe(true);
  expect(host!.innerHTML).toBe('<ol><li>One</li><li>Two</li></ol>');
  format.indent(-1);
  expect(host!.innerHTML).toBe('<ol><li>One</li></ol><p>Two</p>');
 });
 it('cannot sink the first item',()=>{
  const {format}=setup('<ul><li>One</li></ul>','One');
  expect(format.indent(1)).toBe(false);
 });
 it('restarts, continues and sets the start value',()=>{
  const {format}=setup('<ol><li>One</li><li>Two</li><li>Three</li></ol>','Three');
  format.listStart('restart');
  expect(host!.innerHTML).toBe('<ol><li>One</li><li>Two</li></ol><ol><li>Three</li></ol>');
  expect(format.canContinue()).toBe(true);
  format.listStart('continue');
  expect(host!.innerHTML).toContain('<ol start="3"><li>Three</li></ol>');
  format.listStart(7);
  expect(host!.innerHTML).toContain('start="7"');
 });
});

describe('native indent, spacing, direction',()=>{
 it('indents paragraphs in 40 px steps and back',()=>{
  const {format}=setup('<p>Body</p>','Body');
  format.indent(1);format.indent(1);
  expect(host!.innerHTML).toBe('<p style="margin-left: 80px;">Body</p>');
  format.indent(-1);format.indent(-1);
  expect(host!.innerHTML).toBe('<p>Body</p>');
  expect(format.indent(-1)).toBe(false);
 });
 it('sets line spacing and direction on the block',()=>{
  const {format}=setup('<p>Body</p>','Body');
  format.setBlock('line-height','1.5');format.setDirection('rtl');
  expect(host!.innerHTML).toBe('<p style="line-height: 1.5;" dir="rtl">Body</p>');
 });
 it('wraps loose text in a paragraph before styling it',()=>{
  const {format}=setup('Loose text','Loose');
  format.indent(1);
  expect(host!.innerHTML).toBe('<p style="margin-left: 40px;">Loose text</p>');
 });
});

describe('native clear formatting',()=>{
 const rich='<p style="text-align:center"><b>Bo</b><i>ld</i> <span style="color:#ff0000">red</span> <a href="https://x.test/">link</a></p>';
 it('text: unwraps marks and colour, keeps links and the paragraph look',()=>{
  const {format}=setup(rich,'Bo','link');
  format.clear('text');
  expect(host!.innerHTML).toBe('<p style="text-align:center">Bold red <a href="https://x.test/">link</a></p>');
 });
 it('everything also unlinks',()=>{
  const {format}=setup(rich,'Bo','link');
  format.clear('everything');
  expect(host!.innerHTML).toBe('<p>Bold red link</p>');
 });
 it('paragraph: resets alignment, headings and lists',()=>{
  const {format}=setup('<h2 style="text-align:right"><b>Head</b></h2>','Head');
  format.clear('paragraph');
  expect(host!.innerHTML).toBe('<p><b>Head</b></p>');
  const second=setup('<ul><li>One</li></ul>','One');
  second.format.clear('paragraph');
  expect(host!.innerHTML).toBe('<p>One</p>');
 });
});

describe('native format painter',()=>{
 it('copies the look and applies it, removing the target\'s own',()=>{
  const {format}=setup('<p style="text-align:center"><b><span style="color:#336699;font-size:14pt">Fancy</span></b></p>','Fancy','Fancy');
  const taken=format.capture()!;
  expect(taken.summary.color).toBe('#336699');
  host!.innerHTML+='<p><i>Plain text</i></p>';
  const target=setup(host!.innerHTML,'Plain','text');
  target.format.applyPaint(taken.value);
  const html=host!.innerHTML;
  expect(html).toContain('Plain text');expect(html).not.toContain('<i>');
  expect(html).toMatch(/font-size: 14pt|font-size: 1[0-9.]+px/);
 });
});

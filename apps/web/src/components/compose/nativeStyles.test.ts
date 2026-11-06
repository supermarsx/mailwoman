import {afterEach,describe,expect,it,vi} from 'vitest';
import {createNativeHtmlCommands} from './nativeHtmlCommands.ts';
import {createNativeFormat} from './nativeFormatCommands.ts';
import {builtinStyle} from './styleModel.ts';

let host:HTMLElement|undefined;
afterEach(()=>{host?.remove();host=undefined;});
function setup(html:string,from:string,to?:string){
 host=document.createElement('div');host.contentEditable='true';host.innerHTML=html;document.body.append(host);
 const commands=createNativeHtmlCommands({root:()=>host,change:vi.fn(),selection:()=>undefined});
 const find=(text:string)=>{const walker=document.createTreeWalker(host!,4);let node:Node|null;while((node=walker.nextNode())){const at=(node.textContent??'').indexOf(text);if(at>=0)return {node,at};}throw new Error('no '+text);};
 const range=document.createRange();const a=find(from);
 if(to===undefined){range.setStart(a.node,a.at+from.length);range.collapse(true);}
 else{range.setStart(a.node,a.at);const b=find(to);range.setEnd(b.node,b.at+to.length);}
 commands.setRange(range);
 return {commands,format:createNativeFormat(commands)};
}
const style=(id:string,set:'default'|'modern'|'classic'|'compact'='default')=>builtinStyle(set,id)!;
/** The sent form: editor-only attributes removed. */
const sent=()=>host!.innerHTML.replace(/ data-mw-style-id="[^"]*"/g,'');

describe('native styles',()=>{
 it('makes a heading with size, weight and spacing as tags and inline CSS',()=>{
  const {format}=setup('<p>Title here</p>','Title');
  format.applyStyle(style('heading1'));
  const out=sent();
  expect(out).toContain('<h1');expect(out).toContain('margin-top: 12pt');expect(out).toContain('font-size: 20pt');expect(out).toContain('font-weight: bold');
  expect(format.currentStyleId()).toBe('heading1');
 });
 it('replaces the earlier look and Normal resets it',()=>{
  const {format}=setup('<p>Hello</p>','Hello');
  format.applyStyle(style('heading2'));format.applyStyle(style('title'));
  expect(sent()).toContain('font-size: 28pt');expect(sent()).not.toContain('<h2');expect(sent()).not.toContain('font-size: 16pt');
  format.applyStyle(style('normal'));
  expect(sent()).toBe('<p>Hello</p>');
 });
 it('makes quotes, code blocks and tasks, and leaves a quote for Normal',()=>{
  let s=setup('<p>Said</p>','Said');
  s.format.applyStyle(style('quote'));
  expect(sent()).toContain('<blockquote>');expect(sent()).toContain('font-style: italic');
  s.format.applyStyle(style('normal'));
  expect(sent()).toBe('<p>Said</p>');
  s=setup('<p>x = 1</p>','x');
  s.format.applyStyle(style('code'));
  expect(sent()).toBe('<pre>x = 1</pre>');
  s=setup('<p>Buy milk</p>','Buy');
  s.format.applyStyle(style('task-item'));
  expect(host!.querySelector('ul')?.getAttribute('data-mw-list')).toBe('check');
 });
 it('applies inline styles to the selection only',()=>{
  const {format}=setup('<p>Say hello now</p>','hello','hello');
  format.applyStyle(style('strong'));
  expect(host!.innerHTML).toBe('<p>Say <span style="font-weight: bold;">hello</span> now</p>');
 });
 it('captures a selection as a style and applies it to another block',()=>{
  const first=setup('<p>Look</p>','Look','Look');
  first.format.applyStyle(style('heading3','modern'));
  const captured=first.format.captureStyle()!;
  expect(captured.block).toBe('h3');expect(captured.attrs).toEqual({before:12,after:6});
  expect(captured.marks.bold).toBe(true);
  const second=setup('<p>Other</p>','Other');
  second.format.applyStyle({id:'c-callout',name:'Callout',...captured});
  expect(sent()).toContain('<h3');expect(sent()).toContain('margin-top: 12pt');
 });
});

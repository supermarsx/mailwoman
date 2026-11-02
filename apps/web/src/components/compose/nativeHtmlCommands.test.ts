import {afterEach, describe, expect, it, vi} from 'vitest';
import {createNativeHtmlCommands} from './nativeHtmlCommands.ts';

const originalExecCommand=Object.getOwnPropertyDescriptor(document,'execCommand');
afterEach(()=>{document.body.replaceChildren();document.getSelection()?.removeAllRanges();vi.restoreAllMocks();if(originalExecCommand)Object.defineProperty(document,'execCommand',originalExecCommand);else Reflect.deleteProperty(document,'execCommand');});

function fixture(html:string){
 const host=document.createElement('div');host.contentEditable='true';host.tabIndex=0;host.innerHTML=html;document.body.append(host);
 const change=vi.fn(),selection=vi.fn(),replaceNode=vi.fn();
 const commands=createNativeHtmlCommands({root:()=>host,change,selection,replaceNode});
 function select(start:Node,from=0,end:Node=start,to=from):void{
  const range=document.createRange();range.setStart(start,from);range.setEnd(end,to);const selected=document.getSelection()!;selected.removeAllRanges();selected.addRange(range);commands.remember();
 }
 function cell(index:number):HTMLTableCellElement{return host.querySelectorAll<HTMLTableCellElement>('td,th')[index]!;}
 function inCell(index:number):void{const current=cell(index);select(current.firstChild??current);}
 return {host,commands,change,selection,replaceNode,select,cell,inCell};
}
const table='<table><tbody><tr><td>A</td><td>B</td></tr><tr><td>C</td><td>D</td></tr></tbody></table>';
const spanned='<table><tbody><tr><td rowspan="2">A</td><td>B</td></tr><tr><td>C</td></tr></tbody></table>';

describe('native HTML selection and formatting',()=>{
 it('restores the saved message selection after a toolbar control takes focus',()=>{
  const {host,commands,select}=fixture('<p>Native formatting</p>');const text=host.querySelector('p')!.firstChild!;
  select(text,7,text,17);const button=document.createElement('button');document.body.append(button);button.focus();document.getSelection()?.removeAllRanges();
  const exec=vi.fn(()=>{expect(document.getSelection()?.toString()).toBe('formatting');return true;});
  Object.defineProperty(document,'execCommand',{value:exec,configurable:true});
  expect(commands.exec('bold')).toBe(true);expect(exec).toHaveBeenCalledWith('bold',false,undefined);
  expect(commands.range()?.toString()).toBe('formatting');expect(document.activeElement).toBe(host);
 });

 it('styles selected text across inline markup and keeps the native table and class structure',()=>{
  const {host,commands,select,change}=fixture('<table class="brand"><tr><td><p>First <b>second</b> last</p></td></tr></table>');
  const paragraph=host.querySelector('p')!,first=paragraph.firstChild!,last=paragraph.lastChild!;
  select(first,3,last,3);commands.inlineStyle('font-family','Georgia');
  expect(host.textContent).toBe('First second last');expect(host.querySelector('table')?.className).toBe('brand');
  expect(host.querySelectorAll('span')).toHaveLength(3);
  expect(Array.from(host.querySelectorAll('span')).every(span=>span.style.fontFamily==='Georgia')).toBe(true);
  expect(commands.range()?.toString()).toBe('st second la');
  expect(change).toHaveBeenCalledOnce();
 });

 it('applies paragraph spacing and case changes to the selected native DOM',()=>{
  const {host,commands,select}=fixture('<p>First</p><p>Second</p>');const paragraphs=host.querySelectorAll('p');
  select(paragraphs[0]!.firstChild!,0,paragraphs[1]!.firstChild!,6);commands.blockStyle('line-height','1.5');
  expect(Array.from(paragraphs).map(p=>p.style.lineHeight)).toEqual(['1.5','1.5']);
  commands.changeCase(true);expect(host.textContent).toBe('FIRSTSECOND');
 });

 it('rejects unsafe links without reaching the browser command',()=>{
  const {host,commands,select}=fixture('<p>Link text</p>');select(host.querySelector('p')!.firstChild!,0,host.querySelector('p')!.firstChild!,9);
  const exec=vi.fn(()=>true);Object.defineProperty(document,'execCommand',{value:exec,configurable:true});
  for(const value of ['javascript:evil()','data:text/html,x','https://example.test/\nattack','//example.test'])expect(commands.link(value)).toBe(false);
  expect(exec).not.toHaveBeenCalled();expect(host.querySelector('a')).toBeNull();
  expect(commands.link('https://example.test/path')).toBe(true);expect(exec).toHaveBeenLastCalledWith('createLink',false,'https://example.test/path');
 });
});

describe('native HTML table commands',()=>{
 it('inserts bounded tables in the selected DOM and refuses nesting and excessive dimensions',()=>{
  const {host,commands,select}=fixture('<p>Start</p>');select(host.querySelector('p')!.firstChild!,5);
  for(const dimensions of [[0,2],[21,2],[2,11],[1.5,2]])expect(commands.insertTable(dimensions[0]!,dimensions[1]!)).toBe(false);
  expect(host.querySelector('table')).toBeNull();expect(commands.insertTable(2,3)).toBe(true);
  const inserted=host.querySelector('table')!;expect(inserted.rows).toHaveLength(2);expect(inserted.querySelectorAll('td')).toHaveLength(6);
  expect(inserted.contains(commands.range()!.startContainer)).toBe(true);expect(commands.insertTable(1,1)).toBe(false);
 });

 it('extends crossing rowspans when adding a row and retains all cell content',()=>{
  const {host,commands,inCell}=fixture(spanned);inCell(2);
  expect(commands.table('row-before')).toBe(true);const rows=host.querySelector('table')!.rows;
  expect(rows).toHaveLength(3);expect(rows[0]!.cells[0]!.rowSpan).toBe(3);
  expect(Array.from(rows).map(row=>row.cells.length)).toEqual([2,1,1]);expect(host.textContent).toBe('ABC');
 });

 it('moves a rowspan origin into the next row when deleting its original row',()=>{
  const {host,commands,inCell}=fixture(spanned);inCell(0);expect(commands.table('row-delete')).toBe(true);
  const remaining=host.querySelector('table')!;expect(remaining.rows).toHaveLength(1);expect(Array.from(remaining.rows[0]!.cells).map(cell=>cell.textContent)).toEqual(['A','C']);
  expect(remaining.rows[0]!.cells[0]!.rowSpan).toBe(1);
 });

 it('deletes a column covered by another row cell without losing focus or throwing',()=>{
  const {host,commands,inCell}=fixture(spanned);inCell(2);
  expect(commands.table('column-delete')).toBe(true);expect(host.querySelector('table')!.rows).toHaveLength(2);
  expect(host.textContent).toBe('A');expect(host.contains(commands.range()!.startContainer)).toBe(true);
 });

 it('extends crossing colspans when inserting a column and adds cells only to uncovered rows',()=>{
  const {host,commands,inCell}=fixture('<table><tr><td colspan="2">A</td></tr><tr><td>B</td><td>C</td></tr></table>');inCell(2);
  expect(commands.table('column-before')).toBe(true);const rows=host.querySelector('table')!.rows;
  expect(rows[0]!.cells[0]!.colSpan).toBe(3);expect(Array.from(rows).map(row=>row.cells.length)).toEqual([1,3]);
  expect(host.textContent).toBe('ABC');expect(host.contains(commands.range()!.startContainer)).toBe(true);
 });

 it('merges a rectangular selection and splits the spanning cell back into its grid',()=>{
  const {host,commands,select,cell}=fixture(table);select(cell(0).firstChild!,0,cell(3).firstChild!,1);
  expect(commands.canTable('merge')).toBe(true);expect(commands.table('merge')).toBe(true);
  expect(host.querySelectorAll('td')).toHaveLength(1);expect(cell(0).rowSpan).toBe(2);expect(cell(0).colSpan).toBe(2);expect(cell(0).textContent).toBe('ABCD');
  expect(commands.table('split')).toBe(true);expect(host.querySelectorAll('td')).toHaveLength(4);
  expect(Array.from(host.querySelector('table')!.rows).map(row=>row.cells.length)).toEqual([2,2]);expect(cell(0).textContent).toBe('ABCD');
 });

 it('toggles header cells without losing native styles, spans, children or image bookkeeping',()=>{
  const {host,commands,inCell,replaceNode}=fixture('<table><tr><td colspan="2" class="brand" style="padding:9px"><b>Title</b></td></tr><tr><td>A</td><td>B</td></tr></table>');inCell(0);
  expect(commands.table('header')).toBe(true);const header=host.querySelector('th')!;
  expect(header.colSpan).toBe(2);expect(header.className).toBe('brand');expect(header.style.padding).toBe('9px');expect(header.querySelector('b')?.textContent).toBe('Title');expect(replaceNode).toHaveBeenCalledOnce();
  expect(commands.table('header')).toBe(true);expect(host.querySelector('th')).toBeNull();expect(host.querySelector('td')?.colSpan).toBe(2);
 });

 it('enforces grid bounds and declines malformed native tables without mutating them',()=>{
  const {host,commands,inCell}=fixture('<table><tr>'+Array.from({length:10},(_,i)=>`<td>${i}</td>`).join('')+'</tr></table>');inCell(0);const before=host.innerHTML;
  expect(commands.canTable('column-after')).toBe(false);expect(commands.table('column-after')).toBe(false);expect(host.innerHTML).toBe(before);
  host.innerHTML='<table><tr><td>A</td><td>B</td></tr><tr><td>C</td></tr></table>';inCell(2);const malformed=host.innerHTML;
  expect(commands.table('row-after')).toBe(false);expect(host.innerHTML).toBe(malformed);
 });
});

describe('native HTML find and replace',()=>{
 it('finds text across inline nodes, respects block boundaries and supports case sensitivity',()=>{
  const {commands}=fixture('<p>Al<b>ph</b>a alpha</p><p>beta</p>');
  expect(commands.matches('alpha').map(range=>range.toString())).toEqual(['Alpha','alpha']);
  expect(commands.matches('Alpha',true)).toHaveLength(1);expect(commands.matches('alpha beta')).toHaveLength(0);
 });

 it('moves next and previous by one occurrence and wraps at either boundary',()=>{
  const {commands}=fixture('<p>one X two X three X</p>');
  commands.resetSelection();expect(commands.find('X')).toBe(true);expect(commands.range()?.startOffset).toBe(4);
  expect(commands.find('X')).toBe(true);expect(commands.range()?.startOffset).toBe(10);
  expect(commands.find('X',-1)).toBe(true);expect(commands.range()?.startOffset).toBe(4);
  expect(commands.find('X',-1)).toBe(true);expect(commands.range()?.startOffset).toBe(18);
  expect(commands.find('X')).toBe(true);expect(commands.range()?.startOffset).toBe(4);
 });

 it('includes a match at a collapsed caret instead of skipping directly to the second occurrence',()=>{
  const {host,commands,select}=fixture('<p>X and X</p>');select(host.querySelector('p')!.firstChild!,0);
  expect(commands.find('X')).toBe(true);expect(commands.range()?.startOffset).toBe(0);expect(commands.range()?.toString()).toBe('X');
 });

 it('replaces only the selected match or all matches as inert text while preserving layout',()=>{
  const {host,commands}=fixture('<table class="brand"><tr><td><p>cat cat</p></td></tr></table>');
  expect(commands.replace('cat','dog')).toBe(false);commands.find('cat');expect(commands.replace('cat','dog')).toBe(true);expect(host.textContent).toBe('dog cat');
  expect(commands.replace('cat','<script>unsafe()</script>',true)).toBe(true);
  expect(host.textContent).toBe('dog <script>unsafe()</script>');expect(host.querySelector('script')).toBeNull();expect(host.querySelector('table')?.className).toBe('brand');
 });
});

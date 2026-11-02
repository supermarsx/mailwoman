/** Commands over the native editable DOM. No conversion through a rich-text schema. */
export type NativeTableAction='row-before'|'row-after'|'row-delete'|'column-before'|'column-after'|'column-delete'|'merge'|'split'|'header'|'delete';
type Cell=HTMLTableCellElement;
interface Position {cell:Cell;row:number;column:number;rows:number;columns:number}
const BLOCKS='p,div,li,blockquote,pre,h1,h2,h3,h4,h5,h6,td,th';
function element(node:Node|null):Element|null{return node?.nodeType===1?node as Element:node?.parentElement??null;}
function tableGrid(table:HTMLTableElement):{grid:Position[][];positions:Position[];width:number}|null {
 const rows=Array.from(table.rows);if(!rows.length||rows.length>20)return null;
 const grid:Position[][]=rows.map(()=>[]),positions:Position[]=[];
 for(let r=0;r<rows.length;r++){
  let column=0;
  for(const cell of Array.from(rows[r]!.cells)){
   while(grid[r]![column])column++;
   const pos={cell,row:r,column,rows:Math.min(cell.rowSpan,rows.length-r),columns:cell.colSpan};
   if(column+pos.columns>10||pos.rows<1||pos.columns<1)return null;
   positions.push(pos);
   for(let y=r;y<r+pos.rows;y++)for(let x=column;x<column+pos.columns;x++){if(grid[y]![x])return null;grid[y]![x]=pos;}
   column+=pos.columns;
  }
 }
 const width=Math.max(...grid.map(row=>row.length));
 return width&&grid.every(row=>row.length===width&&Array.from({length:width},(_,i)=>row[i]).every(Boolean))?{grid,positions,width}:null;
}
export function createNativeHtmlCommands(options:{root:()=>HTMLElement|undefined;change:()=>void;selection:()=>void;replaceNode?:(from:Element,to:Element)=>void}) {
 let saved:Range|null=null;
 const root=()=>options.root();
 function remember():void {const host=root(),selection=host?.ownerDocument.getSelection();if(host&&selection?.rangeCount){const range=selection.getRangeAt(0);if(host.contains(range.commonAncestorContainer))saved=range.cloneRange();}options.selection();}
 function restore():Range|null {const host=root();if(!host)return null;host.focus({preventScroll:true});const range=saved&&host.contains(saved.commonAncestorContainer)?saved.cloneRange():host.ownerDocument.createRange();if(range!==saved&&!saved){range.selectNodeContents(host);range.collapse(false);}if(!host.contains(range.commonAncestorContainer)){range.selectNodeContents(host);range.collapse(false);}const selection=host.ownerDocument.getSelection();selection?.removeAllRanges();selection?.addRange(range);return range;}
 function run(command:string,value?:string):boolean {const host=root();if(!host||!restore())return false;const result=host.ownerDocument.execCommand(command,false,value);remember();options.change();return result;}
 function current():Element|null {if(!saved)return null;let node=saved.startContainer;if(node.nodeType===1){const child=node.childNodes[saved.startOffset]??(saved.collapsed?node.childNodes[saved.startOffset-1]:undefined);if(child){node=child;while(node.firstChild)node=node.firstChild;}}return element(node);}
 function selectedText():{node:Text;from:number;to:number}[]{const host=root(),range=restore();if(!host||!range)return [];const walker=host.ownerDocument.createTreeWalker(host,4),nodes:{node:Text;from:number;to:number}[]=[];let node:Node|null;while((node=walker.nextNode())){if(!range.intersectsNode(node))continue;const text=node as Text,from=node===range.startContainer?range.startOffset:0,to=node===range.endContainer?range.endOffset:text.length;if(to>from)nodes.push({node:text,from,to});}return nodes;}
 function inlineStyle(property:string,value:string):void {
  const host=root(),range=restore();if(!host||!range)return;
  if(!value&&(property==='font-family'||property==='font-size')){const block=current()?.closest<HTMLElement>(BLOCKS)??host;value=host.ownerDocument.defaultView?.getComputedStyle(block).getPropertyValue(property)??(property==='font-family'?'Arial, sans-serif':'14px');}
  const nodes=selectedText();
  if(range.collapsed){const span=host.ownerDocument.createElement('span');span.style.setProperty(property,value);span.dataset['mwNativeCaret']='true';span.textContent='\u200b';range.insertNode(span);range.selectNodeContents(span);range.collapse(false);saved=range;restore();}
  else{const spans:HTMLElement[]=[];for(const {node,from,to} of nodes.reverse()){const piece=host.ownerDocument.createRange();piece.setStart(node,from);piece.setEnd(node,to);const span=host.ownerDocument.createElement('span');span.style.setProperty(property,value);piece.surroundContents(span);spans.unshift(span);}if(spans.length){const selected=host.ownerDocument.createRange();selected.setStart(spans[0]!,0);selected.setEnd(spans[spans.length-1]!,spans[spans.length-1]!.childNodes.length);saved=selected;restore();}}
  remember();options.change();
 }
 function blockStyle(property:string,value:string):void {const host=root(),range=restore();if(!host||!range)return;const blocks=Array.from(host.querySelectorAll<HTMLElement>(BLOCKS)).filter(block=>range.intersectsNode(block)&&!Array.from(block.children).some(child=>child.matches(BLOCKS)&&range.intersectsNode(child)));const active=current()?.closest<HTMLElement>(BLOCKS);if(range.collapsed){if(active&&active!==host&&host.contains(active))active.style.setProperty(property,value);else{host.ownerDocument.execCommand('formatBlock',false,'p');const block=element(host.ownerDocument.getSelection()?.anchorNode??null)?.closest<HTMLElement>(BLOCKS);if(block&&block!==host)block.style.setProperty(property,value);}}else for(const block of blocks)block.style.setProperty(property,value);remember();options.change();}
 function insertNode(node:Node):void {const host=root(),range=restore();if(!host||!range)return;range.deleteContents();range.insertNode(node);range.setStartAfter(node);range.collapse(true);saved=range;restore();remember();options.change();}
 function context(){const host=root(),cell=current()?.closest<Cell>('td,th'),table=cell?.closest('table');if(!host||!cell||!table||!host.contains(table))return null;const layout=tableGrid(table);if(!layout)return null;const pos=layout.positions.find(value=>value.cell===cell);return pos?{host,cell,table,layout,pos}:null;}
 function selectedCells():Cell[]{const ctx=context();if(!ctx)return [];if(!saved||saved.collapsed)return [ctx.cell];return ctx.layout.positions.filter(pos=>saved!.intersectsNode(pos.cell)).map(pos=>pos.cell);}
 function rectangle(){const ctx=context();if(!ctx)return null;const selected=selectedCells(),positions=ctx.layout.positions.filter(pos=>selected.includes(pos.cell));const top=Math.min(...positions.map(pos=>pos.row)),left=Math.min(...positions.map(pos=>pos.column)),bottom=Math.max(...positions.map(pos=>pos.row+pos.rows)),right=Math.max(...positions.map(pos=>pos.column+pos.columns));for(let y=top;y<bottom;y++)for(let x=left;x<right;x++)if(!selected.includes(ctx.layout.grid[y]![x]!.cell))return null;return {...ctx,selected,top,left,bottom,right};}
 function canTable(action:NativeTableAction):boolean {const ctx=context();if(!ctx)return false;if(action==='merge')return (rectangle()?.selected.length??0)>1;if(action==='split')return ctx.cell.colSpan>1||ctx.cell.rowSpan>1;if(action.startsWith('row-')&&action!=='row-delete')return ctx.table.rows.length<20;if(action.startsWith('column-')&&action!=='column-delete')return ctx.layout.width<10;return true;}
 function blank(doc:Document,header=false):Cell {const cell=doc.createElement(header?'th':'td');cell.style.border='1px solid #808080';cell.style.padding='4px';cell.style.verticalAlign='top';cell.append(doc.createElement('br'));return cell;}
 function focusCell(cell:Cell|undefined):void {const target=cell??root();if(!target)return;const range=target.ownerDocument.createRange();range.selectNodeContents(target);range.collapse(true);saved=range;restore();}
 function table(action:NativeTableAction):boolean {
  restore();const ctx=context();if(!ctx||!canTable(action))return false;
  const {table,layout,pos}=ctx,doc=table.ownerDocument;
  if(action==='delete'){const next=doc.createElement('p');next.append(doc.createElement('br'));table.replaceWith(next);const range=doc.createRange();range.selectNodeContents(next);range.collapse(true);saved=range;}
  else if(action==='merge'){
   const rect=rectangle();if(!rect)return false;const first=layout.grid[rect.top]![rect.left]!.cell;
   for(const cell of rect.selected){if(cell===first)continue;first.append(doc.createElement('br'),...Array.from(cell.childNodes));cell.remove();}
   first.rowSpan=rect.bottom-rect.top;first.colSpan=rect.right-rect.left;focusCell(first);
  }else if(action==='split'){
   const rows=pos.rows,columns=pos.columns;ctx.cell.rowSpan=1;ctx.cell.colSpan=1;
   for(let r=pos.row;r<pos.row+rows;r++)for(let c=pos.column;c<pos.column+columns;c++){if(r===pos.row&&c===pos.column)continue;const row=table.rows[r]!,before=Array.from(row.cells).find(cell=>{const original=layout.positions.find(p=>p.cell===cell);return original?original.column>c:false;});row.insertBefore(blank(doc,ctx.cell.tagName==='TH'),before??null);}
   focusCell(ctx.cell);
  }else if(action==='header'){
   const row=table.rows[0]!,header=Array.from(row.cells).some(cell=>cell.tagName!=='TH');
   for(const cell of Array.from(row.cells)){const next=doc.createElement(header?'th':'td');for(const attr of Array.from(cell.attributes))if(attr.name!=='style')next.setAttribute(attr.name,attr.value);next.style.cssText=cell.style.cssText;next.append(...Array.from(cell.childNodes));options.replaceNode?.(cell,next);if(cell===ctx.cell)ctx.cell=next;cell.replaceWith(next);}focusCell(ctx.cell);
  }else if(action==='row-delete'){
   if(table.rows.length===1)return tableActionDelete();const next=table.rows[pos.row+1];
   for(const entry of layout.positions){if(entry.row<pos.row&&entry.row+entry.rows>pos.row)entry.cell.rowSpan--;else if(entry.row===pos.row&&entry.rows>1&&next){entry.cell.rowSpan--;const before=Array.from(next.cells).find(cell=>(layout.positions.find(p=>p.cell===cell)?.column??Infinity)>entry.column);next.insertBefore(entry.cell,before??null);}}
   table.rows[pos.row]!.remove();focusCell(tableGrid(table)?.grid[Math.min(pos.row,table.rows.length-1)]?.[0]?.cell);
  }else if(action==='column-delete'){
   if(layout.width===1)return tableActionDelete();for(const entry of layout.positions)if(entry.column<=pos.column&&entry.column+entry.columns>pos.column){if(entry.columns>1)entry.cell.colSpan--;else entry.cell.remove();}focusCell(tableGrid(table)?.grid[pos.row]?.[0]?.cell);
  }else if(action==='row-before'||action==='row-after'){
   const at=pos.row+(action==='row-after'?pos.rows:0),row=doc.createElement('tr'),crossing=layout.positions.filter(entry=>entry.row<at&&entry.row+entry.rows>at);for(const entry of crossing)entry.cell.rowSpan++;
   for(let c=0;c<layout.width;c++)if(!crossing.some(entry=>entry.column<=c&&entry.column+entry.columns>c))row.append(blank(doc));
   const before=table.rows[at],parent=before?.parentElement??table.rows[table.rows.length-1]!.parentElement!;parent.insertBefore(row,before??null);focusCell(row.cells[0]??ctx.cell);
  }else{
   const at=pos.column+(action==='column-after'?pos.columns:0),crossing=layout.positions.filter(entry=>entry.column<at&&entry.column+entry.columns>at);for(const entry of crossing)entry.cell.colSpan++;
   for(let r=0;r<table.rows.length;r++){if(crossing.some(entry=>entry.row<=r&&entry.row+entry.rows>r))continue;const row=table.rows[r]!,before=Array.from(row.cells).find(cell=>(layout.positions.find(p=>p.cell===cell)?.column??Infinity)>=at);row.insertBefore(blank(doc),before??null);}focusCell(table.rows[pos.row]!.cells[Math.min(at,table.rows[pos.row]!.cells.length-1)]!);
  }
  restore();remember();options.change();return true;
  function tableActionDelete():boolean {return tableCommandDelete();}
  function tableCommandDelete():boolean {const next=doc.createElement('p');next.append(doc.createElement('br'));table.replaceWith(next);const range=doc.createRange();range.selectNodeContents(next);range.collapse(true);saved=range;restore();remember();options.change();return true;}
 }
 function insertTable(rows:number,columns:number):boolean {const host=root();if(!host||!Number.isInteger(rows)||!Number.isInteger(columns)||rows<1||rows>20||columns<1||columns>10||rows*columns>200||host.querySelectorAll('table').length>=10||current()?.closest('table'))return false;const table=host.ownerDocument.createElement('table');table.style.borderCollapse='collapse';table.style.width='100%';const body=table.createTBody();for(let r=0;r<rows;r++){const row=body.insertRow();for(let c=0;c<columns;c++)row.append(blank(host.ownerDocument));}insertNode(table);focusCell(table.rows[0]!.cells[0]!);remember();return true;}
 function matches(needle:string,caseSensitive=false):Range[]{const host=root();if(!host||!needle)return [];const walker=host.ownerDocument.createTreeWalker(host,4),segments:{node:Text;start:number}[]=[];let text='',previous:Element|null=null,node:Node|null;while((node=walker.nextNode())){const block=element(node)?.closest(BLOCKS)??host;if(previous&&block!==previous)text+='\n';segments.push({node:node as Text,start:text.length});text+=node.textContent??'';previous=block;}const haystack=caseSensitive?text:text.toLocaleLowerCase(),query=caseSensitive?needle:needle.toLocaleLowerCase(),ranges:Range[]=[];let from=0;while((from=haystack.indexOf(query,from))>=0&&ranges.length<10000){const end=from+query.length,first=segments.find(part=>part.start<=from&&part.start+part.node.length>from),last=segments.find(part=>part.start<end&&part.start+part.node.length>=end);if(first&&last){const range=host.ownerDocument.createRange();range.setStart(first.node,from-first.start);range.setEnd(last.node,end-last.start);ranges.push(range);}from=end;}return ranges;}
 function find(needle:string,direction=1,caseSensitive=false):boolean {const found=matches(needle,caseSensitive);if(!found.length)return false;let at:number;if(direction<0){at=-1;for(let index=0;index<found.length;index++)if(!saved||found[index]!.compareBoundaryPoints(0,saved)<0)at=index;if(at<0)at=found.length-1;}else{at=found.findIndex(range=>!saved||(range.compareBoundaryPoints(0,saved)>0||(saved.collapsed&&range.compareBoundaryPoints(0,saved)===0)));if(at<0)at=0;}saved=found[at]!.cloneRange();restore();remember();return true;}
 function replace(needle:string,value:string,all=false,caseSensitive=false):boolean {const host=root();if(!host)return false;const found=matches(needle,caseSensitive).filter(range=>all||(saved&&range.compareBoundaryPoints(0,saved)===0&&range.compareBoundaryPoints(2,saved)===0));if(!found.length)return false;for(const range of found.reverse()){range.deleteContents();const node=host.ownerDocument.createTextNode(value);range.insertNode(node);range.selectNode(node);saved=range.cloneRange();}restore();remember();options.change();return true;}
 return {remember,restore,current,exec:run,inlineStyle,blockStyle,insertNode,canTable,table,insertTable,matches,find,replace,selectedText,
  state:(name:string)=>{try{return root()?.ownerDocument.queryCommandState(name)??false;}catch{return false;}},
  selectAll:()=>{const host=root();if(!host)return;const range=host.ownerDocument.createRange();range.selectNodeContents(host);saved=range;restore();remember();},
  clearSelection:()=>{if(saved){saved.collapse(false);restore();remember();}},
  link:(url:string)=>{if(!url)return run('unlink');if(!/^(https?:|mailto:|tel:)/i.test(url)||(/\s/.test(url)||[...url].some(character=>character.charCodeAt(0)<32||character.charCodeAt(0)===127)))return false;return run('createLink',url);},
  changeCase:(mode:boolean|'lower'|'upper'|'title'|'sentence')=>{let wordStart=true,sentenceStart=true;for(const {node,from,to} of selectedText()){const source=node.data.slice(from,to);const changed=[...source].map(character=>{const letter=/[\p{L}\p{N}]/u.test(character);const upper=mode===true||mode==='upper'||(mode==='title'&&wordStart)||(mode==='sentence'&&sentenceStart);const value=upper?character.toLocaleUpperCase():character.toLocaleLowerCase();if(letter){wordStart=false;sentenceStart=false;}else{wordStart=true;if(/[.!?]/.test(character))sentenceStart=true;}return value;}).join('');node.replaceData(from,to-from,changed);}remember();options.change();},
  sortBlocks:(numeric=false)=>{const host=root(),range=restore();if(!host||!range||range.collapsed)return false;const blocks=Array.from(host.querySelectorAll<HTMLElement>('p,li,h1,h2,h3,h4,h5,h6')).filter(block=>range.intersectsNode(block));if(blocks.length<2||blocks.some(block=>block.parentElement!==blocks[0]!.parentElement))return false;const sorted=[...blocks].sort((a,b)=>(a.textContent??'').localeCompare(b.textContent??'',undefined,{numeric}));const marker=host.ownerDocument.createComment('sort');blocks[0]!.before(marker);for(const block of sorted)marker.before(block);marker.remove();const selected=host.ownerDocument.createRange();selected.setStartBefore(sorted[0]!);selected.setEndAfter(sorted[sorted.length-1]!);saved=selected;restore();remember();options.change();return true;},
  cellStyle:(property:string,value:string)=>{restore();const cells=selectedCells();if(!cells.length)return false;for(const cell of cells)cell.style.setProperty(property,value);remember();options.change();return true;},
  range:()=>saved?.cloneRange()??null,
  setRange:(range:Range)=>{const host=root();if(host?.contains(range.commonAncestorContainer)){saved=range.cloneRange();restore();remember();}},
  resetSelection:()=>{saved=null;},
 };
}
export type NativeHtmlCommands=ReturnType<typeof createNativeHtmlCommands>;

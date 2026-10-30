import {readEmailStyles,applyEmailStyles} from './safeEmailStyles.ts';
import type {NodeSpec,MarkSpec,Node as PMNode} from 'prosemirror-model';
import {tableNodes} from 'prosemirror-tables';
export const TABLE_LIMITS={rows:20,columns:10,cells:200,tables:10} as const;
export const LINE_HEIGHTS=['1','1.15','1.5','2','2.5','3'] as const;
export const PARAGRAPH_SPACING=[0,6,12,18,24,36] as const;
export function safeColor(value:unknown):string|null {
 if(typeof value!=='string')return null;
 const text=value.trim().toLowerCase();
 if(/^#[0-9a-f]{6}$/.test(text))return text;
 if(/^#[0-9a-f]{3}$/.test(text))return '#'+[...text.slice(1)].map(character=>character+character).join('');
 const match=/^rgb\(\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*(\d{1,3})\s*\)$/.exec(text);
 if(!match)return null;const channels=match.slice(1).map(Number);return channels.every(value=>value<=255)?'#'+channels.map(value=>value.toString(16).padStart(2,'0')).join(''):null;
}
export const paragraphAttrs={emailStyles:{default:null},htmlTag:{default:'p'},align:{default:null},lineHeight:{default:null},before:{default:null},after:{default:null},indent:{default:0},shade:{default:null},preset:{default:null}};
function spacing(value:string):number|null {const match=/^(\d+)pt$/.exec(value);return match&&PARAGRAPH_SPACING.includes(Number(match[1]) as typeof PARAGRAPH_SPACING[number])?Number(match[1]):value==='0px'?0:null;}
export function readParagraph(element:HTMLElement):Record<string,unknown> {
 const style=element.style,margin=/^(\d+(?:\.\d+)?)em$/.exec(style.marginLeft),indent=margin?Number(margin[1])/1.5:0;
 return {emailStyles:readEmailStyles(element),htmlTag:element.tagName.toLowerCase(),align:['left','center','right','justify'].includes(style.textAlign)?style.textAlign:null,lineHeight:LINE_HEIGHTS.includes(style.lineHeight as typeof LINE_HEIGHTS[number])?style.lineHeight:null,before:spacing(style.marginTop),after:spacing(style.marginBottom),indent:Number.isInteger(indent)&&indent>=0&&indent<=6?indent:0,shade:safeColor(style.backgroundColor),preset:['title','subtitle','no-spacing'].includes(element.dataset['mwStyle']??'')?element.dataset['mwStyle']:null};
}
export function paragraphDOM(node:PMNode,tag:string):{dom:HTMLElement;contentDOM:HTMLElement} {
 const element=document.createElement(tag),a=node.attrs;applyEmailStyles(element,a['emailStyles']);
 if(['left','center','right','justify'].includes(a['align']))element.style.textAlign=a['align'];
 if(LINE_HEIGHTS.includes(a['lineHeight']))element.style.lineHeight=a['lineHeight'];
 if(PARAGRAPH_SPACING.includes(a['before']))element.style.marginTop=`${a['before']}pt`;
 if(PARAGRAPH_SPACING.includes(a['after']))element.style.marginBottom=`${a['after']}pt`;
 if(Number.isInteger(a['indent'])&&a['indent']>0&&a['indent']<=6)element.style.marginLeft=`${a['indent']*1.5}em`;
 const shade=safeColor(a['shade']);if(shade)element.style.backgroundColor=shade;
 if(['title','subtitle','no-spacing'].includes(a['preset'])){element.dataset['mwStyle']=a['preset'];if(a['preset']==='title')element.style.fontSize='28pt';if(a['preset']==='subtitle')element.style.fontSize='18pt';}
 return {dom:element,contentDOM:element};
}
export function styledBlock(spec:NodeSpec,heading=false):NodeSpec {
 return {...spec,attrs:{...spec.attrs,...paragraphAttrs},parseDOM:heading?[1,2,3,4,5,6].map(level=>({tag:`h${level}`,getAttrs:(element:HTMLElement)=>({...readParagraph(element),level})})):[{tag:'p',getAttrs:readParagraph},{tag:'div',getAttrs:element=>hasBlockChildren(element)?false:readParagraph(element)}],toDOM:node=>paragraphDOM(node,heading?`h${node.attrs['level']}`:node.attrs['htmlTag']==='div'?'div':'p')};
}
const hasBlockChildren=(element:HTMLElement)=>!!element.querySelector('p,div,h1,h2,h3,h4,h5,h6,table,ul,ol,blockquote,pre,hr');
export const emailContainer:NodeSpec={group:'block',content:'block+',attrs:{emailStyles:{default:null}},parseDOM:[{tag:'div',getAttrs:element=>hasBlockChildren(element)?{emailStyles:readEmailStyles(element)}:false}],toDOM:node=>{const element=document.createElement('div');applyEmailStyles(element,node.attrs['emailStyles']);return {dom:element,contentDOM:element};}};
export function colorMark(property:'color'|'background-color'):MarkSpec {
 return {attrs:{value:{}},parseDOM:[{style:property,getAttrs:value=>{const color=safeColor(value);return color?{value:color}:false;}}],toDOM:mark=>{const element=document.createElement('span'),color=safeColor(mark.attrs['value']);if(color)element.style.setProperty(property,color);return {dom:element,contentDOM:element};}};
}
export const composeTableNodes=tableNodes({tableGroup:'block',cellContent:'block+',cellAttributes:{emailStyles:{default:null,getFromDOM:readEmailStyles},shade:{default:null,getFromDOM:element=>safeColor(element.style.backgroundColor),setDOMAttr:(value,attrs)=>{const color=safeColor(value);if(color)attrs['style']=`background-color:${color}`;}}}});
// Emit email-visible table borders; editor-only stylesheet rules are not sent.
// Styles through the CSSOM: a `style` attribute is refused by the production CSP (style-src 'self', t49).
composeTableNodes.table.attrs={emailStyles:{default:null}};
composeTableNodes.table.parseDOM=[{tag:'table',getAttrs:element=>({emailStyles:readEmailStyles(element)})}];
composeTableNodes.table.toDOM=node=>{const table=document.createElement('table'),body=document.createElement('tbody');if(node.attrs['emailStyles']===null){applyEmailStyles(table,{'width':'100%','border-collapse':'collapse'});}else applyEmailStyles(table,node.attrs['emailStyles']);table.appendChild(body);return {dom:table,contentDOM:body};};
for(const name of ['table_cell','table_header'] as const){
 composeTableNodes[name].toDOM=node=>{
  const cell=document.createElement(name==='table_header'?'th':'td');
  cell.colSpan=node.attrs['colspan'];cell.rowSpan=node.attrs['rowspan'];
  if(node.attrs['emailStyles']===null){const defaults=document.createElement('td');defaults.style.border='1px solid #808080';defaults.style.padding='4px';defaults.style.verticalAlign='top';applyEmailStyles(cell,readEmailStyles(defaults));}else applyEmailStyles(cell,node.attrs['emailStyles']);
  const color=safeColor(node.attrs['shade']);if(color)cell.style.backgroundColor=color;
  return {dom:cell,contentDOM:cell};
 };
}
/** Flatten oversized or excessively nested pasted tables without dropping their text. */
export function boundTableHtml(root:HTMLElement):void {
 const tables=[...root.querySelectorAll('table')];
 let cells=0;
 tables.forEach((table,index)=>{
  if(!root.contains(table))return;
  const rows=Array.from(table.rows);
  let depth=0;for(let parent=table.parentElement?.closest('table');parent;parent=parent.parentElement?.closest('table'))depth++;
  cells+=rows.reduce((sum,row)=>sum+row.cells.length,0);
  const invalid=index>=TABLE_LIMITS.tables||depth>=3||cells>TABLE_LIMITS.cells||rows.length>TABLE_LIMITS.rows||rows.some(row=>Array.from(row.cells).reduce((sum,cell)=>sum+cell.colSpan,0)>TABLE_LIMITS.columns)||Array.from(table.querySelectorAll('td,th')).some(cell=>Number(cell.getAttribute('rowspan')??1)>TABLE_LIMITS.rows);
  if(invalid){const paragraph=root.ownerDocument.createElement('p');paragraph.textContent=rows.map(row=>Array.from(row.cells).map(cell=>cell.textContent??'').join('\t')).join('\n');table.replaceWith(paragraph);return;}
  const occupied=Array<number>(TABLE_LIMITS.columns).fill(0);
  let overflow=false;
  rows.forEach((row,rowIndex)=>{
   let column=0;
   for(const cell of Array.from(row.cells)){
    while(column<TABLE_LIMITS.columns&&occupied[column]!>0)column++;
    let free=0;while(column+free<TABLE_LIMITS.columns&&occupied[column+free]===0)free++;
    if(cell.colSpan>free){overflow=true;break;}
    cell.rowSpan=Math.max(1,Math.min(rows.length-rowIndex,cell.rowSpan));cell.removeAttribute('data-colwidth');
    for(let offset=0;offset<cell.colSpan;offset++)occupied[column+offset]=cell.rowSpan;
    column+=cell.colSpan;
   }
   for(let index=0;index<occupied.length;index++)occupied[index]=Math.max(0,occupied[index]!-1);
  });
  if(overflow){const paragraph=root.ownerDocument.createElement('p');paragraph.textContent=rows.map(row=>Array.from(row.cells).map(cell=>cell.textContent??'').join('\t')).join('\n');table.replaceWith(paragraph);}
 });
}

import {countEditorText,type EditorCounts} from './editorCounts.ts';
import {Plugin,TextSelection,AllSelection,type Command,type EditorState} from 'prosemirror-state';
import {setBlockType} from 'prosemirror-commands';
import type {Mark,Node as PMNode} from 'prosemirror-model';
import {TableMap} from 'prosemirror-tables';
import {TABLE_LIMITS,safeColor} from './richFormatting.ts';
export function setParagraphAttributes(attributes:Record<string,unknown>):Command {
 return (state,dispatch)=>{let changed=false;const tr=state.tr;state.doc.nodesBetween(state.selection.from,state.selection.to,(node,pos)=>{if(['paragraph','heading'].includes(node.type.name)){changed=true;tr.setNodeMarkup(pos,undefined,{...node.attrs,...attributes});return false;}return true;});if(changed&&dispatch)dispatch(tr.scrollIntoView());return changed;};
}
export function paragraphStyle(name:string):Command {
 return (state,dispatch)=>{
  const heading=/^heading([1-6])$/.exec(name);
  if(heading)return setBlockType(state.schema.nodes['heading']!,{level:Number(heading[1])})(state,dispatch);
  if(!['normal','no-spacing','title','subtitle'].includes(name))return false;
  return setBlockType(state.schema.nodes['paragraph']!,name==='normal'?{}:{preset:name,...(name==='no-spacing'?{before:0,after:0,lineHeight:'1'}:{})})(state,dispatch);
 };
}
export function applyColor(name:'textColor'|'highlight',value:string):Command {
 return (state,dispatch)=>{const type=state.schema.marks[name];if(!type)return false;const color=safeColor(value),tr=state.tr;
  if(state.selection.empty){tr.removeStoredMark(type);if(color)tr.addStoredMark(type.create({value:color}));}
  else {tr.removeMark(state.selection.from,state.selection.to,type);if(color)tr.addMark(state.selection.from,state.selection.to,type.create({value:color}));}
  dispatch?.(tr);return true;
 };
}
export function tablesWithinLimits(doc:PMNode):boolean {
 let count=0,valid=true;
 doc.descendants(node=>{
  if(node.type.spec.tableRole!=='table')return true;
  count++;if(count>TABLE_LIMITS.tables||node.childCount>TABLE_LIMITS.rows){valid=false;return false;}
  node.descendants(child=>{if(child.type.spec.tableRole==='table')valid=false;if(child.type.spec.tableRole==='row'&&child.childCount>TABLE_LIMITS.columns)valid=false;
   if(['cell','header_cell'].includes(child.type.spec.tableRole??'')&&(!Number.isInteger(child.attrs['colspan'])||child.attrs['colspan']<1||child.attrs['colspan']>TABLE_LIMITS.columns||!Number.isInteger(child.attrs['rowspan'])||child.attrs['rowspan']<1||child.attrs['rowspan']>TABLE_LIMITS.rows))valid=false;return true;});
  if(valid){try{const map=TableMap.get(node);if(map.width>TABLE_LIMITS.columns||map.height>TABLE_LIMITS.rows||map.width*map.height>TABLE_LIMITS.cells)valid=false;}catch{valid=false;}}
  return false;
 });return valid;
}
export function tableLimitPlugin(onRefused:()=>void=()=>undefined):Plugin {return new Plugin({filterTransaction:transaction=>{if(!transaction.docChanged||tablesWithinLimits(transaction.doc))return true;onRefused();return false;}});}
export function insertTable(rows:number,columns:number):Command {
 return (state,dispatch)=>{
  if(!Number.isInteger(rows)||!Number.isInteger(columns)||rows<1||columns<1||rows>TABLE_LIMITS.rows||columns>TABLE_LIMITS.columns||rows*columns>TABLE_LIMITS.cells)return false;
  for(let depth=state.selection.$from.depth;depth>0;depth--)if(state.selection.$from.node(depth).type.spec.tableRole==='table')return false;
  const {table,table_row:row,table_cell:cell,paragraph}=state.schema.nodes;if(!table||!row||!cell||!paragraph)return false;
  const node=table.create(null,Array.from({length:rows},()=>row.create(null,Array.from({length:columns},()=>cell.create(null,paragraph.create())))));
  const tr=state.tr.replaceSelectionWith(node);if(tr.doc.lastChild?.type===table)tr.insert(tr.doc.content.size,paragraph.create());let inserted:number|undefined;tr.doc.descendants((candidate,pos)=>{if(candidate===node)inserted=pos;return candidate!==node;});if(inserted!==undefined)tr.setSelection(TextSelection.create(tr.doc,inserted+4));if(!tablesWithinLimits(tr.doc))return false;dispatch?.(tr.scrollIntoView());return true;
 };
}
export interface LiteralMatch {from:number;to:number}
export function literalMatches(doc:PMNode,needle:string,caseSensitive=false):LiteralMatch[] {
 if(!needle||needle.length>256)return [];const result:LiteralMatch[]=[];
 const escaped=needle.replace(/[.*+?^${}()|[\]\\]/g,'\\$&');
 doc.descendants((block,pos)=>{
  if(!block.isTextblock)return true;let text='';const positions:number[]=[];
  block.forEach((node,offset)=>{const value=node.isText?node.text!:node.type.name==='hard_break'?'\n':'\ufffc';for(let i=0;i<value.length;i++){text+=value[i];positions.push(pos+1+offset+i);}});
  for(const match of text.matchAll(new RegExp(escaped,caseSensitive?'gu':'giu'))){if(result.length>=1000)break;const at=match.index;result.push({from:positions[at]!,to:positions[at+match[0].length-1]!+1});}
  return false;
 });return result;
}

export function selectLiteral(needle:string,direction:1|-1,caseSensitive=false):Command {
 return (state,dispatch)=>{const matches=literalMatches(state.doc,needle,caseSensitive);if(!matches.length)return false;const match=direction===1?matches.find(value=>value.from>=state.selection.to)??matches[0]!:[...matches].reverse().find(value=>value.to<=state.selection.from)??matches[matches.length-1]!;dispatch?.(state.tr.setSelection(TextSelection.create(state.doc,match.from,match.to)).scrollIntoView());return true;};
}
export function replaceLiteral(needle:string,replacement:string,all:boolean,caseSensitive=false):Command {
 return (state,dispatch)=>{if(replacement.length>4096)return false;let matches=literalMatches(state.doc,needle,caseSensitive);if(!all)matches=matches.filter(match=>match.from===state.selection.from&&match.to===state.selection.to);if(!matches.length)return false;
  const tr=state.tr;for(const match of matches.reverse()){const marks=state.doc.nodeAt(match.from)?.marks??state.doc.resolve(match.from).marks();if(replacement)tr.replaceWith(match.from,match.to,state.schema.text(replacement,marks));else tr.delete(match.from,match.to);}dispatch?.(tr.scrollIntoView());return true;
 };
}
export function selectDocument():Command {return (state,dispatch)=>{dispatch?.(state.tr.setSelection(new AllSelection(state.doc)));return true;};}
export function changeCase(mode:'lower'|'upper'|'title'|'sentence'):Command {
 return (state,dispatch)=>{if(state.selection.empty)return false;const ranges:{from:number;to:number;text:string;marks:readonly Mark[]}[]=[];let sentence=true,wordStart=true;
 state.doc.nodesBetween(state.selection.from,state.selection.to,(node,pos)=>{if(!node.isText){if(node.isTextblock){sentence=true;wordStart=true;}return true;}const from=Math.max(pos,state.selection.from),to=Math.min(pos+node.nodeSize,state.selection.to);const original=node.text!.slice(from-pos,to-pos);let text=original;
 if(mode==='upper')text=original.toUpperCase();else if(mode==='lower')text=original.toLowerCase();else if(mode==='title')text=[...original.toLowerCase()].map(character=>{if(/[\s-]/.test(character)){wordStart=true;return character;}if(/[\p{L}\p{N}]/u.test(character)){const result=wordStart?character.toUpperCase():character;wordStart=false;return result;}return character;}).join('');else text=[...original.toLowerCase()].map(character=>{if(/[.!?]/.test(character))sentence=true;if(/\p{L}/u.test(character)&&sentence){sentence=false;return character.toUpperCase();}return character;}).join('');
 ranges.push({from,to,text,marks:node.marks});return false;});const tr=state.tr;for(const range of ranges.reverse())tr.replaceWith(range.from,range.to,state.schema.text(range.text,range.marks));dispatch?.(tr);return true;
 };
}
export function sortSelectedBlocks(numeric:boolean,descending=false):Command {
 return (state,dispatch)=>{const blocks:{node:PMNode;pos:number}[]=[];state.doc.nodesBetween(state.selection.from,state.selection.to,(node,pos,parent)=>{if(parent===state.doc){if(!['paragraph','heading'].includes(node.type.name))return false;blocks.push({node,pos});return false;}return true;});if(blocks.length<2||blocks.length>1000)return false;
  const first=blocks[0]!,last=blocks[blocks.length-1]!;if(blocks.some((block,index)=>index>0&&blocks[index-1]!.pos+blocks[index-1]!.node.nodeSize!==block.pos))return false;
  const collator=new Intl.Collator(undefined,{numeric,sensitivity:'base'}),sorted=[...blocks].sort((a,b)=>(descending?-1:1)*collator.compare(a.node.textContent,b.node.textContent));
  dispatch?.(state.tr.replaceWith(first.pos,last.pos+last.node.nodeSize,sorted.map(block=>block.node)));return true;
 };
}
export function documentCounts(doc:PMNode):EditorCounts {
 const text=doc.textBetween(0,doc.content.size,'\n','\ufffc');let paragraphs=0;doc.descendants(node=>{if(node.isTextblock)paragraphs++;return true;});
 return countEditorText(text,paragraphs);
}
export function selectedTable(state:EditorState):PMNode|null {for(let depth=state.selection.$from.depth;depth>0;depth--){const node=state.selection.$from.node(depth);if(node.type.spec.tableRole==='table')return node;}return null;}

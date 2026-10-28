import {createSignal,For,Show,type JSX} from 'solid-js';
import type {EditorState,Command} from 'prosemirror-state';
import {TextSelection} from 'prosemirror-state';
import {DOMSerializer,Slice,type Mark} from 'prosemirror-model';
import type {EditorView} from 'prosemirror-view';
import {addRowBefore,addRowAfter,deleteRow,addColumnBefore,addColumnAfter,deleteColumn,deleteTable,mergeCells,splitCell,toggleHeaderRow,setCellAttr} from 'prosemirror-tables';
import {applyColor,changeCase,insertTable,literalMatches,paragraphStyle,replaceLiteral,selectDocument,selectLiteral,setParagraphAttributes,sortSelectedBlocks} from './richCommands.ts';
import {LINE_HEIGHTS,PARAGRAPH_SPACING,TABLE_LIMITS} from './richFormatting.ts';
import {docFromHtml} from './richtext.ts';
import {Icon,type IconName} from '../Icon.tsx';
import {RibbonPopover} from './ribbonParts.tsx';
import {t} from '../../i18n/index.ts';

// The editor's clipboard, colour, paragraph, find, and table commands (t38-e5:
// one factory of controls, which RichTextEditor lays out in 3-row ribbon
// groups). Each control is a function, so a ribbon can mount it once per tab.
// State (find text, table size, the copied format) lives in the factory, so it
// survives a tab switch.
export interface AdvancedEditorToolsProps {state:()=>EditorState|null;view:()=>EditorView|undefined;run:(command:Command)=>void;owned:()=>ReadonlySet<string>|undefined;report:(message:string)=>void;markers:()=>boolean;onMarkers:()=>void}
const tableIcons:Record<string,IconName>={'row-before':'tableRowBefore','row-after':'tableRowAfter','row-delete':'tableRowDelete','column-before':'tableColumnBefore','column-after':'tableColumnAfter','column-delete':'tableColumnDelete',merge:'tableMerge',split:'tableSplit',header:'tableHeader',delete:'delete'};
const tableActions:[string,Command][]=[['row-before',addRowBefore],['row-after',addRowAfter],['row-delete',deleteRow],['column-before',addColumnBefore],['column-after',addColumnAfter],['column-delete',deleteColumn],['merge',mergeCells],['split',splitCell],['header',toggleHeaderRow],['delete',deleteTable]];
export type AdvancedEditorTools=ReturnType<typeof createAdvancedEditorTools>;
export function createAdvancedEditorTools(props:AdvancedEditorToolsProps) {
 const [needle,setNeedle]=createSignal(''),[replacement,setReplacement]=createSignal(''),[caseSensitive,setCaseSensitive]=createSignal(false);
 const [rows,setRows]=createSignal(2),[columns,setColumns]=createSignal(2);
 const [paint,setPaint]=createSignal<{marks:readonly Mark[];attributes:Record<string,unknown>}|null>(null);
 const matches=()=>{const state=props.state();return state?literalMatches(state.doc,needle(),caseSensitive()).length:0;};
 const execute=(command:Command):void=>{const state=props.state();if(!state||!command(state)){props.report(t('mail-editor-selection-required'));return;}props.report('');props.run(command);};
 const attrs=()=>props.state()?.selection.$from.parent.attrs??{};
 const paintFormat=():void=>{const snapshot=paint();if(!snapshot)return;execute((state,dispatch)=>{if(state.selection.empty)return false;let tr=state.tr.removeMark(state.selection.from,state.selection.to);for(const mark of snapshot.marks)tr=tr.addMark(state.selection.from,state.selection.to,mark);state.doc.nodesBetween(state.selection.from,state.selection.to,(node,pos)=>{if(['paragraph','heading'].includes(node.type.name))tr.setNodeMarkup(pos,undefined,{...node.attrs,...snapshot.attributes});return true;});dispatch?.(tr);return true;});};
 async function clipboard(action:'copy'|'cut'|'paste'):Promise<void>{
  const view=props.view();if(!view)return;const state=view.state,clipboard=view.dom.ownerDocument.defaultView?.navigator.clipboard;
  if(!clipboard){props.report(t('mail-editor-clipboard-unavailable'));return;}
  try{
   if(action==='paste'){
    let html:string|undefined,text:string|undefined;
    if(clipboard.read){const items=await clipboard.read();const item=items.find(value=>value.types.includes('text/html')||value.types.includes('text/plain'));if(!item)throw new Error('No text');const type=item.types.includes('text/html')?'text/html':'text/plain',blob=await item.getType(type);if(blob.size>786432)throw new Error('Too large');const value=await blob.text();if(type==='text/html')html=value;else text=value;}
    else {text=await clipboard.readText();if(text.length>786432)throw new Error('Too large');}
    if(props.view()!==view||view.state!==state){props.report(t('mail-editor-selection-changed'));return;}
    const tr=html===undefined?state.tr.insertText(text??''):state.tr.replaceSelection(new Slice(docFromHtml(html,props.owned()).content,0,0));view.dispatch(tr);view.focus();
   }else{
    if(state.selection.empty){props.report(t('mail-editor-selection-required'));return;}
    const text=state.doc.textBetween(state.selection.from,state.selection.to,'\n'),container=view.dom.ownerDocument.createElement('div');container.appendChild(DOMSerializer.fromSchema(state.schema).serializeFragment(state.selection.content().content,{document:view.dom.ownerDocument}));
    if(clipboard.write&&typeof ClipboardItem!=='undefined')await clipboard.write([new ClipboardItem({'text/plain':new Blob([text],{type:'text/plain'}),'text/html':new Blob([container.innerHTML],{type:'text/html'})})]);else await clipboard.writeText(text);
    if(action==='cut'){if(props.view()!==view||view.state!==state){props.report(t('mail-editor-selection-changed'));return;}view.dispatch(state.tr.deleteSelection());view.focus();}
   }
   props.report('');
  }catch{props.report(t('mail-editor-clipboard-unavailable'));}
 }
 const actionIcon=(key:string):IconName=>key.startsWith('clear-')?'clearFormat':key==='table-insert'?'table':key==='previous'?'back':key==='next'?'forward':key==='select-all'?'check':'edit';
 const button=(key:string,command:Command,icon?:IconName,iconOnly=false):JSX.Element=><button type="button" class="compose-rt__btn" onMouseDown={event=>event.preventDefault()} aria-label={iconOnly?t(`mail-editor-${key}`):undefined} data-tooltip={t(`mail-editor-${key}`)} onClick={()=>execute(command)}><Icon name={icon??actionIcon(key)}/><Show when={!iconOnly}><span class="compose-rib__text">{t(`mail-editor-${key}`)}</span></Show></button>;
 const clip=(action:'copy'|'cut'|'paste',large=false):JSX.Element=><button type="button" class={`compose-rt__btn${large?' compose-rib__large':''}`} onMouseDown={event=>event.preventDefault()} data-tooltip={t(`mail-editor-${action}`)} onClick={()=>void clipboard(action)}><Icon name={action==='copy'?'clone':action}/><span class="compose-rib__text">{t(`mail-editor-${action}`)}</span></button>;
 // The format painter: copy, then apply (icon-only; the name is the label and the tooltip).
 const painter=(key:'copy-format'|'apply-format',onClick:()=>void,disabled:()=>boolean):JSX.Element=><button type="button" class="compose-rt__btn" disabled={disabled()} onMouseDown={event=>event.preventDefault()} onClick={onClick} aria-label={t(`mail-editor-${key}`)} data-tooltip={t(`mail-editor-${key}`)}><Icon name="paint"/><Show when={key==='apply-format'}><Icon name="check"/></Show></button>;
 const color=(key:'text-color'|'highlight'|'shading'|'table-shading',onColor:(value:string)=>void,initial?:string):JSX.Element=><label class="compose-rib__color" data-tooltip={t(`mail-editor-${key}`)}><Icon name="paint"/><input type="color" value={initial??'#000000'} aria-label={t(`mail-editor-${key}`)} onInput={event=>onColor(event.currentTarget.value)}/></label>;
 return {
  paste:()=>clip('paste',true),
  cut:()=>clip('cut'),
  copy:()=>clip('copy'),
  copyFormat:()=>painter('copy-format',()=>{const state=props.state();if(state)setPaint({marks:state.storedMarks??state.selection.$from.marks(),attributes:{...state.selection.$from.parent.attrs}});},()=>false),
  applyFormat:()=>painter('apply-format',paintFormat,()=>!paint()),
  style:()=><select aria-label={t('mail-editor-style')} data-tooltip={t('mail-editor-style')} value={attrs()['preset']??(props.state()?.selection.$from.parent.type.name==='heading'?`heading${attrs()['level']}`:'normal')} onChange={event=>execute(paragraphStyle(event.currentTarget.value))}><For each={['normal','no-spacing','title','subtitle','heading1','heading2','heading3','heading4','heading5','heading6']}>{name=><option value={name}>{t(`mail-editor-style-${name}`)}</option>}</For></select>,
  textColor:()=>color('text-color',value=>execute(applyColor('textColor',value))),
  clearColor:()=>button('clear-color',applyColor('textColor',''),undefined,true),
  highlight:()=>color('highlight',value=>execute(applyColor('highlight',value)),'#ffff00'),
  clearHighlight:()=>button('clear-highlight',applyColor('highlight',''),undefined,true),
  changeCase:()=><select aria-label={t('mail-editor-case')} data-tooltip={t('mail-editor-case')} value="" onChange={event=>{const value=event.currentTarget.value;if(['lower','upper','title','sentence'].includes(value))execute(changeCase(value as 'lower'|'upper'|'title'|'sentence'));event.currentTarget.value='';}}><option value="">{t('mail-editor-case')}</option><For each={['lower','upper','title','sentence']}>{value=><option value={value}>{t(`mail-editor-case-${value}`)}</option>}</For></select>,
  sort:()=><select aria-label={t('mail-editor-sort')} data-tooltip={t('mail-editor-sort')} value="" onChange={event=>{if(event.currentTarget.value)execute(sortSelectedBlocks(event.currentTarget.value==='numeric'));event.currentTarget.value='';}}><option value="">{t('mail-editor-sort')}</option><option value="alpha">{t('mail-editor-sort-alpha')}</option><option value="numeric">{t('mail-editor-sort-numeric')}</option></select>,
  markers:()=><button type="button" class="compose-rt__btn" aria-pressed={props.markers()} onClick={props.onMarkers} aria-label={t('mail-editor-markers')} data-tooltip={t('mail-editor-markers')}><Icon name="paragraph"/></button>,
  paragraphOptions:()=><RibbonPopover label={t('mail-editor-paragraph')} icon="preferences" iconOnly keepSelection><div class="compose-rt__options compose-rib__form">
    <label>{t('mail-editor-alignment')}<select aria-label={t('mail-editor-alignment')} value={attrs()['align']??'left'} onChange={event=>execute(setParagraphAttributes({align:event.currentTarget.value}))}><For each={['left','center','right','justify']}>{value=><option value={value}>{t(`mail-editor-align-${value}`)}</option>}</For></select></label>
    <label>{t('mail-editor-line-height')}<select aria-label={t('mail-editor-line-height')} value={attrs()['lineHeight']??''} onChange={event=>execute(setParagraphAttributes({lineHeight:event.currentTarget.value||null}))}><option value="">{t('mail-compose-font-inherit')}</option><For each={LINE_HEIGHTS}>{value=><option>{value}</option>}</For></select></label>
    <For each={['before','after'] as const}>{side=><label>{t(`mail-editor-space-${side}`)}<select aria-label={t(`mail-editor-space-${side}`)} value={attrs()[side]??''} onChange={event=>execute(setParagraphAttributes({[side]:event.currentTarget.value===''?null:Number(event.currentTarget.value)}))}><option value="">{t('mail-compose-font-inherit')}</option><For each={PARAGRAPH_SPACING}>{value=><option value={value}>{value} pt</option>}</For></select></label>}</For>
    <label>{t('mail-editor-indent')}<select aria-label={t('mail-editor-indent')} value={attrs()['indent']??0} onChange={event=>execute(setParagraphAttributes({indent:Number(event.currentTarget.value)}))}><For each={[0,1,2,3,4,5,6]}>{value=><option value={value}>{value}</option>}</For></select></label>
    <label>{t('mail-editor-shading')}<input type="color" aria-label={t('mail-editor-shading')} onInput={event=>execute(setParagraphAttributes({shade:event.currentTarget.value}))}/></label>{button('clear-shading',setParagraphAttributes({shade:null}))}
   </div></RibbonPopover>,
  findReplace:()=><RibbonPopover label={t('mail-editor-find-replace')} icon="search" keepSelection><div class="compose-rt__options compose-rib__form"><label>{t('mail-editor-find')}<input maxlength="256" aria-label={t('mail-editor-find')} value={needle()} onInput={event=>setNeedle(event.currentTarget.value)}/></label><label>{t('mail-editor-replacement')}<input maxlength="4096" aria-label={t('mail-editor-replacement')} value={replacement()} onInput={event=>setReplacement(event.currentTarget.value)}/></label><label><input type="checkbox" checked={caseSensitive()} onChange={event=>setCaseSensitive(event.currentTarget.checked)}/>{t('mail-editor-match-case')}</label>{button('previous',(state,dispatch)=>selectLiteral(needle(),-1,caseSensitive())(state,dispatch))}{button('next',(state,dispatch)=>selectLiteral(needle(),1,caseSensitive())(state,dispatch))}{button('replace',(state,dispatch)=>replaceLiteral(needle(),replacement(),false,caseSensitive())(state,dispatch))}{button('replace-all',(state,dispatch)=>replaceLiteral(needle(),replacement(),true,caseSensitive())(state,dispatch))}<span>{t('mail-editor-matches',{count:matches()})}</span></div></RibbonPopover>,
  selectAll:()=>button('select-all',selectDocument()),
  clearSelection:()=>button('clear-selection',(state,dispatch)=>{dispatch?.(state.tr.setSelection(TextSelection.create(state.doc,state.selection.to)));return true;}),
  tableRows:()=><label class="compose-rib__field">{t('mail-editor-table-rows')}<input type="number" min="1" max={TABLE_LIMITS.rows} value={rows()} onInput={event=>setRows(Number(event.currentTarget.value))}/></label>,
  tableColumns:()=><label class="compose-rib__field">{t('mail-editor-table-columns')}<input type="number" min="1" max={TABLE_LIMITS.columns} value={columns()} onInput={event=>setColumns(Number(event.currentTarget.value))}/></label>,
  insertTable:()=><button type="button" class="compose-rt__btn" onMouseDown={event=>event.preventDefault()} data-tooltip={t('mail-editor-table-limits')} onClick={()=>execute((state,dispatch)=>insertTable(rows(),columns())(state,dispatch))}><Icon name="table"/><span class="compose-rib__text">{t('mail-editor-table-insert')}</span></button>,
  tableActionList:tableActions,
  tableAction:(name:string,command:Command)=><button type="button" class="compose-rt__btn" disabled={!props.state()||!command(props.state()!)} onMouseDown={event=>event.preventDefault()} data-tooltip={t(`mail-editor-table-${name}`)} onClick={()=>execute(command)}><Icon name={tableIcons[name]??'table'}/><span class="compose-rib__text">{t(`mail-editor-table-${name}`)}</span></button>,
  tableShading:()=>color('table-shading',value=>execute(setCellAttr('shade',value))),
  clearTableShading:()=>button('clear-shading',setCellAttr('shade',null)),
 };
}

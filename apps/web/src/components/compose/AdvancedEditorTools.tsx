import {createSignal,For,Show,type JSX} from 'solid-js';
import type {EditorState,Command} from 'prosemirror-state';
import {TextSelection} from 'prosemirror-state';
import {DOMSerializer,Slice,type Mark} from 'prosemirror-model';
import type {EditorView} from 'prosemirror-view';
import {addRowBefore,addRowAfter,deleteRow,addColumnBefore,addColumnAfter,deleteColumn,deleteTable,mergeCells,splitCell,toggleHeaderRow,setCellAttr} from 'prosemirror-tables';
import {applyColor,changeCase,insertTable,literalMatches,paragraphStyle,replaceLiteral,selectDocument,selectLiteral,setParagraphAttributes,sortSelectedBlocks} from './richCommands.ts';
import {LINE_HEIGHTS,PARAGRAPH_SPACING,TABLE_LIMITS} from './richFormatting.ts';
import {docFromHtml} from './richtext.ts';
import {t} from '../../i18n/index.ts';
interface Props {state:()=>EditorState|null;view:()=>EditorView|undefined;run:(command:Command)=>void;tab:()=> 'format'|'insert'|'table'|null;owned:()=>ReadonlySet<string>|undefined;report:(message:string)=>void}
const tableActions:[string,Command][]=[['row-before',addRowBefore],['row-after',addRowAfter],['row-delete',deleteRow],['column-before',addColumnBefore],['column-after',addColumnAfter],['column-delete',deleteColumn],['merge',mergeCells],['split',splitCell],['header',toggleHeaderRow],['delete',deleteTable]];
export function AdvancedEditorTools(props:Props):JSX.Element {
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
 const button=(key:string,command:Command):JSX.Element=><button type="button" class="compose-rt__btn" onMouseDown={event=>event.preventDefault()} onClick={()=>execute(command)}>{t(`mail-editor-${key}`)}</button>;
 return <Show when={props.tab()!==null}><div class="compose-rt__advanced" data-detached-local>
  <div class="compose-rt__group" aria-label={t('mail-editor-clipboard')}><For each={['copy','cut','paste'] as const}>{action=><button type="button" class="compose-rt__btn" onMouseDown={event=>event.preventDefault()} onClick={()=>void clipboard(action)}>{t(`mail-editor-${action}`)}</button>}</For>
   <button type="button" class="compose-rt__btn" onMouseDown={event=>event.preventDefault()} onClick={()=>{const state=props.state();if(state)setPaint({marks:state.storedMarks??state.selection.$from.marks(),attributes:{...state.selection.$from.parent.attrs}});}}>{t('mail-editor-copy-format')}</button><button type="button" class="compose-rt__btn" disabled={!paint()} onMouseDown={event=>event.preventDefault()} onClick={paintFormat}>{t('mail-editor-apply-format')}</button>
  </div>
  <Show when={props.tab()==='format'}><div class="compose-rt__group">
   <label>{t('mail-editor-style')}<select aria-label={t('mail-editor-style')} value={attrs()['preset']??(props.state()?.selection.$from.parent.type.name==='heading'?`heading${attrs()['level']}`:'normal')} onChange={event=>execute(paragraphStyle(event.currentTarget.value))}><For each={['normal','no-spacing','title','subtitle','heading1','heading2','heading3','heading4','heading5','heading6']}>{name=><option value={name}>{t(`mail-editor-style-${name}`)}</option>}</For></select></label>
   <label>{t('mail-editor-text-color')}<input type="color" aria-label={t('mail-editor-text-color')} onInput={event=>execute(applyColor('textColor',event.currentTarget.value))}/></label>{button('clear-color',applyColor('textColor',''))}
   <label>{t('mail-editor-highlight')}<input type="color" value="#ffff00" aria-label={t('mail-editor-highlight')} onInput={event=>execute(applyColor('highlight',event.currentTarget.value))}/></label>{button('clear-highlight',applyColor('highlight',''))}
   <details><summary>{t('mail-editor-paragraph')}</summary><div class="compose-rt__options">
    <label>{t('mail-editor-alignment')}<select aria-label={t('mail-editor-alignment')} value={attrs()['align']??'left'} onChange={event=>execute(setParagraphAttributes({align:event.currentTarget.value}))}><For each={['left','center','right','justify']}>{value=><option value={value}>{t(`mail-editor-align-${value}`)}</option>}</For></select></label>
    <label>{t('mail-editor-line-height')}<select aria-label={t('mail-editor-line-height')} value={attrs()['lineHeight']??''} onChange={event=>execute(setParagraphAttributes({lineHeight:event.currentTarget.value||null}))}><option value="">{t('mail-compose-font-inherit')}</option><For each={LINE_HEIGHTS}>{value=><option>{value}</option>}</For></select></label>
    <For each={['before','after'] as const}>{side=><label>{t(`mail-editor-space-${side}`)}<select aria-label={t(`mail-editor-space-${side}`)} value={attrs()[side]??''} onChange={event=>execute(setParagraphAttributes({[side]:event.currentTarget.value===''?null:Number(event.currentTarget.value)}))}><option value="">{t('mail-compose-font-inherit')}</option><For each={PARAGRAPH_SPACING}>{value=><option value={value}>{value} pt</option>}</For></select></label>}</For>
    <label>{t('mail-editor-indent')}<select aria-label={t('mail-editor-indent')} value={attrs()['indent']??0} onChange={event=>execute(setParagraphAttributes({indent:Number(event.currentTarget.value)}))}><For each={[0,1,2,3,4,5,6]}>{value=><option value={value}>{value}</option>}</For></select></label>
    <label>{t('mail-editor-shading')}<input type="color" aria-label={t('mail-editor-shading')} onInput={event=>execute(setParagraphAttributes({shade:event.currentTarget.value}))}/></label>{button('clear-shading',setParagraphAttributes({shade:null}))}
   </div></details>
   <label>{t('mail-editor-case')}<select aria-label={t('mail-editor-case')} value="" onChange={event=>{const value=event.currentTarget.value;if(['lower','upper','title','sentence'].includes(value))execute(changeCase(value as 'lower'|'upper'|'title'|'sentence'));event.currentTarget.value='';}}><option value="">{t('mail-editor-case')}</option><For each={['lower','upper','title','sentence']}>{value=><option value={value}>{t(`mail-editor-case-${value}`)}</option>}</For></select></label>
   <label>{t('mail-editor-sort')}<select aria-label={t('mail-editor-sort')} value="" onChange={event=>{if(event.currentTarget.value)execute(sortSelectedBlocks(event.currentTarget.value==='numeric'));event.currentTarget.value='';}}><option value="">{t('mail-editor-sort')}</option><option value="alpha">{t('mail-editor-sort-alpha')}</option><option value="numeric">{t('mail-editor-sort-numeric')}</option></select></label>
  </div></Show>
  <Show when={props.tab()==='insert'}><div class="compose-rt__group"><label>{t('mail-editor-table-rows')}<input type="number" min="1" max={TABLE_LIMITS.rows} value={rows()} onInput={event=>setRows(Number(event.currentTarget.value))}/></label><label>{t('mail-editor-table-columns')}<input type="number" min="1" max={TABLE_LIMITS.columns} value={columns()} onInput={event=>setColumns(Number(event.currentTarget.value))}/></label>{button('table-insert',(state,dispatch)=>insertTable(rows(),columns())(state,dispatch))}<small>{t('mail-editor-table-limits')}</small></div></Show>
  <Show when={props.tab()==='table'}><div class="compose-rt__group"><For each={tableActions}>{([name,command])=><button type="button" class="compose-rt__btn" disabled={!props.state()||!command(props.state()!)} onMouseDown={event=>event.preventDefault()} onClick={()=>execute(command)}>{t(`mail-editor-table-${name}`)}</button>}</For><label>{t('mail-editor-table-shading')}<input type="color" aria-label={t('mail-editor-table-shading')} onInput={event=>execute(setCellAttr('shade',event.currentTarget.value))}/></label>{button('clear-shading',setCellAttr('shade',null))}</div></Show>
  <details class="compose-rt__find"><summary>{t('mail-editor-find-replace')}</summary><div class="compose-rt__options"><label>{t('mail-editor-find')}<input maxlength="256" aria-label={t('mail-editor-find')} value={needle()} onInput={event=>setNeedle(event.currentTarget.value)}/></label><label>{t('mail-editor-replacement')}<input maxlength="4096" aria-label={t('mail-editor-replacement')} value={replacement()} onInput={event=>setReplacement(event.currentTarget.value)}/></label><label><input type="checkbox" checked={caseSensitive()} onChange={event=>setCaseSensitive(event.currentTarget.checked)}/>{t('mail-editor-match-case')}</label>{button('previous',(state,dispatch)=>selectLiteral(needle(),-1,caseSensitive())(state,dispatch))}{button('next',(state,dispatch)=>selectLiteral(needle(),1,caseSensitive())(state,dispatch))}{button('replace',(state,dispatch)=>replaceLiteral(needle(),replacement(),false,caseSensitive())(state,dispatch))}{button('replace-all',(state,dispatch)=>replaceLiteral(needle(),replacement(),true,caseSensitive())(state,dispatch))}<span>{t('mail-editor-matches',{count:matches()})}</span></div></details>
  <div class="compose-rt__group">{button('select-all',selectDocument())}{button('clear-selection',(state,dispatch)=>{dispatch?.(state.tr.setSelection(TextSelection.create(state.doc,state.selection.to)));return true;})}</div>
 </div></Show>;
}

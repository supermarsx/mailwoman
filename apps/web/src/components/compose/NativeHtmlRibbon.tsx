import {For,Show,createSignal,type JSX} from 'solid-js';
import {RibbonGroup,RibbonItem} from '../bars/index.ts';
import {Icon,type IconName} from '../Icon.tsx';
import {ColorField} from '../color/ColorPicker.tsx';
import {t,loadCatalog} from '../../i18n/index.ts';
import {COMPOSER_FONTS} from './fontCatalog.ts';
import {EDITOR_TABS,EditorRibbon,createEditorRibbon,type EditorSlots,type EditorTab} from './EditorRibbon.tsx';
import {InRibbonSlot,RibbonPopover} from './ribbonParts.tsx';
import type {RichTextApi} from './RichTextEditor.tsx';
import type {NativeHtmlCommands,NativeTableAction} from './nativeHtmlCommands.ts';
import type {NativeFormat} from './nativeFormatCommands.ts';
import {Feature,createFormatControls,type FormatAdapter,type Painter} from './FormatControls.tsx';
import type {PasteMode} from './formatModel.ts';
import * as a11y from '../mailA11y.css.ts';

const SIZES=[8,9,10,11,12,14,16,18,20,24,28,32,36,48,72];
const tableActions:[NativeTableAction,IconName][]=[['row-before','tableRowBefore'],['row-after','tableRowAfter'],['row-delete','tableRowDelete'],['column-before','tableColumnBefore'],['column-after','tableColumnAfter'],['column-delete','tableColumnDelete'],['merge','tableMerge'],['split','tableSplit'],['header','tableHeader'],['delete','delete']];
export function NativeHtmlRibbon(props:{commands:NativeHtmlCommands;version:()=>number;ready:()=>boolean;ribbonSlots?:()=>EditorSlots;externalHistory?:boolean;history:()=>RichTextApi|null;format:NativeFormat;painter:Painter;clipboard:(action:'copy'|'cut'|'paste',mode?:PasteMode)=>Promise<void>;images:(files:File[])=>void;canImages:boolean;report:(message:string)=>void;zoom:(value:number)=>void;view:()=>{dark:boolean;markers:boolean};toggleView:(kind:'dark'|'markers')=>void}):JSX.Element {
 void loadCatalog('color-picker');
 const own=props.ribbonSlots===undefined?createEditorRibbon():undefined;
 const slots=()=>props.ribbonSlots?.()??own?.slots()??{};
 const [rows,setRows]=createSignal(2),[columns,setColumns]=createSignal(2),[url,setUrl]=createSignal(''),[linkOpen,setLinkOpen]=createSignal(false);
 const [needle,setNeedle]=createSignal(''),[replacement,setReplacement]=createSignal(''),[matchCase,setMatchCase]=createSignal(false);
 const matchesCount=()=>{props.version();return props.commands.matches(needle(),matchCase()).length;};
 const active=(name:string)=>{props.version();return props.commands.state(name);};
 const computed=(property:string)=>{props.version();const element=props.commands.current();return element?element.ownerDocument.defaultView?.getComputedStyle(element).getPropertyValue(property)??'':'';};
 const fontValue=()=>{const value=computed('font-family'),first=(text:string)=>text.split(',')[0]?.replace(/["']/g,'').trim().toLowerCase();return COMPOSER_FONTS.find(font=>font.stack.toLowerCase()===value.toLowerCase()||first(font.stack)===first(value))?.stack??value;};
 const sizeValue=()=>{const value=computed('font-size'),px=/^([\d.]+)px$/.exec(value);return px?`${Math.round(Number(px[1])*.75*1000)/1000}pt`:value;};
 const colorValue=(property:string):string|null=>{const value=computed(property),rgb=/^rgba?\((\d+),\s*(\d+),\s*(\d+)/.exec(value);return /^#[a-f0-9]{6}$/i.test(value)?value:rgb?'#'+rgb.slice(1).map(channel=>Number(channel).toString(16).padStart(2,'0')).join(''):null;};
 const execute=(run:()=>boolean|void)=>{try{if(run()===false)props.report(t('mail-editor-selection-required'));else props.report('');}catch{props.report(t('mail-editor-selection-required'));}};
 const button=(key:string,icon:IconName,run:()=>boolean|void,options:{label?:string;testId?:string;disabled?:()=>boolean;pressed?:()=>boolean;large?:boolean;iconOnly?:boolean}={}):JSX.Element=><button type="button" class={`compose-rt__btn${options.large?' compose-rib__large':''}`} disabled={!props.ready()||options.disabled?.()} aria-label={options.label??t(`mail-editor-${key}`)} aria-pressed={options.pressed?.()} title={options.label??t(`mail-editor-${key}`)} data-testid={options.testId??`native-${key}`} onMouseDown={event=>event.preventDefault()} onClick={()=>execute(run)}><Icon name={icon}/><Show when={!options.iconOnly}><span class="compose-rib__text">{options.label??t(`mail-editor-${key}`)}</span></Show></button>;
 const row=(...children:JSX.Element[]) => <div class="compose-rib__row">{children}</div>;
 const mark=(name:string,icon:IconName,key=name)=>button(key,icon,()=>props.commands.exec(name),{label:t(`mail-compose-rt-${key}`),testId:`rt-${key}`,pressed:()=>active(name),iconOnly:true});
 const color=(key:'text-color'|'highlight'|'table-shading'|'shading',property:string,cell=false):JSX.Element=><ColorField variant="ribbon" keepSelection showValue={false} label={t(`mail-editor-${key}`)} value={colorValue(property)} fallback={key==='text-color'?'#000000':key==='highlight'?'#ffff00':'#eeeeee'} emptyLabel={t('color-picker-no-colour')} contrast={{against:key==='text-color'?'#ffffff':'#000000',use:'text',againstLabel:()=>t(key==='text-color'?'color-picker-against-white-message':'color-picker-against-black-text')}} onChange={value=>execute(()=>{if(cell)return props.commands.cellStyle(property,value??'');if(key==='shading')props.commands.blockStyle(property,value??'');else props.commands.inlineStyle(property,value??(key==='text-color'?'#1c1e21':'transparent'));})} testId={`editor-color-${key}`}/>;
 const style=(value:string):void=>{const heading=/^heading([1-6])$/.exec(value);props.commands.exec('formatBlock',heading?`h${heading[1]}`:'p');if(value==='title'||value==='subtitle')props.commands.blockStyle('font-size',value==='title'?'28pt':'18pt');else props.commands.blockStyle('font-size','');if(value==='no-spacing'){props.commands.blockStyle('margin-top','0');props.commands.blockStyle('margin-bottom','0');}};
 const block=():HTMLElement|null=>{props.version();return props.format.currentBlock();};
 const fmtAdapter:FormatAdapter={
  ready:props.ready,
  fontStep:(direction,fine)=>execute(()=>props.format.stepFont(direction,fine)),
  listType:()=>{props.version();return props.format.listType();},
  applyList:(kind,style)=>execute(()=>props.format.applyList(kind,style)),
  listStart:value=>execute(()=>props.format.listStart(value)),
  canContinue:()=>{props.version();return props.format.canContinue();},
  indent:direction=>execute(()=>props.format.indent(direction)),
  canOutdent:()=>{props.version();return props.format.canOutdent();},
  clear:variant=>execute(()=>props.format.clear(variant)),
  lineSpacing:value=>execute(()=>props.format.setBlock('line-height',value??'')),
  currentLineSpacing:()=>block()?.style.lineHeight||null,
  spacing:(side,points)=>execute(()=>props.format.setBlock(side==='before'?'margin-top':'margin-bottom',points===null?'':`${points}pt`)),
  align:value=>execute(()=>props.format.setBlock('text-align',value)),
  currentAlign:()=>block()?.style.textAlign||null,
  direction:value=>execute(()=>props.format.setDirection(value)),
  currentDirection:()=>block()?.getAttribute('dir')??null,
  paste:mode=>void props.clipboard('paste',mode),
  painter:props.painter,
 };
 const fmt=createFormatControls(fmtAdapter);
 const formatGroups=():JSX.Element=><>
  <RibbonGroup label={t('mail-compose-group-clipboard')} priority={100} icon="paste" testId="rt-group-clipboard">
   <RibbonItem size="large">{fmt.pasteLarge()}</RibbonItem><RibbonItem>{button('cut','cut',()=>{void props.clipboard('cut');})}</RibbonItem><RibbonItem>{button('copy','clone',()=>{void props.clipboard('copy');})}</RibbonItem>
  </RibbonGroup>
  <Feature id="format.painter"><RibbonGroup label={t('cfmt-group-painter')} priority={95} icon="brushCopy" testId="rt-group-painter">
   <RibbonItem>{fmt.copyFormat()}</RibbonItem><RibbonItem>{fmt.applyFormat()}</RibbonItem><RibbonItem>{fmt.formatChip()}</RibbonItem>
  </RibbonGroup></Feature>
  <RibbonGroup label={t('mail-compose-font')} priority={90} icon="bold" testId="rt-group-font">
   <RibbonItem>{row(<select class="compose-rib__font" aria-label={t('mail-compose-font')} disabled={!props.ready()} value={fontValue()} onChange={event=>props.commands.inlineStyle('font-family',event.currentTarget.value)}><option value="">{t('mail-compose-font-inherit')}</option><Show when={fontValue()&&!COMPOSER_FONTS.some(font=>font.stack===fontValue())}><option value={fontValue()}>{fontValue()}</option></Show><For each={COMPOSER_FONTS}>{font=><option value={font.stack}>{font.label}</option>}</For></select>,<select class="compose-rib__size" aria-label={t('mail-compose-font-size')} disabled={!props.ready()} value={sizeValue()} onChange={event=>props.commands.inlineStyle('font-size',event.currentTarget.value)}><option value="">{t('mail-compose-font-inherit')}</option><Show when={sizeValue()&&!SIZES.some(size=>`${size}pt`===sizeValue())}><option value={sizeValue()}>{sizeValue()}</option></Show><For each={SIZES}>{size=><option value={`${size}pt`}>{size} pt</option>}</For></select>,fmt.fontSteps())}</RibbonItem>
   <RibbonItem>{row(mark('bold','bold'),mark('italic','italic'),mark('underline','underline'),mark('strikeThrough','strike','strike'),<select aria-label={t('mail-editor-case')} value="" onChange={event=>{const value=event.currentTarget.value;if(['lower','upper','title','sentence'].includes(value))props.commands.changeCase(value as 'lower'|'upper'|'title'|'sentence');event.currentTarget.value='';}}><option value="">{t('mail-editor-case')}</option><option value="lower">{t('mail-editor-case-lower')}</option><option value="upper">{t('mail-editor-case-upper')}</option><option value="title">{t('mail-editor-case-title')}</option><option value="sentence">{t('mail-editor-case-sentence')}</option></select>,fmt.clearFormatting())}</RibbonItem>
   <RibbonItem>{row(color('text-color','color'),button('clear-color','clearFormat',()=>props.commands.inlineStyle('color','#1c1e21'),{iconOnly:true}),color('highlight','background-color'),button('clear-highlight','clearFormat',()=>props.commands.inlineStyle('background-color','transparent'),{iconOnly:true}))}</RibbonItem>
  </RibbonGroup>
  <RibbonGroup label={t('mail-compose-group-paragraph')} priority={80} icon="paragraph" testId="rt-group-paragraph">
   <RibbonItem>{row(fmt.bulletList(),fmt.numberList(),fmt.indentButtons(),button('quote','quote',()=>props.commands.exec('formatBlock','blockquote'),{label:t('mail-compose-rt-quote'),iconOnly:true}),<select aria-label={t('mail-editor-sort')} value="" onChange={event=>{if(event.currentTarget.value)execute(()=>props.commands.sortBlocks(event.currentTarget.value==='numeric'));event.currentTarget.value='';}}><option value="">{t('mail-editor-sort')}</option><option value="alpha">{t('mail-editor-sort-alpha')}</option><option value="numeric">{t('mail-editor-sort-numeric')}</option></select>)}</RibbonItem>
   <RibbonItem>{row(fmt.alignButtons(),fmt.lineSpacing(),fmt.direction())}</RibbonItem>
   <RibbonItem>{row(button('markers','paragraph',()=>props.toggleView('markers'),{pressed:()=>props.view().markers,iconOnly:true}),<RibbonPopover label={t('mail-editor-paragraph')} icon="preferences" keepSelection><div class="compose-rib__form">
    <label>{t('mail-editor-alignment')}<select aria-label={t('mail-editor-alignment')} onChange={event=>props.commands.blockStyle('text-align',event.currentTarget.value)}><For each={['left','center','right','justify']}>{align=><option value={align}>{t(`mail-editor-align-${align}`)}</option>}</For></select></label>
    <label>{t('mail-editor-line-height')}<select aria-label={t('mail-editor-line-height')} onChange={event=>props.commands.blockStyle('line-height',event.currentTarget.value)}><option value="">{t('mail-compose-font-inherit')}</option><For each={['1','1.15','1.5','2','2.5','3']}>{value=><option>{value}</option>}</For></select></label>
    <For each={['before','after'] as const}>{side=><label>{t(`mail-editor-space-${side}`)}<select aria-label={t(`mail-editor-space-${side}`)} onChange={event=>props.commands.blockStyle(side==='before'?'margin-top':'margin-bottom',`${event.currentTarget.value}pt`)}><For each={[0,6,12,18,24,36]}>{value=><option value={value}>{value} pt</option>}</For></select></label>}</For>
    <label>{t('mail-editor-indent')}<select aria-label={t('mail-editor-indent')} onChange={event=>props.commands.blockStyle('margin-left',`${Number(event.currentTarget.value)*40}px`)}><For each={[0,1,2,3,4,5,6]}>{value=><option>{value}</option>}</For></select></label>
    {color('shading','background-color')}{button('clear-shading','clearFormat',()=>props.commands.blockStyle('background-color',''),{testId:'native-clear-paragraph-shading'})}
   </div></RibbonPopover>)}</RibbonItem>
  </RibbonGroup>
  <RibbonGroup label={t('cfmt-group-styles')} priority={70} icon="heading" testId="rt-group-styles">
   <RibbonItem><select aria-label={t('mail-editor-style')} value="normal" onChange={event=>style(event.currentTarget.value)}><For each={['normal','no-spacing','title','subtitle','heading1','heading2','heading3','heading4','heading5','heading6']}>{name=><option value={name}>{t(`mail-editor-style-${name}`)}</option>}</For></select></RibbonItem>
  </RibbonGroup>
  <RibbonGroup label={t('mail-compose-group-editing')} priority={45} icon="search" testId="rt-group-editing">
   <RibbonItem><RibbonPopover label={t('mail-editor-find-replace')} icon="search" keepSelection><div class="compose-rib__form"><label>{t('mail-editor-find')}<input maxlength="256" value={needle()} onInput={event=>setNeedle(event.currentTarget.value)}/></label><label>{t('mail-editor-replacement')}<input maxlength="4096" value={replacement()} onInput={event=>setReplacement(event.currentTarget.value)}/></label><label><input type="checkbox" checked={matchCase()} onChange={event=>setMatchCase(event.currentTarget.checked)}/>{t('mail-editor-match-case')}</label>{button('previous','back',()=>props.commands.find(needle(),-1,matchCase()))}{button('next','forward',()=>props.commands.find(needle(),1,matchCase()))}{button('replace','edit',()=>props.commands.replace(needle(),replacement(),false,matchCase()))}{button('replace-all','edit',()=>props.commands.replace(needle(),replacement(),true,matchCase()))}<span>{t('mail-editor-matches',{count:matchesCount()})}</span></div></RibbonPopover></RibbonItem>
   <RibbonItem>{button('select-all','check',()=>props.commands.selectAll())}</RibbonItem><RibbonItem>{button('clear-selection','clearFormat',()=>props.commands.clearSelection())}</RibbonItem>
  </RibbonGroup>
  <RibbonGroup label={t('mail-compose-group-view')} priority={20} icon="eye" testId="rt-group-view"><RibbonItem>{button('dark-background','eye',()=>props.toggleView('dark'),{label:t('mail-compose-dark-background'),testId:'rt-dark-background',pressed:()=>props.view().dark})}</RibbonItem><RibbonItem><label>{t('mail-editor-zoom')}<select aria-label={t('mail-editor-zoom')} onChange={event=>props.zoom(Number(event.currentTarget.value))}><For each={[75,100,125,150,200]}>{value=><option value={value} selected={value===100}>{value}%</option>}</For></select></label></RibbonItem></RibbonGroup>
 </>;
 const insertGroups=():JSX.Element=><>
  <RibbonGroup label={t('mail-compose-ribbon-insert')} priority={50} icon="link" testId="rt-group-insert">
   <RibbonItem><RibbonPopover label={t('mail-compose-rt-link')} icon="link" keepSelection buttonTestId="rt-link" open={linkOpen()} onOpenChange={setLinkOpen}><div class="compose-rib__form"><label>{t('mail-compose-rt-link')}<input type="url" value={url()} onInput={event=>setUrl(event.currentTarget.value)} placeholder={t('mail-compose-rt-link-placeholder')}/></label>{button('link-apply','link',()=>{const result=props.commands.link(url());if(result)setLinkOpen(false);return result;},{label:t('mail-compose-rt-link-apply'),testId:'rt-link-apply'})}</div></RibbonPopover></RibbonItem>
   <Show when={props.canImages}><RibbonItem><label class={`compose-rt__btn ${a11y.focusable}`} data-testid="rt-image"><Icon name="image"/>{t('mail-compose-rt-image')}<input type="file" class={a11y.srOnly} accept="image/png,image/jpeg,image/gif,image/webp" multiple aria-label={t('mail-compose-rt-image')} onChange={event=>{props.images(Array.from(event.currentTarget.files??[]));event.currentTarget.value='';}}/></label></RibbonItem></Show>
   <RibbonItem>{button('rule','minus',()=>{const root=props.commands.current();if(root)props.commands.insertNode(root.ownerDocument.createElement('hr'));},{label:t('mail-compose-rt-rule'),testId:'rt-rule'})}</RibbonItem>
  </RibbonGroup>
  <RibbonGroup label={t('mail-compose-ribbon-table')} priority={30} icon="table" testId="rt-group-table-insert"><RibbonItem><label>{t('mail-editor-table-rows')}<input type="number" min="1" max="20" value={rows()} onInput={event=>setRows(Number(event.currentTarget.value))}/></label></RibbonItem><RibbonItem><label>{t('mail-editor-table-columns')}<input type="number" min="1" max="10" value={columns()} onInput={event=>setColumns(Number(event.currentTarget.value))}/></label></RibbonItem><RibbonItem>{button('table-insert','table',()=>props.commands.insertTable(rows(),columns()))}</RibbonItem></RibbonGroup>
 </>;
 const tableGroups=():JSX.Element=><><RibbonGroup label={t('mail-compose-ribbon-table')} priority={30} icon="table" testId="rt-group-table"><For each={tableActions}>{([name,icon])=><RibbonItem>{button(`table-${name}`,icon,()=>props.commands.table(name),{disabled:()=>{props.version();return !props.commands.canTable(name);}})}</RibbonItem>}</For></RibbonGroup><RibbonGroup label={t('mail-editor-table-shading')} priority={25} icon="paint" testId="rt-group-table-shading"><RibbonItem>{color('table-shading','background-color',true)}</RibbonItem><RibbonItem>{button('clear-shading','clearFormat',()=>props.commands.cellStyle('background-color',''))}</RibbonItem></RibbonGroup></>;
 const groups=(tab:EditorTab)=>tab==='format'?formatGroups():tab==='insert'?insertGroups():tableGroups();
 return <><Show when={own}>{state=><EditorRibbon state={state()} tabs={EDITOR_TABS.map(id=>({id}))} visible={()=>EDITOR_TABS} history={props.history} showHistory={()=>!props.externalHistory}/>}</Show><For each={EDITOR_TABS}>{tab=><Show when={slots()[tab]} keyed>{target=><InRibbonSlot target={target}>{groups(tab)}</InRibbonSlot>}</Show>}</For></>;
}

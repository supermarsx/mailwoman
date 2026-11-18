import {createSignal,lazy,Match,onCleanup,Show,Suspense,Switch,type JSX} from 'solid-js';
import {loadCatalog,t} from '../../i18n';
import {escapeHtml} from '../../viewers/sandbox.ts';
import {toast} from '../toast/index.ts';
import {InlineLoading} from '../InlineLoading.tsx';
import {BodyModeGroup,EditorRibbon,createEditorRibbon,type BodyMode} from './EditorRibbon.tsx';
import type {RichTextApi} from './RichTextEditor.tsx';
import {EditorCounts} from './EditorCounts.tsx';
import * as css from '../../screens/Settings/styles.css.ts';
import * as signature from '../../screens/Settings/signatureEditor.css.ts';

const RichEditor=lazy(()=>import('./RichTextEditor.tsx'));
const SourceEditor=lazy(()=>import('./HtmlSourceEditor.tsx'));
const NO_OWNED:ReadonlySet<string>=new Set();
const asHtml=(text:string)=>`<p>${escapeHtml(text).replace(/\r\n?|\n/g,'<br>')}</p>`;

/** The same composing controls as signatures, without message delivery fields. */
export function TemplateBodyEditor(props:{text:string;html?:string;label:string;id:string;disabled?:boolean|undefined;maxText?:number;maxHtml?:number;onChange:(text:string,html:string)=>void}):JSX.Element{
 void loadCatalog('settings');void loadCatalog('mail');
 const [mode,setMode]=createSignal<BodyMode>('rich');
 const [api,setApi]=createSignal<RichTextApi|null>(null);
 const ribbon=createEditorRibbon<never>();
 let live=true,settling=true;onCleanup(()=>{live=false;});
 const html=()=>props.html||asHtml(props.text);
 const publish=(markup:string,text:string)=>{if(live&&!props.disabled)props.onChange(text,markup);};
 function changeMode(next:BodyMode):void{
  if(props.disabled)return;
  if(next==='plain')publish('',props.text);
  settling=true;setApi(null);setMode(next);
 }
 const edited=(markup:string,text:string)=>{if(!settling)publish(markup,text);};
 const ready=(value:RichTextApi)=>{setApi(()=>value);const done=()=>{if(live&&api()===value)settling=false;};if(value.whenReady)void value.whenReady().then(done,done);else done();};
 const embedded={fits:(bytes:number)=>new TextEncoder().encode(html()).length+bytes<=(props.maxHtml??32000),
  refused:(reason:'type'|'size',name:string)=>toast.error(t(reason==='type'?'settings-sig-image-type':'settings-sig-image-size',{name}))};
 return <div aria-disabled={props.disabled}>
  <Show when={!props.disabled} fallback={<textarea class={css.textarea} aria-label={props.label} disabled value={props.text}/> }>
   <EditorRibbon state={ribbon} visible={()=>mode()==='plain'?['format']:['format','insert','table']} history={api} showHistory={()=>mode()!=='plain'}
    tabs={[{id:'format',after:()=><BodyModeGroup label={t('settings-sig-format')} value={mode} onChange={changeMode} testId={`${props.id}-group-format`} selectTestId={`${props.id}-format`}/>},{id:'insert'},{id:'table'}]}/>
   <div class={signature.body}>
    <Switch>
     <Match when={mode()==='plain'}><textarea class={css.textarea} aria-label={props.label} maxLength={props.maxText??8000} value={props.text} onInput={event=>publish('',event.currentTarget.value)}/></Match>
     <Match when={mode()==='rich'}><Suspense fallback={<InlineLoading/>}><RichEditor externalHistory ribbonSlots={ribbon.slots} initialHtml={html()} ariaLabel={props.label} embedded={embedded} onChange={edited} onReady={ready}/></Suspense></Match>
     <Match when={mode()==='html'}><Suspense fallback={<InlineLoading/>}><SourceEditor externalHistory htmlLimit={props.maxHtml??32000} ribbonSlots={ribbon.slots} initialHtml={html()} owned={()=>NO_OWNED} embedded onChange={edited} onReady={ready}/></Suspense></Match>
    </Switch>
   </div>
   <EditorCounts text={props.text} html={props.html||props.text}/>
  </Show>
 </div>;
}

import {EditorCounts} from './EditorCounts.tsx';
import {createEffect,createSignal,onMount,type JSX} from 'solid-js';
import {docFromHtml,docFromText,htmlFromDoc,textFromDoc} from './richtext.ts';
import type {RichTextApi} from './RichTextEditor.tsx';
import {t} from '../../i18n/index.ts';
/** Source editing uses the same output schema as the visual editor. Never renders raw HTML; `onChange` also gives the source as typed.
 * `externalText` (the composer's plain body) is reconciled in; `embedded` keeps raster `data:` images (signatures). */
export default function HtmlSourceEditor(props:{initialHtml:string;externalText?:()=>string;owned:()=>ReadonlySet<string>;embedded?:boolean;onChange:(html:string,text:string,source:string)=>void;onReady:(api:RichTextApi)=>void}):JSX.Element {
 const [source,setSource]=createSignal(props.initialHtml);const [text,setText]=createSignal('');let field!:HTMLTextAreaElement;let lastText=props.externalText?.()??'';
 function publish(value:string):void{const doc=docFromHtml(value,props.owned(),props.embedded===true);lastText=textFromDoc(doc);setText(lastText);props.onChange(htmlFromDoc(doc),lastText,value);}
 function replace(value:string):void{setSource(value);publish(value);}
 onMount(()=>{publish(source());props.onReady({setHtml:replace,appendHtml:value=>replace(source()+value),focus:()=>field.focus(),refreshImages:()=>undefined});});
 createEffect(()=>{const external=props.externalText;if(external===undefined)return;const value=external();if(value!==lastText)replace(htmlFromDoc(docFromText(value)));});
 return <div class="compose-source"><p>{t('mail-compose-html-help')}</p><textarea ref={field} aria-label={t('mail-compose-html-source')} spellcheck={false} maxlength={786432} value={source()} onInput={event=>replace(event.currentTarget.value)}/><EditorCounts text={props.externalText?.()??text()}/></div>;
}

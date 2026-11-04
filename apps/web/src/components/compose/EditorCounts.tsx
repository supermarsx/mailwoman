import {createMemo,type JSX} from 'solid-js';
import {t} from '../../i18n/index.ts';
import {formatNumber} from '../../i18n/format.ts';
import {countEditorText,editorBodyBytes,type EditorCounts as Counts} from './editorCounts.ts';
export function EditorCounts(props:{text?:string;html?:string;paragraphs?:number;counts?:Counts}):JSX.Element {
 const counts=createMemo(()=>props.counts??countEditorText(props.text??'',props.paragraphs));
 const bytes=createMemo(()=>editorBodyBytes(props.html??props.text??''));
 const size=()=>t('mail-editor-body-size',{size:formatNumber(bytes()/1024,{minimumFractionDigits:1,maximumFractionDigits:1})});
 return <div class="compose-editor-counts" aria-label={t('mail-editor-counts')}><span>{t('mail-editor-words',{count:counts().words})}</span><span>{t('mail-editor-paragraphs',{count:counts().paragraphs})}</span><span>{t('mail-editor-characters',{count:counts().characters})}</span><span data-testid="editor-body-size" aria-label={`${t('mail-editor-body-size-label')}: ${size()}`} title={t('mail-editor-body-size-help')}>{size()}</span></div>;
}

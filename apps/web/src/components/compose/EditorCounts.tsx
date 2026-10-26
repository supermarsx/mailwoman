import {createMemo,type JSX} from 'solid-js';
import {t} from '../../i18n/index.ts';
import {countEditorText,type EditorCounts as Counts} from './editorCounts.ts';
export function EditorCounts(props:{text?:string;paragraphs?:number;counts?:Counts}):JSX.Element {
 const counts=createMemo(()=>props.counts??countEditorText(props.text??'',props.paragraphs));
 return <div class="compose-editor-counts" aria-label={t('mail-editor-counts')}><span>{t('mail-editor-words',{count:counts().words})}</span><span>{t('mail-editor-paragraphs',{count:counts().paragraphs})}</span><span>{t('mail-editor-characters',{count:counts().characters})}</span></div>;
}

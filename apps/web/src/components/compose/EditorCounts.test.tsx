import {expect,it} from 'vitest';
import {createSignal} from 'solid-js';
import {render,screen} from '@solidjs/testing-library';
import {EditorCounts} from './EditorCounts.tsx';
import {editorBodyBytes} from './editorCounts.ts';
import {t} from '../../i18n/index.ts';
import {formatNumber} from '../../i18n/format.ts';
it('measures UTF-8 bytes rather than UTF-16 length, including HTML and embedded image data',()=>{
 expect(editorBodyBytes('Aé🙂')).toBe(7);expect(editorBodyBytes('👩🏽‍💻')).toBe(15);
 const image='<img src="data:image/png;base64,'+'A'.repeat(2048)+'">';expect(editorBodyBytes(image)).toBe(2082);
});
it('updates the localized body size from text or actual HTML while keeping visible text counts separate',()=>{
 const [html,setHtml]=createSignal<string|undefined>();render(()=><EditorCounts text={'é'.repeat(1024)} {...(html()===undefined?{}:{html:html()!})}/>);
 const size=screen.getByTestId('editor-body-size');expect(size).toHaveTextContent(t('mail-editor-body-size',{size:formatNumber(2,{minimumFractionDigits:1,maximumFractionDigits:1})}));expect(size).toHaveAttribute('title',t('mail-editor-body-size-help'));
 setHtml('<p>Short</p><img src="data:image/png;base64,'+'A'.repeat(4096)+'">');expect(size).toHaveTextContent(t('mail-editor-body-size',{size:formatNumber((4096+46)/1024,{minimumFractionDigits:1,maximumFractionDigits:1})}));
});

import {beforeAll,expect,it,vi} from 'vitest';
import {fireEvent,render,screen} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {initSync} from '../../wasm/mw-sanitize/mw_sanitize.js';
import {chooseEditorFormat} from './nativeHtmlTestHarness.ts';
import {TemplateBodyEditor} from './TemplateBodyEditor.tsx';
beforeAll(async()=>{initSync({module:readFileSync(resolve('src/wasm/mw-sanitize/mw_sanitize_bg.wasm'))});await import('./RichTextEditor.tsx');},60000);
it('opening a legacy plain template does not rewrite it into HTML or normalize its saved text',async()=>{
 const changed=vi.fn();
 render(()=><TemplateBodyEditor id="legacy" label="Template content" text={'  <b>Literal</b>\r\nNext line  '} onChange={changed}/>);
 const body=await screen.findByRole('textbox',{name:'Template content'});
 expect(body).toHaveTextContent('<b>Literal</b>');
 expect(body.querySelector('b')).toBeNull();expect(changed).not.toHaveBeenCalled();
});
it('plain composing preserves literal characters when the user edits',async()=>{
 const [text,setText]=createSignal('Before');const changed=vi.fn((next:string)=>setText(next));
 render(()=><TemplateBodyEditor id="literal" label="Template content" text={text()} onChange={changed}/>);
 await chooseEditorFormat('literal-format','plain');changed.mockClear();
 fireEvent.input(screen.getByRole('textbox',{name:'Template content'}),{target:{value:'<img src=x>\nAfter'}});
 expect(changed).toHaveBeenCalledWith('<img src=x>\nAfter','');expect(text()).toBe('<img src=x>\nAfter');
});

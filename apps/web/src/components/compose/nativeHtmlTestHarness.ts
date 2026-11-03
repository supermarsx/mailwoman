// jsdom does not load srcdoc documents. Exercise the real editor's load handler
// so compose tests can await native HTML readiness without replacing the editor.
import {afterEach,beforeAll,beforeEach} from 'vitest';
import {fireEvent,screen,waitFor} from '@solidjs/testing-library';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {initSync} from '../../wasm/mw-sanitize/mw_sanitize.js';

beforeAll(async()=>{
  initSync({module:readFileSync(resolve(process.cwd(),'src/wasm/mw-sanitize/mw_sanitize_bg.wasm'))});
  await import('./HtmlSourceEditor.tsx');
},60000);

export function loadNativeHtmlFrame(frame:HTMLIFrameElement):void {
  if(frame.contentDocument?.getElementById('mw-native-editor'))return;
  const view=frame.contentWindow as (Window&{CSSStyleSheet:typeof CSSStyleSheet})|null;
  if(view&&!view.CSSStyleSheet.prototype.replaceSync)Object.defineProperty(view.CSSStyleSheet.prototype,'replaceSync',{value:()=>{},configurable:true});
  frame.contentDocument?.documentElement.setAttribute('data-mw-native-document','true');
  fireEvent.load(frame);
}

let observer:MutationObserver|undefined;
beforeEach(()=>{
  observer=new MutationObserver(()=>document.querySelectorAll<HTMLIFrameElement>('iframe[title="Visual editor"]').forEach(loadNativeHtmlFrame));
  observer.observe(document.body,{childList:true,subtree:true});
});
afterEach(()=>observer?.disconnect());

/** A structured-editor test chooses its editor explicitly; new-item defaults
 * remain exercised by the native-editor tests. */
export async function chooseEditorFormat(controlId:string,mode:'plain'|'rich'|'html'):Promise<void> {
  const control=await screen.findByTestId(controlId) as HTMLButtonElement;
  if(control.value!==mode){fireEvent.click(control);fireEvent.click(await screen.findByRole('menuitemradio',{name:{plain:'Plain text',rich:'Rich text',html:'HTML'}[mode]}));}
  await waitFor(()=>{if(control.value!==mode)throw new Error('Editor format is still changing');});
}
export async function chooseComposeFormat(mode:'plain'|'rich'|'html'):Promise<void> {
  await chooseEditorFormat('format-toggle',mode);
  const control=screen.getByTestId('format-toggle') as HTMLButtonElement;
  await waitFor(()=>{
    if(control.value!==mode)throw new Error('Composer format is still changing');
    if(mode==='plain'&&screen.queryByLabelText('Body')?.tagName!=='TEXTAREA')throw new Error('Plain body is not ready');
    if(mode==='rich'&&screen.queryByRole('tab',{name:'HTML source'}))throw new Error('Native HTML is still closing');
    if(mode==='html'&&!screen.queryByRole('tab',{name:'HTML source'}))throw new Error('Native HTML is not ready');
  });
}
export async function structuredComposeEditor():Promise<HTMLElement>{await chooseComposeFormat('rich');return screen.findByTestId('compose-richtext');}

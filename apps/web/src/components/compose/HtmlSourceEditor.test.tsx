import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {afterEach,beforeAll,expect,it,vi} from 'vitest';
import HtmlSourceEditor from './HtmlSourceEditor.tsx';
import type {RichTextApi} from './RichTextEditor.tsx';
import {initSync} from '../../wasm/mw-sanitize/mw_sanitize.js';
import * as nativeHtml from './htmlSource.ts';
beforeAll(()=>initSync({module:readFileSync(resolve(process.cwd(),'src/wasm/mw-sanitize/mw_sanitize_bg.wasm'))}));
afterEach(()=>vi.restoreAllMocks());
function mount(initial='<p>Start</p>'){
 const change=vi.fn();let api!:RichTextApi;
 render(()=><HtmlSourceEditor initialHtml={initial} owned={()=>new Set()} onChange={change} onReady={value=>api=value}/>);
 const frame=screen.getByTitle('Visual editor') as HTMLIFrameElement;
 const view=frame.contentWindow as (Window&{CSSStyleSheet:typeof CSSStyleSheet})|null;
 if(view&&!view.CSSStyleSheet.prototype.replaceSync)Object.defineProperty(view.CSSStyleSheet.prototype,'replaceSync',{value:()=>{},configurable:true});
 frame.contentDocument!.documentElement.setAttribute('data-mw-native-document','true');
 fireEvent.load(frame);
 return {change,api,frame};
}
async function sourceField(){fireEvent.click(screen.getByRole('tab',{name:'HTML source'}));return screen.findByRole('textbox',{name:'HTML source'});}
it('defaults to native visual editing and preserves authored source and email layout safely',async()=>{
 const original='<style>.brand{padding:12px}</style><table class="brand" width="600" cellpadding="8"><tr><td><font face="Georgia">Start</font></td></tr></table>';
 const {api,change}=mount(original);await api.whenReady!();
 expect(screen.getByRole('tab',{name:'Visual editor'})).toHaveAttribute('aria-selected','true');
 expect(screen.getByTestId('html-view-group')).toContainElement(screen.getByRole('tab',{name:'Visual editor'}));
 expect(await sourceField()).toHaveValue(original);
 expect(screen.getByTestId('html-view-group')).toContainElement(screen.getByRole('tab',{name:'HTML source'}));
 expect(screen.queryByTestId('rt-group-font')).toBeNull();
 const source='<style>.brand{padding:12px}</style><table class="brand" width="600" onclick="steal()"><tr><td><font face="Georgia"><strong>Safe</strong></font><img src="https://tracker.test/pixel"></td></tr></table><script>steal()</script>';
 fireEvent.input(screen.getByRole('textbox',{name:'HTML source'}),{target:{value:source}});await api.whenReady!();
 const [html,text,authored]=change.mock.calls.at(-1)!;
 expect(html).toContain('<strong>Safe</strong>');expect(html).toContain('width="600"');expect(html).toContain('face="Georgia"');expect(html).toContain('class="brand"');expect(html).not.toMatch(/<script|onclick|tracker/);expect(text).toContain('Safe');expect(authored).toBe(source);
 expect(document.querySelector('script,table')).toBeNull();
 api.appendHtml('<p>Signature</p>');await api.whenReady!();expect(change.mock.calls.at(-1)![0]).toContain('Signature');
});
it('publishes the newest source edit and ignores a late older normalization',async()=>{
 const {api,change}=mount();await api.whenReady!();const field=await sourceField();
 const normalize=nativeHtml.normalizeHtmlSource;let release!:(value:{html:string;text:string})=>void;
 vi.spyOn(nativeHtml,'normalizeHtmlSource').mockImplementation((html,owned,embedded)=>html==='<p>Old</p>'?new Promise(resolve=>{release=resolve;}):normalize(html,owned,embedded));
 fireEvent.input(field,{target:{value:'<p>Old</p>'}});fireEvent.input(field,{target:{value:'<p>Newest</p>'}});await api.whenReady!();
 expect(change.mock.calls.at(-1)![0]).toContain('Newest');release({html:'<p>Old</p>',text:'Old'});await Promise.resolve();
 expect(change.mock.calls.at(-1)![0]).not.toContain('Old');
});
it('rejects readiness on sanitization failure and retains the last safe output',async()=>{
 const {api,change}=mount();await api.whenReady!();const field=await sourceField();const last=change.mock.calls.at(-1)![0];
 vi.spyOn(nativeHtml,'normalizeHtmlSource').mockRejectedValue(new Error('Sanitizer unavailable'));
 fireEvent.input(field,{target:{value:'<img src="https://unsafe.test/x"><script>unsafe()</script>'}});
 await expect(api.whenReady!()).rejects.toThrow('Sanitizer unavailable');
 expect(change.mock.calls.at(-1)![0]).toBe(last);await waitFor(()=>expect(screen.getByRole('status')).toHaveTextContent('The HTML could not be prepared'));
});
it('keeps source accessible after initial preparation fails and retries without publishing unsafe HTML',async()=>{
 const normalize=nativeHtml.normalizeHtmlSource;
 const prepare=vi.spyOn(nativeHtml,'normalizeHtmlSource').mockRejectedValue(new Error('WASM unavailable'));
 const original='<table><tr><td>Recover me</td></tr></table><script>unsafe()</script>';
 const {api,change}=mount(original);
 await expect(api.whenReady!()).rejects.toThrow('WASM unavailable');
 expect(change).not.toHaveBeenCalled();
 expect(await sourceField()).toHaveValue(original);
 await expect(api.whenReady!()).rejects.toThrow('WASM unavailable');
 expect(document.querySelector('script,table')).toBeNull();
 prepare.mockImplementation(normalize);
 fireEvent.click(screen.getByRole('button',{name:'Retry'}));
 await api.whenReady!();
 expect(change.mock.calls.at(-1)![0]).toContain('Recover me');
 expect(change.mock.calls.at(-1)![0]).not.toContain('<script');
 expect(change.mock.calls.at(-1)![2]).toBe(original);
 expect(screen.queryByRole('button',{name:'Retry'})).toBeNull();
});
it('switches the visual/source tabs with keyboard navigation and keeps focus on the active tab',async()=>{
 const {api}=mount();await api.whenReady!();
 const visual=screen.getByRole('tab',{name:'Visual editor'}),source=screen.getByRole('tab',{name:'HTML source'});
 visual.focus();fireEvent.keyDown(visual,{key:'ArrowRight'});
 await waitFor(()=>expect(source).toHaveAttribute('aria-selected','true'));expect(source).toHaveFocus();expect(visual).toHaveAttribute('tabindex','-1');
 fireEvent.keyDown(source,{key:'Home'});await waitFor(()=>expect(visual).toHaveAttribute('aria-selected','true'));expect(visual).toHaveFocus();
});

import {describe,it,expect,vi,beforeAll,beforeEach} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {SignatureEditor} from './SignatureEditor.tsx';
import {SettingsService} from './service.ts';
import type {Signature} from './types.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {initSync} from '../../wasm/mw-sanitize/mw_sanitize.js';
import {chooseEditorFormat,loadNativeHtmlFrame} from '../../components/compose/nativeHtmlTestHarness.ts';
// The editor and the source editor are lazy chunks (~290 kB of ProseMirror). Their
// cold transform under a full parallel run took longer than the 1 s `findBy*`
// budget, so the first await of the file timed out (the flake other lanes saw).
// Loading them here moves that cost into a hook with its own budget; every find
// below keeps the 1 s fast-fail (same mechanism and fix as Compose.test, see
// vite.config.ts FLAKE POLICY).
beforeAll(async()=>{initSync({module:readFileSync(resolve(process.cwd(),'src/wasm/mw-sanitize/mw_sanitize_bg.wasm'))});await Promise.all([import('../../components/compose/RichTextEditor.tsx'),import('../../components/compose/HtmlSourceEditor.tsx')]);},60_000);
beforeEach(()=>suspendAccountPreferences());
function setup(initial?:Signature){const save=vi.fn(async(_value:Signature):Promise<void>=>undefined);const done=vi.fn();const cancel=vi.fn();const view=render(()=><SignatureEditor {...(initial?{initial}:{})} service={{upsertSignature:save} as unknown as SettingsService} onSaved={done} onCancel={cancel}/>);return {...view,save,done,cancel};}
async function sourceField(){
 const frame=await screen.findByTitle('Visual editor') as HTMLIFrameElement;
 // jsdom does not navigate srcdoc; exercise the frame's real load handler.
 const frameView=frame.contentWindow as (Window&{CSSStyleSheet:typeof CSSStyleSheet})|null;
 const sheet=frameView?.CSSStyleSheet.prototype;
 if(sheet&&!sheet.replaceSync)Object.defineProperty(sheet,'replaceSync',{value:()=>{},configurable:true});
 frame.contentDocument?.documentElement.setAttribute('data-mw-native-document','true');
 if(!frame.contentDocument?.getElementById('mw-native-editor'))fireEvent.load(frame);
 fireEvent.click(await screen.findByRole('tab',{name:'HTML source'}));
 return screen.findByRole('textbox',{name:'HTML source'});
}
const PNG='iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==';
function png(name='logo.png',bytes=Uint8Array.from(atob(PNG),c=>c.charCodeAt(0))):File{return new File([bytes],name,{type:'image/png'});}
describe('dedicated signature formats',()=>{
 it.each(['rich','html'] as const)('opens a new editable %s variant with its content and independent plain alternative intact',async format=>{
  const html=format==='html'?'<table cellpadding="7"><tr><td><strong>Regards</strong></td></tr></table>':'<p><strong>Regards</strong></p>';
  const save=vi.fn(async()=>{});render(()=><SignatureEditor seed={{name:'',body:'Plain custom',text:'Plain custom',textEdited:true,html,format,isDefault:false}} reservedNames={['Original']} service={{upsertSignature:save} as unknown as SettingsService} onSaved={()=>{}} onCancel={()=>{}}/>);
  fireEvent.input(screen.getByLabelText('Name'),{target:{value:'New variant'}});expect(screen.getByTestId('signature-format')).toHaveValue(format);
  if(format==='html')expect(await sourceField()).toHaveValue(html);else await screen.findByRole('textbox',{name:'Rich signature'});
  fireEvent.click(screen.getByTestId('signature-save'));await waitFor(()=>expect(save).toHaveBeenCalledWith(expect.objectContaining({name:'New variant',format,html,text:'Plain custom',body:'Plain custom',isDefault:false})));
 });
 it('saves explicit HTML source as typed and an independent literal plain alternative',async()=>{
  const {save,done}=setup();
  fireEvent.input(screen.getByLabelText('Name'),{target:{value:'Work'}});
  await chooseEditorFormat('signature-format','html');
  fireEvent.input(await sourceField(),{target:{value:'<p>Regards <strong>Team</strong></p>'}});
  fireEvent.input(screen.getByLabelText('Plain-text alternative'),{target:{value:'Regards <plain> & Team'}});
  fireEvent.click(screen.getByTestId('signature-save'));
  await waitFor(()=>expect(done).toHaveBeenCalledTimes(1));
  expect(save.mock.calls[0]?.[0]).toMatchObject({name:'Work',format:'html',body:'Regards <plain> & Team',html:'<p>Regards <strong>Team</strong></p>',text:'Regards <plain> & Team',textEdited:true});
 });
 it('renders rich source through the editor schema without active elements or remote images',async()=>{
  setup({name:'Work',body:'Fallback',format:'rich',html:'<p onclick="evil()">Hello <strong>team</strong><script>evil()</script><img src="https://tracker.example/x"></p>',text:'Plain',isDefault:false});
  const editor=await screen.findByRole('textbox',{name:'Rich signature'});
  expect(editor.querySelector('strong')).toHaveTextContent('team');
  expect(editor.querySelector('script,img,[onclick]')).toBeNull();
 });
 it('plain format explicitly removes HTML while retaining authored text',async()=>{
  const {save}=setup({name:'Work',body:'Legacy',format:'rich',html:'<p>Rich</p>',text:'Authored plain',textEdited:true,isDefault:false});
  await chooseEditorFormat('signature-format','plain');
  expect(screen.getByLabelText('Signature')).toHaveValue('Authored plain');
  fireEvent.click(screen.getByTestId('signature-save'));
  await waitFor(()=>expect(save).toHaveBeenCalledTimes(1));
  expect(save.mock.calls[0]?.[0]).toMatchObject({html:null,text:'Authored plain',body:'Authored plain'});
 });
 it('refuses an old account editor before any save',async()=>{
  activateAccountPreferences('owner');const {save}=setup({name:'Work',body:'private',isDefault:false});
  suspendAccountPreferences();activateAccountPreferences('other');
  fireEvent.click(screen.getByTestId('signature-save'));
  expect(await screen.findByRole('alert')).toHaveTextContent('signed-in account changed');expect(save).not.toHaveBeenCalled();
 });
 it('keeps the dedicated editor open on a refused save',async()=>{
  const {save,done}=setup({name:'Work',body:'text',isDefault:false});save.mockRejectedValueOnce(new Error('refused'));
  fireEvent.click(screen.getByTestId('signature-save'));
  await waitFor(()=>expect(save).toHaveBeenCalledTimes(1));expect(done).not.toHaveBeenCalled();expect(screen.getByTestId('signature-editor')).toBeInTheDocument();
 });
 it('makes the rich editor inert during a pending save and keeps later edits unsaved',async()=>{
  const {save,done}=setup({name:'Work',body:'Plain',format:'rich',html:'<p>Rich</p>',text:'Plain',isDefault:false});
  let release!:()=>void;save.mockImplementationOnce(()=>new Promise<void>(resolve=>{release=resolve;}));
  const editor=await screen.findByRole('textbox',{name:'Rich signature'});
  fireEvent.click(screen.getByTestId('signature-save'));
  let ancestor=editor.parentElement;let inert=false;while(ancestor){inert ||= ancestor.inert;ancestor=ancestor.parentElement;}expect(inert).toBe(true);expect(screen.getByTestId('signature-save')).toBeDisabled();
  // Programmatic model updates must not be mistaken for the submitted snapshot.
  fireEvent.input(screen.getByLabelText('Plain-text alternative'),{target:{value:'Later local edit'}});
  release();await waitFor(()=>expect(screen.getByTestId('signature-save')).not.toBeDisabled());
  expect(done).not.toHaveBeenCalled();expect(screen.getByLabelText('Plain-text alternative')).toHaveValue('Later local edit');
 });
 it('sends captured account authority and rejects stale acknowledgements',async()=>{
  activateAccountPreferences('owner','msa1_owner_verified_authority');let resolve!:(response:Response)=>void;
  const fetcher=vi.fn(async(_input:string,_init?:RequestInit)=>new Promise<Response>(done=>{resolve=done;}));
  const service=new SettingsService(fetcher);const pending=service.upsertSignature({name:'Work',body:'text',isDefault:false});
  expect(new Headers(fetcher.mock.calls[0]?.[1]?.headers).get('X-Mailwoman-Account')).toBe('owner');
  const rejected=expect(pending).rejects.toThrow('signed-in account changed');suspendAccountPreferences();activateAccountPreferences('other','msa1_other_verified_authority');resolve(new Response('{"ok":true}'));
  await rejected;
 });
});

describe('the composer editor in the signature editor (t49)',()=>{
 it('offers the composer ribbon with the signature groups only',async()=>{
  setup({name:'Work',body:'Regards',format:'rich',html:'<p>Regards</p>',text:'Regards',isDefault:false});
  await screen.findByRole('textbox',{name:'Rich signature'});
  const tabs=screen.getAllByRole('tab').map(tab=>tab.textContent);
  expect(tabs).toEqual(['Format text','Insert','Table']);
  for(const id of ['rt-group-clipboard','rt-group-font','rt-group-paragraph','rt-group-editing','rt-group-view','rt-group-insert','rt-group-table-insert','rt-group-table','signature-group-format'])expect(document.querySelector(`[data-testid="${id}"]`),id).not.toBeNull();
  for(const id of ['compose-group-recipients','compose-attach','compose-crypto','compose-send-options','compose-group-signature'])expect(document.querySelector(`[data-testid="${id}"]`),id).toBeNull();
  expect(screen.queryByRole('button',{name:'Send'})).toBeNull();
  expect(screen.getByRole('button',{name:'Undo'})).toBeInTheDocument();
  expect(screen.getByRole('button',{name:'Copy formatting'})).toBeInTheDocument();
 });
 it('starts a new signature in visual HTML with the full ribbon and source view',async()=>{
  const {cancel}=setup();
  const frame=await screen.findByTitle('Visual editor') as HTMLIFrameElement;loadNativeHtmlFrame(frame);
  expect(screen.getByRole('button',{name:'Signature format'})).toHaveValue('html');
  expect(screen.getByRole('tab',{name:'Visual editor'})).toHaveAttribute('aria-selected','true');
  expect(screen.getByRole('tab',{name:'HTML source'})).toBeInTheDocument();
  expect(screen.getByRole('button',{name:'Find and replace'})).toBeInTheDocument();
  expect(screen.getByRole('button',{name:'Select all'})).toBeInTheDocument();
  fireEvent.click(screen.getByRole('tab',{name:'Insert'}));
  expect(screen.getByLabelText('Insert an image')).toBeInTheDocument();
  expect(screen.getByRole('button',{name:'Insert table'})).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button',{name:'Cancel'}));
  await waitFor(()=>expect(cancel).toHaveBeenCalledOnce());
 });
 it('uses composer find/replace and formatting commands when saving a signature',async()=>{
  const {save}=setup({name:'Work',body:'Regards Team',format:'rich',html:'<p>Regards Team</p>',text:'Regards Team',isDefault:false});
  await screen.findByRole('textbox',{name:'Rich signature'});
  fireEvent.click(screen.getByRole('button',{name:'Find and replace'}));
  fireEvent.input(screen.getByRole('textbox',{name:'Find text'}),{target:{value:'Team'}});
  fireEvent.input(screen.getByRole('textbox',{name:'Replace with'}),{target:{value:'Support'}});
  fireEvent.click(screen.getByRole('button',{name:'Replace all matches'}));
  fireEvent.keyDown(screen.getByRole('textbox',{name:'Replace with'}),{key:'Escape'});
  fireEvent.click(screen.getByRole('button',{name:'Select all'}));
  fireEvent.click(screen.getByTestId('rt-bold'));
  fireEvent.click(screen.getByRole('button',{name:'Undo'}));
  fireEvent.click(screen.getByRole('button',{name:'Redo'}));
  fireEvent.click(screen.getByTestId('signature-save'));
  await waitFor(()=>expect(save).toHaveBeenCalledOnce());
  expect(save.mock.calls[0]?.[0]).toMatchObject({html:'<p><strong>Regards Support</strong></p>',text:'Regards Support',body:'Regards Support'});
 });
 it('opens an existing rich signature without unsaved changes',async()=>{
  const {cancel}=setup({name:'Work',body:'Regards',format:'rich',html:'<p>Regards <strong>Team</strong></p>',text:'Regards Team',isDefault:false});
  await screen.findByRole('textbox',{name:'Rich signature'});
  fireEvent.click(screen.getByRole('button',{name:'Cancel'}));
  await waitFor(()=>expect(cancel).toHaveBeenCalledTimes(1));
 });
 it('guards a change of the saved signature format',async()=>{
  const {cancel}=setup({name:'Work',body:'Regards',format:'rich',html:'<p>Regards</p>',text:'Regards',isDefault:false});
  await screen.findByRole('textbox',{name:'Rich signature'});
  await chooseEditorFormat('signature-format','html');
  await screen.findByRole('tab',{name:'HTML source'});
  fireEvent.click(screen.getByRole('button',{name:'Cancel'}));
  expect(await screen.findByRole('alertdialog')).toBeInTheDocument();
  expect(cancel).not.toHaveBeenCalled();
 });
 it('opens authored HTML visually and retains the source and format when saved and reopened',async()=>{
  const html='<style>.brand td{padding:14px}</style><table width="540" class="brand"><tbody><tr><td style="font-family:Georgia;font-size:17px">Native signature</td></tr></tbody></table>';
  const first=setup({name:'HTML template',body:'Native signature',text:'Native signature',format:'html',html,isDefault:false});
  expect(await screen.findByRole('tab',{name:'Visual editor'})).toHaveAttribute('aria-selected','true');
  expect(screen.queryByRole('textbox',{name:'Rich signature'})).toBeNull();
  expect(screen.queryByRole('textbox',{name:'HTML source'})).toBeNull();
  const source=await sourceField();
  expect(source).toHaveValue(html);
  fireEvent.input(source,{target:{value:html.replace('Native signature','Updated signature')}});
  fireEvent.click(screen.getByTestId('signature-save'));
  await waitFor(()=>expect(first.save).toHaveBeenCalledOnce());
  const saved=first.save.mock.calls[0]![0];
  expect(saved).toMatchObject({format:'html',html:html.replace('Native signature','Updated signature')});
  first.unmount();
  setup(saved);
  expect(await screen.findByRole('tab',{name:'Visual editor'})).toHaveAttribute('aria-selected','true');
  expect(await sourceField()).toHaveValue(saved.html);
 });
 it('opens legacy HTML in the native visual/source editor without reducing it through rich text',async()=>{
  const html='<table width="510"><tbody><tr><td style="padding:15px">Legacy HTML</td></tr></tbody></table>';
  const {cancel}=setup({name:'Legacy HTML',body:'Legacy HTML',html,isDefault:false});
  expect(screen.getByRole('button',{name:'Signature format'})).toHaveValue('html');
  expect(await screen.findByRole('tab',{name:'Visual editor'})).toHaveAttribute('aria-selected','true');
  expect(await sourceField()).toHaveValue(html);
  fireEvent.click(screen.getByRole('button',{name:'Cancel'}));
  await waitFor(()=>expect(cancel).toHaveBeenCalledOnce());
 });
 it('settles the latest HTML edit before changing signature format',async()=>{
  const {save}=setup({name:'Change format',format:'html',html:'<p>Old</p>',body:'Old',text:'Old',isDefault:false});
  const source=await sourceField();
  fireEvent.input(source,{target:{value:'<p>Latest typed signature</p>'}});
  await chooseEditorFormat('signature-format','plain');
  expect(await screen.findByRole('textbox',{name:'Signature'})).toHaveValue('Latest typed signature');
  fireEvent.click(screen.getByTestId('signature-save'));
  await waitFor(()=>expect(save).toHaveBeenCalledOnce());
  expect(save.mock.calls[0]![0]).toMatchObject({format:'plain',html:null,body:'Latest typed signature'});
 });
 it('asks before discarding an edit, through the shared dialog',async()=>{
  const {cancel}=setup({name:'Work',body:'Regards',isDefault:false});
  fireEvent.input(screen.getByLabelText('Signature'),{target:{value:'Changed'}});
  fireEvent.click(screen.getByRole('button',{name:'Cancel'}));
  expect(await screen.findByRole('alertdialog')).toBeInTheDocument();expect(cancel).not.toHaveBeenCalled();
 });
 it('keeps the rich HTML across a plain round trip',async()=>{
  const {save}=setup({name:'Work',body:'Regards',format:'rich',html:'<p>Regards <em>Team</em></p>',text:'Regards Team',isDefault:false});
  await screen.findByRole('textbox',{name:'Rich signature'});
  await chooseEditorFormat('signature-format','plain');
  await chooseEditorFormat('signature-format','rich');
  const editor=await screen.findByRole('textbox',{name:'Rich signature'});
  expect(editor.querySelector('em')).toHaveTextContent('Team');
  fireEvent.click(screen.getByTestId('signature-save'));await waitFor(()=>expect(save).toHaveBeenCalled());
  expect(save.mock.calls[0]?.[0].html).toBe('<p>Regards <em>Team</em></p>');
 });
 it('inserts a table and a horizontal line and saves them',async()=>{
  const {save}=setup({name:'Table signature',body:'Regards',format:'rich',html:'<p>Regards</p>',text:'Regards',isDefault:false});
  await screen.findByRole('textbox',{name:'Rich signature'});
  fireEvent.click(screen.getByRole('tab',{name:'Insert'}));fireEvent.click(screen.getByRole('button',{name:'Insert table'}));
  fireEvent.click(screen.getByTestId('rt-rule'));
  fireEvent.click(screen.getByTestId('signature-save'));await waitFor(()=>expect(save).toHaveBeenCalled());
  const html=save.mock.calls[0]?.[0].html??'';
  const document=new DOMParser().parseFromString(html,'text/html');
  expect(document.querySelector('table')).not.toBeNull();
  expect(document.querySelector('td')?.style.borderTopWidth).toBe('1px');
  expect(document.querySelector('td')?.style.borderTopStyle).toBe('solid');
  expect(document.querySelector('hr')).not.toBeNull();
 });
 it('embeds an image in the signature HTML and keeps it on reload',async()=>{
  const {save}=setup({name:'Logo',body:'Regards',format:'rich',html:'<p>Regards</p>',text:'Regards',isDefault:false});
  await screen.findByRole('textbox',{name:'Rich signature'});
  fireEvent.click(screen.getByRole('tab',{name:'Insert'}));
  fireEvent.change(screen.getByLabelText('Insert an image'),{target:{files:[png()]}});
  await waitFor(()=>expect(screen.getByRole('textbox',{name:'Rich signature'}).querySelector('img')).not.toBeNull());
  fireEvent.click(screen.getByTestId('signature-save'));await waitFor(()=>expect(save).toHaveBeenCalled());
  const html=save.mock.calls[0]?.[0].html??'';
  expect(html).toContain(`<img src="data:image/png;base64,${PNG}" alt="logo.png">`);
 });
 it('refuses an image that does not fit the 32 KiB signature limit, or is not raster',async()=>{
  setup({name:'Logo',body:'Regards',format:'rich',html:'<p>Regards</p>',text:'Regards',isDefault:false});
  await screen.findByRole('textbox',{name:'Rich signature'});
  fireEvent.click(screen.getByRole('tab',{name:'Insert'}));
  fireEvent.change(screen.getByLabelText('Insert an image'),{target:{files:[png('big.png',new Uint8Array(30*1024))]}});
  expect(await screen.findByRole('alert')).toHaveTextContent('limited to 32 KiB');
  fireEvent.change(screen.getByLabelText('Insert an image'),{target:{files:[new File(['<svg/>'],'x.svg',{type:'image/svg+xml'})]}});
  await waitFor(()=>expect(screen.getByRole('alert')).toHaveTextContent('is not a PNG'));
  expect(screen.getByRole('textbox',{name:'Rich signature'}).querySelector('img')).toBeNull();
 });
 it('refuses to save HTML over 32 KiB',async()=>{
  const {save}=setup({name:'Big',body:'x',isDefault:false});
  await chooseEditorFormat('signature-format','html');
  fireEvent.input(await sourceField(),{target:{value:`<p>${'x'.repeat(33*1024)}</p>`}});
  fireEvent.click(screen.getByTestId('signature-save'));
  expect(await screen.findByRole('alert')).toHaveTextContent('over 32 KiB');expect(save).not.toHaveBeenCalled();
 });
});

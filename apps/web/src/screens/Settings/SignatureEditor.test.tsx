import {describe,it,expect,vi,beforeAll,beforeEach} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {SignatureEditor} from './SignatureEditor.tsx';
import {SettingsService} from './service.ts';
import type {Signature} from './types.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
// The editor and the source editor are lazy chunks (~290 kB of ProseMirror). Their
// cold transform under a full parallel run took longer than the 1 s `findBy*`
// budget, so the first await of the file timed out (the flake other lanes saw).
// Loading them here moves that cost into a hook with its own budget; every find
// below keeps the 1 s fast-fail (same mechanism and fix as Compose.test, see
// vite.config.ts FLAKE POLICY).
beforeAll(async()=>{await Promise.all([import('../../components/compose/RichTextEditor.tsx'),import('../../components/compose/HtmlSourceEditor.tsx')]);},60_000);
beforeEach(()=>suspendAccountPreferences());
function setup(initial?:Signature){const save=vi.fn(async(_value:Signature):Promise<void>=>undefined);const done=vi.fn();const cancel=vi.fn();render(()=><SignatureEditor {...(initial?{initial}:{})} service={{upsertSignature:save} as unknown as SettingsService} onSaved={done} onCancel={cancel}/>);return {save,done,cancel};}
const PNG='iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==';
function png(name='logo.png',bytes=Uint8Array.from(atob(PNG),c=>c.charCodeAt(0))):File{return new File([bytes],name,{type:'image/png'});}
describe('dedicated signature formats',()=>{
 it('saves explicit HTML source as typed and an independent literal plain alternative',async()=>{
  const {save,done}=setup();
  fireEvent.input(screen.getByLabelText('Name'),{target:{value:'Work'}});
  fireEvent.change(screen.getByRole('combobox',{name:'Signature format'}),{target:{value:'html'}});
  fireEvent.input(await screen.findByRole('textbox',{name:'HTML source'}),{target:{value:'<p>Regards <strong>Team</strong></p>'}});
  fireEvent.input(screen.getByLabelText('Plain-text alternative'),{target:{value:'Regards <plain> & Team'}});
  fireEvent.click(screen.getByTestId('signature-save'));
  await waitFor(()=>expect(done).toHaveBeenCalledTimes(1));
  expect(save.mock.calls[0]?.[0]).toMatchObject({name:'Work',body:'Regards <plain> & Team',html:'<p>Regards <strong>Team</strong></p>',text:'Regards <plain> & Team',textEdited:true});
 });
 it('renders rich source through the editor schema without active elements or remote images',async()=>{
  setup({name:'Work',body:'Fallback',html:'<p onclick="evil()">Hello <strong>team</strong><script>evil()</script><img src="https://tracker.example/x"></p>',text:'Plain',isDefault:false});
  const editor=await screen.findByRole('textbox',{name:'Rich signature'});
  expect(editor.querySelector('strong')).toHaveTextContent('team');
  expect(editor.querySelector('script,img,[onclick]')).toBeNull();
 });
 it('plain format explicitly removes HTML while retaining authored text',async()=>{
  const {save}=setup({name:'Work',body:'Legacy',html:'<p>Rich</p>',text:'Authored plain',textEdited:true,isDefault:false});
  fireEvent.change(screen.getByRole('combobox',{name:'Signature format'}),{target:{value:'plain'}});
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
  const {save,done}=setup({name:'Work',body:'Plain',html:'<p>Rich</p>',text:'Plain',isDefault:false});
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
  setup({name:'Work',body:'Regards',html:'<p>Regards</p>',text:'Regards',isDefault:false});
  await screen.findByRole('textbox',{name:'Rich signature'});
  const tabs=screen.getAllByRole('tab').map(tab=>tab.textContent);
  expect(tabs).toEqual(['Format text','Insert','Table']);
  for(const id of ['rt-group-clipboard','rt-group-font','rt-group-paragraph','rt-group-insert','rt-group-table-insert','rt-group-table','signature-group-format'])expect(document.querySelector(`[data-testid="${id}"]`),id).not.toBeNull();
  for(const id of ['rt-group-editing','compose-group-recipients','compose-attach','compose-crypto','compose-send-options','compose-group-signature'])expect(document.querySelector(`[data-testid="${id}"]`),id).toBeNull();
  expect(screen.queryByRole('button',{name:'Send'})).toBeNull();
  expect(screen.getByRole('button',{name:'Undo'})).toBeInTheDocument();
  expect(screen.getByRole('button',{name:'Copy formatting'})).toBeInTheDocument();
 });
 it('opens an existing rich signature without unsaved changes',async()=>{
  const {cancel}=setup({name:'Work',body:'Regards',html:'<p>Regards <strong>Team</strong></p>',text:'Regards Team',isDefault:false});
  await screen.findByRole('textbox',{name:'Rich signature'});
  fireEvent.change(screen.getByRole('combobox',{name:'Signature format'}),{target:{value:'html'}});
  await screen.findByRole('textbox',{name:'HTML source'});
  fireEvent.click(screen.getByRole('button',{name:'Cancel'}));
  await waitFor(()=>expect(cancel).toHaveBeenCalledTimes(1));
 });
 it('asks before discarding an edit, through the shared dialog',async()=>{
  const {cancel}=setup({name:'Work',body:'Regards',isDefault:false});
  fireEvent.input(screen.getByLabelText('Signature'),{target:{value:'Changed'}});
  fireEvent.click(screen.getByRole('button',{name:'Cancel'}));
  expect(await screen.findByRole('alertdialog')).toBeInTheDocument();expect(cancel).not.toHaveBeenCalled();
 });
 it('keeps the rich HTML across a plain round trip',async()=>{
  const {save}=setup({name:'Work',body:'Regards',html:'<p>Regards <em>Team</em></p>',text:'Regards Team',isDefault:false});
  await screen.findByRole('textbox',{name:'Rich signature'});
  fireEvent.change(screen.getByRole('combobox',{name:'Signature format'}),{target:{value:'plain'}});
  fireEvent.change(screen.getByRole('combobox',{name:'Signature format'}),{target:{value:'rich'}});
  const editor=await screen.findByRole('textbox',{name:'Rich signature'});
  expect(editor.querySelector('em')).toHaveTextContent('Team');
  fireEvent.click(screen.getByTestId('signature-save'));await waitFor(()=>expect(save).toHaveBeenCalled());
  expect(save.mock.calls[0]?.[0].html).toBe('<p>Regards <em>Team</em></p>');
 });
 it('inserts a table and a horizontal line and saves them',async()=>{
  const {save}=setup({name:'Table signature',body:'Regards',html:'<p>Regards</p>',text:'Regards',isDefault:false});
  await screen.findByRole('textbox',{name:'Rich signature'});
  fireEvent.click(screen.getByRole('tab',{name:'Insert'}));fireEvent.click(screen.getByRole('button',{name:'Insert table'}));
  fireEvent.click(screen.getByTestId('rt-rule'));
  fireEvent.click(screen.getByTestId('signature-save'));await waitFor(()=>expect(save).toHaveBeenCalled());
  const html=save.mock.calls[0]?.[0].html??'';
  expect(html).toContain('<table');expect(html).toContain('border: 1px solid');expect(html).toContain('<hr>');
 });
 it('embeds an image in the signature HTML and keeps it on reload',async()=>{
  const {save}=setup({name:'Logo',body:'Regards',html:'<p>Regards</p>',text:'Regards',isDefault:false});
  await screen.findByRole('textbox',{name:'Rich signature'});
  fireEvent.click(screen.getByRole('tab',{name:'Insert'}));
  fireEvent.change(screen.getByLabelText('Insert an image'),{target:{files:[png()]}});
  await waitFor(()=>expect(screen.getByRole('textbox',{name:'Rich signature'}).querySelector('img')).not.toBeNull());
  fireEvent.click(screen.getByTestId('signature-save'));await waitFor(()=>expect(save).toHaveBeenCalled());
  const html=save.mock.calls[0]?.[0].html??'';
  expect(html).toContain(`<img src="data:image/png;base64,${PNG}" alt="logo.png">`);
 });
 it('refuses an image that does not fit the 32 KiB signature limit, or is not raster',async()=>{
  setup({name:'Logo',body:'Regards',html:'<p>Regards</p>',text:'Regards',isDefault:false});
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
  fireEvent.change(screen.getByRole('combobox',{name:'Signature format'}),{target:{value:'html'}});
  fireEvent.input(await screen.findByRole('textbox',{name:'HTML source'}),{target:{value:`<p>${'x'.repeat(33*1024)}</p>`}});
  fireEvent.click(screen.getByTestId('signature-save'));
  expect(await screen.findByRole('alert')).toHaveTextContent('over 32 KiB');expect(save).not.toHaveBeenCalled();
 });
});

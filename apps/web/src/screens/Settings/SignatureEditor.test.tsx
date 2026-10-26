import {describe,it,expect,vi,beforeEach} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {SignatureEditor} from './SignatureEditor.tsx';
import {SettingsService} from './service.ts';
import type {Signature} from './types.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
beforeEach(()=>suspendAccountPreferences());
function setup(initial?:Signature){const save=vi.fn(async(_value:Signature):Promise<void>=>undefined);const done=vi.fn();const cancel=vi.fn();render(()=><SignatureEditor {...(initial?{initial}:{})} service={{upsertSignature:save} as unknown as SettingsService} onSaved={done} onCancel={cancel}/>);return {save,done,cancel};}
describe('dedicated signature formats',()=>{
 it('saves explicit HTML and an independent literal plain alternative',async()=>{
  const {save,done}=setup();
  fireEvent.input(screen.getByLabelText('Name'),{target:{value:'Work'}});
  fireEvent.change(screen.getByLabelText('Signature format'),{target:{value:'html'}});
  fireEvent.click(screen.getByRole('checkbox',{name:'Edit HTML source'}));
  fireEvent.input(screen.getByRole('textbox',{name:'Edit HTML source'}),{target:{value:'<p>Regards <strong>Team</strong></p>'}});
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
  fireEvent.change(screen.getByLabelText('Signature format'),{target:{value:'plain'}});
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

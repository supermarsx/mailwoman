import {describe,it,expect,vi} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {Signatures} from './Signatures.tsx';
import {Identities} from './Identities.tsx';
import {createSettingsNavigation,SettingsNavigationContext} from './navigation.tsx';
import type {SettingsService} from './service.ts';
function signatureService(){return {listSignatures:vi.fn(async()=>[{name:'Work',body:'Original',isDefault:true}]),upsertSignature:vi.fn(async()=>{}),deleteSignature:vi.fn(async()=>{})} as unknown as SettingsService;}
describe('dedicated account entity editors',()=>{
 it('opens a signature page with no list and guards section navigation and workspace close',async()=>{
  const navigation=createSettingsNavigation({section:'signatures'});const service=signatureService();render(()=><SettingsNavigationContext.Provider value={navigation}><Signatures service={service}/></SettingsNavigationContext.Provider>);
  fireEvent.click(await screen.findByRole('button',{name:'Edit'}));await screen.findByTestId('signature-editor');expect(screen.queryByTestId('signature-list')).not.toBeInTheDocument();expect(navigation.route().editor).toEqual({kind:'edit',id:'Work'});
  const body=screen.getByLabelText('Signature');fireEvent.input(body,{target:{value:'Unsaved signature'}});
  const close=navigation.canLeave();await screen.findByRole('alertdialog');fireEvent.click(screen.getByRole('button',{name:'Keep editing'}));expect(await close).toBe(false);expect(body).toHaveValue('Unsaved signature');
  const leave=navigation.navigate({section:'identities'});await screen.findByRole('alertdialog');fireEvent.click(screen.getByRole('button',{name:'Discard changes'}));expect(await leave).toBe(true);expect(navigation.route()).toEqual({section:'identities'});
 });
 it('reports unsaved changes without asking, and a discard already chosen does not ask again (t50 bulk tab close)',async()=>{
  const navigation=createSettingsNavigation({section:'signatures'});const service=signatureService();render(()=><SettingsNavigationContext.Provider value={navigation}><Signatures service={service}/></SettingsNavigationContext.Provider>);
  fireEvent.click(await screen.findByRole('button',{name:'Edit'}));await screen.findByTestId('signature-editor');expect(navigation.isDirty()).toBe(false);
  fireEvent.input(screen.getByLabelText('Signature'),{target:{value:'Unsaved signature'}});expect(navigation.isDirty()).toBe(true);expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
  expect(await navigation.canLeave(true)).toBe(true);expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
 });
 it('saves the dedicated page then returns to the list without a discard prompt',async()=>{
  const navigation=createSettingsNavigation({section:'signatures',editor:{kind:'new'}});const service=signatureService();render(()=><SettingsNavigationContext.Provider value={navigation}><Signatures service={service}/></SettingsNavigationContext.Provider>);
  fireEvent.input(screen.getByLabelText('Name'),{target:{value:'New signature'}});fireEvent.input(screen.getByLabelText('Signature'),{target:{value:'Hello'}});fireEvent.click(screen.getByTestId('signature-save'));
  await waitFor(()=>expect(navigation.route()).toEqual({section:'signatures'}));expect(service.upsertSignature).toHaveBeenCalledWith({name:'New signature',body:'Hello',html:null,text:'Hello',textEdited:true,isDefault:false});expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
 });
 it('keeps failed saves on the editor with the draft still guarded',async()=>{
  const navigation=createSettingsNavigation({section:'signatures',editor:{kind:'new'}});const service=signatureService();vi.mocked(service.upsertSignature).mockRejectedValue(new Error('private backend detail'));render(()=><SettingsNavigationContext.Provider value={navigation}><Signatures service={service}/></SettingsNavigationContext.Provider>);
  fireEvent.input(screen.getByLabelText('Name'),{target:{value:'Draft'}});fireEvent.click(screen.getByTestId('signature-save'));await waitFor(()=>expect(service.upsertSignature).toHaveBeenCalled());await waitFor(()=>expect(screen.getByTestId('signature-save')).not.toBeDisabled());fireEvent.click(screen.getByRole('button',{name:'Cancel'}));await screen.findByRole('alertdialog');fireEvent.click(screen.getByRole('button',{name:'Keep editing'}));expect(screen.getByLabelText('Name')).toHaveValue('Draft');
 });
 it('opens identities as a separate editor and keeps list fields out of that page',async()=>{
  const service={listIdentities:vi.fn(async()=>[{id:'a',name:'Work',email:'work@example.org'}]),listSignatures:vi.fn(async()=>[]),upsertIdentity:vi.fn(async()=>{})} as unknown as SettingsService;
  const navigation=createSettingsNavigation({section:'identities'});render(()=><SettingsNavigationContext.Provider value={navigation}><Identities service={service}/></SettingsNavigationContext.Provider>);fireEvent.click(await screen.findByRole('button',{name:'Edit'}));await screen.findByTestId('identity-editor');expect(screen.queryByTestId('identity-list')).not.toBeInTheDocument();expect(navigation.route().editor).toEqual({kind:'edit',id:'a'});
 });
});

it('prevents editing while a save is pending so later keystrokes cannot be marked saved',async()=>{
 let finish:()=>void=()=>{};const save=vi.fn(()=>new Promise<void>(resolve=>{finish=resolve;}));const service={...signatureService(),upsertSignature:save} as unknown as SettingsService;const navigation=createSettingsNavigation({section:'signatures',editor:{kind:'new'}});render(()=><SettingsNavigationContext.Provider value={navigation}><Signatures service={service}/></SettingsNavigationContext.Provider>);
 fireEvent.input(screen.getByLabelText('Name'),{target:{value:'Pending'}});fireEvent.click(screen.getByTestId('signature-save'));await waitFor(()=>expect(save).toHaveBeenCalled());expect(screen.getByLabelText('Signature')).toBeDisabled();expect(await navigation.canLeave()).toBe(false);finish();await waitFor(()=>expect(navigation.route().editor).toBeUndefined());
});

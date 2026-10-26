import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {createResource,createSignal,onCleanup,Suspense} from 'solid-js';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import {RetainedSettingsSection} from './RetainedSettingsSection.tsx';
import {AccountSettings} from './AccountSettings.tsx';
import {createSettingsNavigation,SettingsNavigationContext} from './navigation.tsx';
import type {SettingsService} from './service.ts';

beforeEach(()=>activateAccountPreferences('same-id','msa1_first_settings_authority'));
afterEach(()=>suspendAccountPreferences());
it('keeps the settings shell visible while rapidly switching away from pending sections',async()=>{
 const [active,setActive]=createSignal('first');
 let resolve!: (value:string)=>void;
 function Pending(){const [value]=createResource(()=>new Promise<string>(done=>{resolve=done;}));return <p>{value()}</p>;}
 render(()=><Suspense fallback={<p>Whole app loading</p>}><nav>Settings navigation</nav>
  <RetainedSettingsSection name="first" active={active()==='first'}>{()=> <p>First panel</p>}</RetainedSettingsSection>
  <RetainedSettingsSection name="pending" active={active()==='pending'}>{()=> <Pending/>}</RetainedSettingsSection>
 </Suspense>);
 setActive('pending');
 await waitFor(()=>expect(screen.getByText('Settings navigation')).toBeVisible());
 expect(screen.queryByText('Whole app loading')).not.toBeInTheDocument();
 setActive('first');expect(screen.getByText('First panel')).toBeVisible();
 resolve('Loaded panel');await waitFor(()=>expect(screen.getByText('Loaded panel')).not.toBeVisible());
 setActive('pending');expect(screen.getByText('Loaded panel')).toBeVisible();
});
it('mounts only on first visit and retains the same DOM and unsaved local controls',()=>{
 const mounted=vi.fn(),disposed=vi.fn();const [active,setActive]=createSignal(false);
 function Panel(){mounted();onCleanup(disposed);return <input aria-label="Local filter"/>;}
 render(()=><RetainedSettingsSection name="sample" active={active()}>{()=> <Panel/>}</RetainedSettingsSection>);
 expect(mounted).not.toHaveBeenCalled();setActive(true);const input=screen.getByLabelText('Local filter');fireEvent.input(input,{target:{value:'unchanged'}});
 setActive(false);expect(input).not.toBeVisible();expect((input.parentElement as HTMLElement).inert).toBe(true);expect(disposed).not.toHaveBeenCalled();
 setActive(true);expect(screen.getByLabelText('Local filter')).toBe(input);expect(input).toHaveValue('unchanged');expect(mounted).toHaveBeenCalledOnce();
});
it('destroys retained private state at suspension and same-ID realm replacement',()=>{
 const disposed=vi.fn();function Panel(){onCleanup(disposed);return <input aria-label="Private editor"/>;}
 render(()=><RetainedSettingsSection name="private" active>{()=> <Panel/>}</RetainedSettingsSection>);
 const first=screen.getByLabelText('Private editor');fireEvent.input(first,{target:{value:'realm-one secret'}});
 suspendAccountPreferences();expect(screen.queryByLabelText('Private editor')).not.toBeInTheDocument();expect(disposed).toHaveBeenCalledOnce();
 activateAccountPreferences('same-id','msa1_second_settings_authority');expect(screen.getByLabelText('Private editor')).not.toBe(first);expect(screen.getByLabelText('Private editor')).toHaveValue('');
});
it('disposes challenge and generated-secret sections whenever they become inactive',()=>{
 const disposed=vi.fn();const [active,setActive]=createSignal(true);function Challenge(){onCleanup(disposed);return <input aria-label="Recovery secret"/>;}
 render(()=><RetainedSettingsSection name="security" active={active()} retain={false}>{()=> <Challenge/>}</RetainedSettingsSection>);
 fireEvent.input(screen.getByLabelText('Recovery secret'),{target:{value:'one-time secret'}});setActive(false);expect(disposed).toHaveBeenCalledOnce();setActive(true);expect(screen.getByLabelText('Recovery secret')).toHaveValue('');
});
it('retains list data while preserving the actual dirty editor discard guard',async()=>{
 const listSignatures=vi.fn(async()=>[]);const service={listSignatures,listIdentities:vi.fn(async()=>[])} as unknown as SettingsService;
 const nav=createSettingsNavigation({section:'signatures'});
 render(()=><SettingsNavigationContext.Provider value={nav}><button onClick={()=>void nav.navigate({section:'identities'})}>Other section</button><AccountSettings service={service} hideSecurity section={()=>nav.route().section}/></SettingsNavigationContext.Provider>);
 await screen.findByTestId('signature-list');fireEvent.click(screen.getByRole('button',{name:'New signature'}));
 fireEvent.input(await screen.findByLabelText('Signature'),{target:{value:'Unsaved private text'}});
 fireEvent.click(screen.getByRole('button',{name:'Other section'}));await screen.findByRole('alertdialog');fireEvent.click(screen.getByRole('button',{name:'Keep editing'}));
 expect(screen.getByLabelText('Signature')).toHaveValue('Unsaved private text');expect(nav.route().section).toBe('signatures');
 await new Promise(resolve=>setTimeout(resolve,0));
 fireEvent.click(screen.getByRole('button',{name:'Other section'}));await screen.findByRole('alertdialog');fireEvent.click(screen.getByRole('button',{name:'Discard changes'}));
 await waitFor(()=>expect(nav.route().section).toBe('identities'));await nav.navigate({section:'signatures'});
 expect(screen.queryByDisplayValue('Unsaved private text')).not.toBeInTheDocument();expect(listSignatures).toHaveBeenCalledOnce();
});

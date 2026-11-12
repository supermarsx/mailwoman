import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor,within} from '@solidjs/testing-library';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import {Signatures} from './Signatures.tsx';
import type {SettingsService} from './service.ts';
vi.mock('./SignatureEditor.tsx',()=>({SignatureEditor:(props:{initial?:{name:string};seed?:{name:string};onCancel:()=>void})=><div data-testid="signature-editor">{props.initial?.name??'New variant'}<button onClick={props.onCancel}>Back</button></div>}));
afterEach(()=>{cleanup();suspendAccountPreferences();});
it('shows compact named icon actions and preserves edit, variant and delete flows',async()=>{
 activateAccountPreferences('signatures-table');const deleteSignature=vi.fn(async()=>{});
 const service={listSignatures:vi.fn(async()=>[{name:'Work',body:'Regards from the team',isDefault:true}]),deleteSignature} as unknown as SettingsService;
 render(()=><Signatures service={service}/>);let row=await screen.findByRole('listitem');
 expect(row).toHaveTextContent('Work');expect(row).toHaveTextContent('Regards from the team');expect(row).toHaveTextContent('Default');
 const edit=within(row).getByRole('button',{name:'Edit'});expect(edit.textContent).toBe('');expect(edit).toHaveAttribute('data-tooltip','Edit');fireEvent.click(edit);expect(await screen.findByTestId('signature-editor')).toHaveTextContent('Work');
 fireEvent.click(screen.getByText('Back'));row=await screen.findByRole('listitem');fireEvent.click(within(row).getByRole('button',{name:'Create variant'}));expect(await screen.findByTestId('signature-editor')).toHaveTextContent('New variant');
 fireEvent.click(screen.getByText('Back'));row=await screen.findByRole('listitem');fireEvent.click(within(row).getByRole('button',{name:'Delete'}));await waitFor(()=>expect(deleteSignature).toHaveBeenCalledWith('Work'));
});

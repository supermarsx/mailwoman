import {beforeAll,describe,it,expect,vi} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {loadCatalog} from '../../i18n';
import {ClientAccess} from './ClientAccess.tsx';
import {validClientPsk,validClientSlug,type ClientAccessApi,type ClientAccessView} from '../../api/admin/clientAccess.ts';
import {canOpenSecurityTab,SECURITY_TABS} from './shell/securityNavigation.ts';
import {createCan} from './shell/can.ts';
import {anyDirty} from './shell/useDirtyGuard.ts';
const view=(values:Partial<ClientAccessView>={}):ClientAccessView=>({pskEnabled:false,slugEnabled:false,slug:null,pskConfigured:false,grantHours:24,clientEntryPath:'/',...values});
const api=(data:ClientAccessView,save:ClientAccessApi['save']=vi.fn()):ClientAccessApi=>({read:async()=>({data,etag:'"one"',status:200}),save});
beforeAll(async()=>{await loadCatalog('admin');await loadCatalog('admin-security');});
describe('client entry policy',()=>{
 it('validates byte-bounded keys and a single address segment',()=>{
  expect(validClientPsk(' padded secret value ')).toBe(true);expect(validClientPsk('é'.repeat(128))).toBe(true);expect(validClientPsk('é'.repeat(129))).toBe(false);
  for(const value of ['','short',' '.repeat(20)])expect(validClientPsk(value)).toBe(false);
  expect(validClientSlug('private-mail')).toBe(true);for(const value of ['abc','/private','UPPER','-private','private-','a/bc','x'.repeat(65)])expect(validClientSlug(value)).toBe(false);
 });
 it('starts disabled and requires a key before enabling the gate, preserving secret whitespace',async()=>{
  const secret=' padded secret value ';const save=vi.fn(async()=>({data:view({pskEnabled:true,slugEnabled:true,slug:'private-mail',pskConfigured:true,clientEntryPath:'/private-mail'}),etag:'"two"',status:200}));
  render(()=><ClientAccess readOnly={false} api={api(view(),save)}/>);
  await screen.findByTestId('client-entry-path');expect(screen.getByLabelText('Require a pre-shared key')).not.toBeChecked();expect(screen.getByLabelText('Require a private client address')).not.toBeChecked();
  fireEvent.click(screen.getByLabelText('Require a pre-shared key'));expect(screen.getByRole('button',{name:'Save client access'})).toBeDisabled();
  fireEvent.input(screen.getByLabelText('New pre-shared key'),{target:{value:secret}});fireEvent.click(screen.getByLabelText('Require a private client address'));expect(screen.getByRole('button',{name:'Save client access'})).toBeDisabled();
  fireEvent.input(screen.getByLabelText('Client address segment'),{target:{value:'private-mail'}});fireEvent.click(screen.getByRole('button',{name:'Save client access'}));
  await waitFor(()=>expect(save).toHaveBeenCalledWith({pskEnabled:true,slugEnabled:true,slug:'private-mail',grantHours:24,psk:secret},'"one"'));
  await waitFor(()=>expect(screen.getByLabelText('New pre-shared key')).toHaveValue(''));expect(screen.getByTestId('client-entry-path')).toHaveTextContent('/private-mail');
 });
 it('omits an unchanged secret and clears it only after disabling the gate',async()=>{
  const save=vi.fn(async()=>({data:view({pskEnabled:true,pskConfigured:true,grantHours:48}),etag:'"two"',status:200}));
  render(()=><ClientAccess readOnly={false} api={api(view({pskEnabled:true,pskConfigured:true}),save)}/>);await screen.findByTestId('client-psk-status');
  expect(screen.getByLabelText('New pre-shared key')).toHaveValue('');expect(screen.queryByLabelText('Forget the stored pre-shared key')).not.toBeInTheDocument();
  fireEvent.input(screen.getByLabelText('Client access grant lifetime (hours)'),{target:{value:'48'}});fireEvent.click(screen.getByRole('button',{name:'Save client access'}));
  await waitFor(()=>expect(save).toHaveBeenCalledWith({pskEnabled:true,slugEnabled:false,slug:null,grantHours:48},'"one"'));
  await waitFor(()=>expect(screen.getByLabelText('Require a pre-shared key')).toBeEnabled());fireEvent.click(screen.getByLabelText('Require a pre-shared key'));fireEvent.click(screen.getByLabelText('Forget the stored pre-shared key'));fireEvent.click(screen.getByRole('button',{name:'Save client access'}));
  await waitFor(()=>expect(save).toHaveBeenLastCalledWith({pskEnabled:false,slugEnabled:false,slug:null,grantHours:48,clearPsk:true},'"two"'));
 });
 it('allows unscoped settings readers to view the tab without granting writes or scoped access',async()=>{
  const tab=SECURITY_TABS.find(value=>value.id==='client-access')!;
  for(const scope of [null,['example.org']]){const grants=createCan(()=>({kind:'listed',me:{username:'reader',displayName:null,roles:[],capabilities:['settings.read'],domainScope:scope,totpEnrolled:false}}));expect(canOpenSecurityTab(grants,tab)).toBe(scope===null);}
  const save=vi.fn();render(()=><ClientAccess readOnly api={api(view(),save)}/>);await screen.findByTestId('client-entry-path');expect(screen.getByLabelText('Require a pre-shared key')).toBeDisabled();expect(screen.getByLabelText('Client address segment')).toBeDisabled();expect(screen.getByRole('button',{name:'Save client access'})).toBeDisabled();expect(save).not.toHaveBeenCalled();
 });
 it('retains unsaved secrets after a stale write and reloads the canonical policy explicitly',async()=>{
  const save=vi.fn(async()=>{throw {status:412,code:'stale',detail:'Policy changed'};});const service=api(view(),save);
  service.read=vi.fn().mockResolvedValueOnce({data:view(),etag:'"one"',status:200}).mockResolvedValue({data:view({slug:'new-policy'}),etag:'"new"',status:200});
  render(()=><ClientAccess readOnly={false} api={service}/>);await screen.findByTestId('client-entry-path');const secret=screen.getByLabelText('New pre-shared key');
  fireEvent.input(secret,{target:{value:'new secret retained'}});fireEvent.click(screen.getByRole('button',{name:'Save client access'}));
  await waitFor(()=>expect(save).toHaveBeenCalledTimes(1));await waitFor(()=>expect(screen.getByRole('button',{name:'Save client access'})).toBeEnabled());expect(secret).toHaveValue('new secret retained');
  fireEvent.click(screen.getByRole('button',{name:'Reload saved policy'}));await waitFor(()=>expect(screen.getByLabelText('Client address segment')).toHaveValue('new-policy'));expect(secret).toHaveValue('');
 });
 it('guards unsaved changes and prevents a save from racing a delayed policy reload',async()=>{
  let finish!:(value:Awaited<ReturnType<ClientAccessApi['read']>>)=>void;const pending=new Promise<Awaited<ReturnType<ClientAccessApi['read']>>>(resolve=>{finish=resolve;});
  const save=vi.fn();const service=api(view(),save);service.read=vi.fn().mockResolvedValueOnce({data:view(),etag:'"one"',status:200}).mockReturnValue(pending);
  const mounted=render(()=><ClientAccess readOnly={false} api={service}/>);await screen.findByTestId('client-entry-path');expect(anyDirty()).toBe(false);
  fireEvent.input(screen.getByLabelText('New pre-shared key'),{target:{value:'unsaved private key'}});expect(anyDirty()).toBe(true);
  fireEvent.click(screen.getByRole('button',{name:'Reload saved policy'}));expect(screen.getByLabelText('New pre-shared key')).toBeDisabled();expect(screen.getByRole('button',{name:'Save client access'})).toBeDisabled();
  fireEvent.click(screen.getByRole('button',{name:'Save client access'}));expect(save).not.toHaveBeenCalled();
  finish({data:view({grantHours:72}),etag:'"new"',status:200});await waitFor(()=>expect(screen.getByLabelText('Client access grant lifetime (hours)')).toHaveValue(72));expect(anyDirty()).toBe(false);
  fireEvent.input(screen.getByLabelText('New pre-shared key'),{target:{value:'another unsaved key'}});expect(anyDirty()).toBe(true);mounted.unmount();expect(anyDirty()).toBe(false);
 });
});

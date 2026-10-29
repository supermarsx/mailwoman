import {beforeAll,it,expect,vi} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {loadCatalog} from '../../i18n';
import {NextcloudPim} from './NextcloudPim.tsx';
import type {NextcloudPimApi} from '../../api/nextcloudPim.ts';
beforeAll(()=>loadCatalog('nextcloud'));
function service():NextcloudPimApi{return {
  list:vi.fn(async()=>({accountId:'account-a',supported:true,collections:[]})),
  discover:vi.fn(async()=>({accountId:'account-a',collections:[{href:'https://cloud.test/work/',name:'Work',kind:'calendar' as const}]})),
  create:vi.fn(async(_a:string,_i:string,_k:'calendar'|'tasks',name:string)=>({collectionId:'cal-new',href:'https://cloud.test/new/',name})),
  apply:vi.fn(async()=>({collectionId:'cal-a'})),disconnect:vi.fn(async()=>{}),sync:vi.fn(async()=>({ok:true})),
};}
function mount(api:NextcloudPimApi){return render(()=><NextcloudPim accountId="account-a" instanceId="cloud" instanceName="Cloud" onBack={()=>{}} api={api}/>);}
it('previews without mutation then applies explicit account-scoped selection',async()=>{
  const api=service();mount(api);fireEvent.click(await screen.findByRole('button',{name:'Discover collections'}));
  const connect=await screen.findByRole('button',{name:'Connect Work'});expect(api.apply).not.toHaveBeenCalled();expect(api.sync).not.toHaveBeenCalled();
  fireEvent.click(connect);await waitFor(()=>expect(api.apply).toHaveBeenCalledWith('account-a','cloud',{href:'https://cloud.test/work/',name:'Work',kind:'calendar'}));
  await screen.findByText('Collection connected. Synchronize now to load its contents.');expect(api.sync).not.toHaveBeenCalled();
});
it('does not replay refused discovery automatically',async()=>{
  const api=service();vi.mocked(api.discover).mockRejectedValue(new Error('revoked'));mount(api);
  fireEvent.click(await screen.findByRole('button',{name:'Discover collections'}));await screen.findByRole('alert');
  await waitFor(()=>expect(screen.getByRole('button',{name:'Discover collections'})).toBeEnabled());
  expect(api.discover).toHaveBeenCalledTimes(1);expect(api.apply).not.toHaveBeenCalled();
});
it('does not expose unsupported proxy-mode PIM controls',async()=>{
  const api=service();vi.mocked(api.list).mockResolvedValue({accountId:'account-a',supported:false,collections:[]});mount(api);
  await screen.findByText(/requires the native mail engine/);expect(screen.queryByRole('button',{name:'Discover collections'})).not.toBeInTheDocument();expect(api.discover).not.toHaveBeenCalled();
});
it('creates a calendar in Nextcloud for this account and reports it connected',async()=>{
  const api=service();mount(api);
  const name=await screen.findByLabelText('Name of the new collection');
  const create=screen.getByRole('button',{name:'Create in Nextcloud'});expect(create).toBeDisabled();
  fireEvent.input(name,{target:{value:'  Team plans '}});
  fireEvent.click(create);
  await waitFor(()=>expect(api.create).toHaveBeenCalledWith('account-a','cloud','calendar','Team plans'));
  await screen.findByText(/Team plans.*was created in Nextcloud and connected/);
  expect(api.list).toHaveBeenCalledTimes(2);expect(api.sync).not.toHaveBeenCalled();
});
it('offers no creation for address books',async()=>{
  const api=service();mount(api);
  fireEvent.change(await screen.findByLabelText('Collection category'),{target:{value:'contacts'}});
  await waitFor(()=>expect(screen.queryByRole('button',{name:'Create in Nextcloud'})).not.toBeInTheDocument());
});

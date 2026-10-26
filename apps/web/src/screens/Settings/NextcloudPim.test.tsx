import {beforeAll,it,expect,vi} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {loadCatalog} from '../../i18n';
import {NextcloudPim} from './NextcloudPim.tsx';
import type {NextcloudPimApi} from '../../api/nextcloudPim.ts';
beforeAll(()=>loadCatalog('nextcloud'));
function service():NextcloudPimApi{return {
  list:vi.fn(async()=>({accountId:'account-a',supported:true,collections:[]})),
  discover:vi.fn(async()=>({accountId:'account-a',collections:[{href:'https://cloud.test/work/',name:'Work',kind:'calendar' as const}]})),
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

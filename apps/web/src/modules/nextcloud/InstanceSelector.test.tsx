import {beforeEach,describe,it,expect,vi} from 'vitest';
import {render,screen,fireEvent,waitFor,within} from '@solidjs/testing-library';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import {NextcloudService,type Fetcher} from './service.ts';
import {FilePicker} from './FilePicker.tsx';
beforeEach(()=>{suspendAccountPreferences();activateAccountPreferences('alice');});
const json=(body:unknown)=>new Response(JSON.stringify(body));
describe('explicit Nextcloud instance selection',()=>{
  it('keeps two pickers independent and never lists before an explicit choice',async()=>{
    const fetcher=vi.fn<Fetcher>(async(path)=>{
      if(path==='/api/account/nextcloud')return json({accountId:'alice',instances:[{id:'a',name:'Work cloud'},{id:'b',name:'Personal cloud'},{id:'stale',name:'Revoked cloud'}],links:[{instanceId:'a',ready:true},{instanceId:'b',ready:true},{instanceId:'stale',ready:false}]});
      if(path.startsWith('/api/nextcloud/files?')){const id=new URL(path,'https://app.test').searchParams.get('instanceId');return json({entries:[{path:`/${id}.txt`,name:`${id}.txt`,isDir:false,size:1,modified:null,contentType:'text/plain'}]});}
      return json({attachments:[]});
    });
    const first=new NextcloudService(fetcher).forAccount('alice');const second=new NextcloudService(fetcher).forAccount('alice');
    render(()=><><FilePicker service={first} mode="files"/><FilePicker service={second} mode="files"/></>);
    await screen.findAllByRole('option',{name:'Personal cloud'});
    expect(fetcher.mock.calls.every(([path])=>path==='/api/account/nextcloud')).toBe(true);
    expect(screen.queryByRole('option',{name:'Revoked cloud'})).not.toBeInTheDocument();
    const selects=screen.getAllByRole('combobox',{name:'Nextcloud instance'});
    fireEvent.change(selects[0]!,{target:{value:'a'}});fireEvent.change(selects[1]!,{target:{value:'b'}});
    const pickers=screen.getAllByTestId('nc-picker');
    await within(pickers[0]!).findByText('a.txt');await within(pickers[1]!).findByText('b.txt');
    await first.attach(['/a.txt']);await second.attach(['/b.txt']);
    const posted=fetcher.mock.calls.filter(([path])=>path==='/api/nextcloud/attach').map(([,init])=>JSON.parse(init?.body as string));
    expect(posted.map(body=>body.instanceId)).toEqual(['a','b']);
    fireEvent.change(selects[0]!,{target:{value:'b'}});
    await waitFor(()=>expect(within(pickers[0]!).getByText('b.txt')).toBeVisible());
    expect(within(pickers[1]!).getByText('b.txt')).toBeVisible();
  });
});

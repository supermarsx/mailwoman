import {describe,it,expect,vi} from 'vitest';
import {screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {AdminNextcloud} from './Nextcloud.tsx';
import {renderInShell,holding} from './shell/testkit.tsx';
import type {NextcloudAccountsApi} from '../../api/nextcloudAccounts.ts';
const row={id:'cloud',name:'Company cloud',baseUrl:'https://cloud.test/',enabled:true,etag:'"revision"'};
function service():NextcloudAccountsApi{return {account:vi.fn(),link:vi.fn(),unlink:vi.fn(),instances:vi.fn(async()=>({instances:[row],maxInstances:64,maxFileBytes:8388608})),save:vi.fn()};}
describe('approved Nextcloud instances',()=>{
  it('holds readonly editors disabled without integration write capability',async()=>{
    renderInShell(()=><AdminNextcloud api={service()}/>,{loadMe:holding(['integrations.read']),route:()=>({section:'nextcloud',rest:['cloud']})});
    expect(await screen.findByLabelText(/^HTTPS installation URL/)).toBeDisabled();
    expect(screen.queryByRole('button',{name:'Company cloud'})).not.toBeInTheDocument();
  });
  it('preserves refused edits and submits the exact loaded revision',async()=>{
    const api=service();vi.mocked(api.save).mockRejectedValue(new Error('stale'));
    renderInShell(()=><AdminNextcloud api={api}/>,{route:()=>({section:'nextcloud',rest:['cloud']})});
    const input=await screen.findByLabelText(/^Display name/);await waitFor(()=>expect(input).toBeEnabled());
    fireEvent.input(input,{target:{value:'Renamed cloud'}});fireEvent.input(screen.getByLabelText(/^Additional certificate authorities/),{target:{value:'test-ca-pem'}});fireEvent.click(screen.getByRole('button',{name:'Save'}));
    await screen.findByRole('alert');expect(api.save).toHaveBeenCalledWith('cloud',{name:'Renamed cloud',baseUrl:'https://cloud.test/',enabled:true,caPem:'test-ca-pem'},'"revision"');expect(input).toHaveValue('Renamed cloud');
  });
});

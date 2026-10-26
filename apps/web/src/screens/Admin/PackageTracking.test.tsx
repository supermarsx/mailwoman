import {it,expect,vi,beforeEach} from 'vitest';
import {screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {PackageTracking} from './PackageTracking.tsx';
import {renderInShell,holding} from './shell/testkit.tsx';
const fixture=vi.hoisted(()=>({request:vi.fn()}));
vi.mock('./shell/adminApi.ts',async original=>({...await original<typeof import('./shell/adminApi.ts')>(),adminRequest:fixture.request}));
beforeEach(()=>fixture.request.mockReset().mockResolvedValue({data:{provider:{enabled:false,refreshSecs:900,configured:true,revision:'r1'}},etag:'"r1"'}));
it('keeps a stored key secret and honors read-only administrators',async()=>{
  renderInShell(()=><PackageTracking/>,{loadMe:holding(['integrations.read'])});
  const enabled=await screen.findByLabelText('Allow opted-in accounts to contact this provider');
  expect(enabled).not.toBeChecked();expect(enabled).toBeDisabled();
  const key=screen.getByLabelText(/^Provider API key/);expect(key).toHaveValue('');expect(key).toBeDisabled();
});
it('retains an existing key and sends conditional provider changes',async()=>{
  renderInShell(()=><PackageTracking/>);
  const enabled=await screen.findByLabelText('Allow opted-in accounts to contact this provider');
  await waitFor(()=>expect(enabled).toBeEnabled());fireEvent.click(enabled);
  fireEvent.click(screen.getByRole('button',{name:'Save provider configuration'}));
  await waitFor(()=>expect(fixture.request).toHaveBeenCalledWith('/package-tracking',{method:'PUT',ifMatch:'"r1"',body:{enabled:true,refreshSecs:900}}));
});

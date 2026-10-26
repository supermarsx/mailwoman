import { describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { Analytics } from './Analytics.tsx';
import { renderInShell, holding } from './shell/testkit.tsx';
import type { AnalyticsApi, AnalyticsView } from '../../api/admin/analytics.ts';
const data:AnalyticsView={config:{enabled:false,provider:'umami',endpoint:'',siteId:'',allowHttp:false},status:{queued:0,delivered:0,dropped:0,mode:'event-counts',uniqueVisitors:null},allowlist:[]};
function api():AnalyticsApi{return {read:vi.fn(async()=>({data,etag:'"one"'})),save:vi.fn(async config=>({data:{...data,config},etag:'"two"'})),allow:vi.fn(async()=>{}),remove:vi.fn(async()=>{})};}
describe('analytics configuration',()=>{
  it('shows truthful disabled defaults and denies edits without capability',async()=>{
    renderInShell(()=><Analytics api={api()}/>,{loadMe:holding(['integrations.read'])});
    const enabled=await screen.findByLabelText('Enable optional analytics collection');
    expect(enabled).not.toBeChecked();expect(enabled).toBeDisabled();
    expect(screen.queryByRole('button',{name:'Allow destination'})).not.toBeInTheDocument();
  });
  it('saves only explicit edits with the loaded revision and preserves a refused edit',async()=>{
    const transport=api();vi.mocked(transport.save).mockRejectedValueOnce(new Error('Configuration changed; reload before saving'));
    renderInShell(()=><Analytics api={transport}/>);
    const endpoint=await screen.findByLabelText(/^Collector endpoint/);
    await waitFor(()=>expect(endpoint).toBeEnabled());
    fireEvent.input(endpoint,{target:{value:'https://stats.example/api/send'}});
    fireEvent.click(screen.getByRole('button',{name:'Save configuration'}));
    await waitFor(()=>expect(transport.save).toHaveBeenCalledWith({...data.config,endpoint:'https://stats.example/api/send'},'"one"'));
    expect(endpoint).toHaveValue('https://stats.example/api/send');
    expect(await screen.findByRole('alert')).toBeInTheDocument();
  });
});

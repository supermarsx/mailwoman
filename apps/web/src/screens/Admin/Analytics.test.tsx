import { describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { Analytics } from './Analytics.tsx';
import { renderInShell, holding } from './shell/testkit.tsx';
import type { AnalyticsApi, AnalyticsView } from '../../api/admin/analytics.ts';
import type { AnalyticsTagApi, AnalyticsTagView } from '../../api/admin/analyticsTag.ts';
const tagView:AnalyticsTagView={config:{enabled:false,provider:'plausible',siteId:'',host:'',region:'',customKind:'url',scriptUrl:'',snippet:'',scriptOrigins:[],connectOrigins:[],imgOrigins:[]},preview:{valid:false,error:'x'},maxSnippetBytes:32768};
const tagApi:AnalyticsTagApi={read:vi.fn(async()=>({data:tagView,etag:'"t"'})),save:vi.fn(),test:vi.fn()};
const data:AnalyticsView={config:{enabled:false,provider:'umami',endpoint:'',siteId:'',allowHttp:false},status:{queued:0,delivered:0,dropped:0,mode:'event-counts',uniqueVisitors:null},allowlist:[]};
function api():AnalyticsApi{return {read:vi.fn(async()=>({data,etag:'"one"'})),save:vi.fn(async config=>({data:{...data,config},etag:'"two"'})),allow:vi.fn(async()=>{}),remove:vi.fn(async()=>{})};}
describe('analytics configuration',()=>{
  it('shows exporter status as a named table',async()=>{
    renderInShell(()=><Analytics api={api()} tagApi={tagApi}/>);
    const table=await screen.findByRole('table',{name:'Exporter status'});
    expect(table).toHaveTextContent('Queued');expect(table).toHaveTextContent('Accepted by collector');expect(table).toHaveTextContent('Dropped');
  });
  it('configures Google without enabling it and clears the write-only secret after saving',async()=>{
    const transport=api();
    transport.save=vi.fn(async config=>({data:{...data,config:{enabled:config.enabled,provider:config.provider,endpoint:config.endpoint,siteId:config.siteId,allowHttp:config.allowHttp,apiSecretConfigured:true}},etag:'"two"'}));
    renderInShell(()=><Analytics api={transport} tagApi={tagApi}/>);
    const provider=await screen.findByLabelText('Provider');await waitFor(()=>expect(provider).toBeEnabled());
    fireEvent.change(provider,{target:{value:'ga4'}});
    expect(screen.getByLabelText('Enable optional analytics collection')).not.toBeChecked();
    expect(screen.queryByLabelText('Allow unencrypted HTTP for this collector')).not.toBeInTheDocument();
    fireEvent.input(screen.getByLabelText('Measurement ID'),{target:{value:'G-EXAMPLE123'}});
    const secret=screen.getByLabelText(/^Measurement Protocol API secret/);
    fireEvent.input(secret,{target:{value:'fixture-secret'}});
    fireEvent.click(screen.getByRole('button',{name:'Save configuration'}));
    await waitFor(()=>expect(transport.save).toHaveBeenCalledWith({enabled:false,provider:'ga4',endpoint:'https://www.google-analytics.com/mp/collect',siteId:'G-EXAMPLE123',allowHttp:false,apiSecret:'fixture-secret'},'"one"'));
    await waitFor(()=>expect(secret).toHaveValue(''));
    fireEvent.click(screen.getByLabelText('Remove the stored API secret (disable collection first)'));
    fireEvent.click(screen.getByRole('button',{name:'Save configuration'}));
    await waitFor(()=>expect(transport.save).toHaveBeenLastCalledWith(expect.objectContaining({clearApiSecret:true}),'"two"'));
    expect(vi.mocked(transport.save).mock.calls[1]![0]).not.toHaveProperty('apiSecret');
    expect(vi.mocked(transport.save).mock.calls[1]![0]).not.toHaveProperty('apiSecretConfigured');
  });
  it('shows truthful disabled defaults and denies edits without capability',async()=>{
    renderInShell(()=><Analytics api={api()} tagApi={tagApi}/>,{loadMe:holding(['integrations.read'])});
    const enabled=await screen.findByLabelText('Enable optional analytics collection');
    expect(enabled).not.toBeChecked();expect(enabled).toBeDisabled();
    expect(screen.queryByRole('button',{name:'Allow destination'})).not.toBeInTheDocument();
  });
  it('saves only explicit edits with the loaded revision and preserves a refused edit',async()=>{
    const transport=api();vi.mocked(transport.save).mockRejectedValueOnce(new Error('Configuration changed; reload before saving'));
    renderInShell(()=><Analytics api={transport} tagApi={tagApi}/>);
    const endpoint=await screen.findByLabelText(/^Collector endpoint/);
    await waitFor(()=>expect(endpoint).toBeEnabled());
    fireEvent.input(endpoint,{target:{value:'https://stats.example/api/send'}});
    fireEvent.click(screen.getByRole('button',{name:'Save configuration'}));
    await waitFor(()=>expect(transport.save).toHaveBeenCalledWith({...data.config,endpoint:'https://stats.example/api/send'},'"one"'));
    expect(endpoint).toHaveValue('https://stats.example/api/send');
    expect(await screen.findByRole('alert')).toBeInTheDocument();
  });
});

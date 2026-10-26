import { describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { Telemetry } from './Telemetry.tsx';
import { renderInShell, holding } from './shell/testkit.tsx';
import type { TelemetryApi, TelemetryView } from '../../api/admin/telemetry.ts';
const config={enabled:false,endpoint:'',serviceName:'mailwoman',environment:'',traces:true,metrics:true,sampleRatio:0.1,exportTimeoutMs:5000,metricsIntervalSecs:60,spanQueueSize:2048,spanBatchSize:512,spanExportDelayMs:5000,prometheusEnabled:false};
const data:TelemetryView={config,effective:config,active:config,pinned:[],restartRequired:false,metricsTokenConfigured:false};
const api=():TelemetryApi=>({read:vi.fn(async()=>({data,etag:'"one"'})),save:vi.fn(async value=>({data:{...data,config:value,restartRequired:true},etag:'"two"'}))});
describe('operational telemetry editor',()=>{
  it('keeps export off by default and respects read-only access',async()=>{
    renderInShell(()=><Telemetry api={api()}/>,{loadMe:holding(['observability.read'])});
    const enabled=await screen.findByLabelText('Enable OTLP export');
    expect(enabled).not.toBeChecked();expect(enabled).toBeDisabled();
    expect(screen.getByText(/No scrape token/)).toBeInTheDocument();
  });
  it('saves explicit edits with revision and shows pending restart',async()=>{
    const transport=api();renderInShell(()=><Telemetry api={transport}/>);
    const endpoint=await screen.findByLabelText(/^OTLP gRPC endpoint/);
    await waitFor(()=>expect(endpoint).toBeEnabled());
    fireEvent.input(endpoint,{target:{value:'http://collector:4317'}});
    fireEvent.click(screen.getByRole('button',{name:'Save telemetry configuration'}));
    await waitFor(()=>expect(transport.save).toHaveBeenCalledWith({...config,endpoint:'http://collector:4317'},'"one"'));
    expect(await screen.findByText(/saved configuration differs/)).toBeInTheDocument();
  });
  it('keeps failures visible rather than presenting defaults as loaded',async()=>{
    const transport=api();vi.mocked(transport.read).mockRejectedValue(new Error('offline'));
    renderInShell(()=><Telemetry api={transport}/>);
    expect(await screen.findByRole('alert')).toBeInTheDocument();
    expect(screen.queryByLabelText('Enable OTLP export')).not.toBeInTheDocument();
  });
});

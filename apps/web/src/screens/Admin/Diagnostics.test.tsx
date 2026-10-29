import { describe, expect, it, vi } from 'vitest';
import { fireEvent, screen } from '@solidjs/testing-library';
import { Diagnostics, type DiagnosticsApi } from './Diagnostics.tsx';
import { renderInShell } from './shell/testkit.tsx';
import { expectNoAxeViolations } from './shell/uxTestkit.ts';

const result = { id: 'run-1', suite: 'server', startedAt: 1, status: 'fail', results: [{ id: 'db', summary: 'Database accepts a query', status: 'fail', durationMs: 5 }] };
describe('diagnostics console', () => {
  it('copies the allowlisted server report with the selected results', async () => {
    const writeText=vi.fn<(value:string)=>Promise<void>>(async()=>undefined);
    Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText}});
    renderInShell(()=><Diagnostics api={{history:async()=>({kind:'ok',data:{runs:[result],nextCursor:null},etag:null}),run:async()=>({run:result,historySaved:true,auditSaved:true}),report:async()=>({schemaVersion:1,server:{mode:'engine'},redaction:{excluded:['credentials']}})}}/>);
    await screen.findByText('Database accepts a query');
    fireEvent.click(screen.getByRole('button',{name:'Copy troubleshooting report'}));
    await vi.waitFor(()=>expect(writeText).toHaveBeenCalledTimes(1));
    expect(JSON.parse(writeText.mock.calls[0]![0] as string)).toMatchObject({schemaVersion:1,server:{mode:'engine'},selectedRun:{id:'run-1'}});
  });
  it('asks the server to replace domain names and does not attach an unsaved run with names', async () => {
    const writeText=vi.fn<(value:string)=>Promise<void>>(async()=>undefined);
    Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText}});
    const named={...result,id:'unsaved',results:[{...result.results[0]!,domain:'secret.example',backendId:'primary'}]};
    const report=vi.fn(async(options?:{omitDomains?:boolean;selected?:string|undefined})=>({schemaVersion:1,redaction:{domainsOmitted:options?.omitDomains===true}}));
    renderInShell(()=><Diagnostics api={{history:async()=>({kind:'ok',data:{runs:[named],nextCursor:null},etag:null}),run:async()=>({run:named,historySaved:true,auditSaved:true}),report}}/>);
    await screen.findByText('Database accepts a query');
    fireEvent.click(screen.getByRole('checkbox',{name:'Replace domain names and backend IDs in the report'}));
    fireEvent.click(screen.getByRole('button',{name:'Copy troubleshooting report'}));
    await vi.waitFor(()=>expect(writeText).toHaveBeenCalledTimes(1));
    expect(report).toHaveBeenCalledWith({omitDomains:true,selected:'unsaved'});
    const text=writeText.mock.calls[0]![0] as string;
    expect(text).not.toContain('secret.example');
    expect(text).not.toContain('primary');
    expect(JSON.parse(text)).toMatchObject({redaction:{domainsOmitted:true},selectedRunOmitted:true});
  });
  it('filters backend results and sends cancellation for the running job', async () => {
    const cancel=vi.fn(async()=>undefined);
    const job={run:{...result,id:'bulk',status:'running',results:[{...result.results[0]!,domain:'a.test',backendId:'backup'}]},total:2,completed:1,limited:false,historySaved:false};
    const api:DiagnosticsApi={history:async()=>({kind:'ok',data:{runs:[],nextCursor:null},etag:null}),run:async()=>({run:result,historySaved:true,auditSaved:true}),start:async()=>job,job:async()=>job,cancel};
    renderInShell(()=><Diagnostics api={api}/>);
    const button=await screen.findByRole('button',{name:'Check all saved backends'});
    await vi.waitFor(()=>expect(button).toBeEnabled());fireEvent.click(button);
    fireEvent.click(await screen.findByRole('button',{name:'Cancel checks'}));
    await vi.waitFor(()=>expect(cancel).toHaveBeenCalledWith('bulk'));
    fireEvent.input(screen.getByLabelText('Filter checks, domains or backends'),{target:{value:'unrelated'}});
    expect(screen.queryByText('Database accepts a query')).not.toBeInTheDocument();
    fireEvent.input(screen.getByLabelText('Filter checks, domains or backends'),{target:{value:'backup'}});
    expect(screen.getByText('Database accepts a query')).toBeInTheDocument();
  });
  it('reports actual failed checks and unsaved history without claiming success', async () => {
    const api: DiagnosticsApi = {
      history: async () => ({ kind: 'ok', data: { runs: [], nextCursor: null }, etag: null }),
      run: vi.fn(async () => ({ run: result, historySaved: false, auditSaved: false })),
    };
    const { container } = renderInShell(() => <Diagnostics api={api} />);
    const button = await screen.findByRole('button', { name: 'Run server checks' });
    await vi.waitFor(() => expect(button).toBeEnabled());
    fireEvent.click(button);
    expect(await screen.findByText('Database accepts a query')).toBeInTheDocument();
    expect(screen.getByRole('cell', { name: 'fail' })).toBeInTheDocument();
    expect(screen.getByText(/history or audit storage failed/)).toBeInTheDocument();
    expect(api.run).toHaveBeenCalledTimes(1);
    await expectNoAxeViolations(container);
  });
  it('loads saved history and follows its cursor', async () => {
    const history = vi.fn(async (cursor?: string) => ({ kind: 'ok' as const, etag: null, data: { runs: cursor ? [{ ...result, id: 'older', startedAt: 0 }] : [result], nextCursor: cursor ? null : '1:run-1' } }));
    renderInShell(() => <Diagnostics api={{ history, run: async () => ({ run: result, historySaved: true, auditSaved: true }) }} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Older runs' }));
    await vi.waitFor(() => expect(history).toHaveBeenLastCalledWith('1:run-1'));
  });
});

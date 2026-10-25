import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { t } from '../../test/i18n.ts';
import { ScriptsPanel } from './ScriptsPanel.tsx';
import { VacationPanel } from './VacationPanel.tsx';
import type { SieveApi } from './sieve-api.ts';
import type { VacationApi, Vacation } from './vacation-api.ts';

function api(rawEdit = true): SieveApi {
  return {
    connect: vi.fn(async () => ({ maxSizeScriptName:512,maxSizeScript:65536,maxNumberScripts:5,maxNumberRedirects:4,sieveExtensions:['fileinto'],
      'urn:mailwoman:policy':{rawEdit,forwardExternal:false,ownDomains:['example.org'],runsAt:'engine' as const} })),
    get: vi.fn(async () => ({ accountId:'alice',state:'version-1',list:[{id:'script',name:'Actual source',blobId:'blob',isActive:true}],notFound:[] })),
    source: vi.fn(async () => '# handwritten\nkeep;'), validate: vi.fn(async () => null),
    save: vi.fn(async () => undefined),activate: vi.fn(async () => undefined),remove: vi.fn(async () => undefined),
  };
}
describe('authoritative scripts', () => {
  it('loads stored bytes and submits the observed state for a save', async () => {
    const service=api();render(() => <ScriptsPanel api={service}/>);
    await waitFor(()=>expect((screen.getByLabelText(t('rules-raw-label')) as HTMLTextAreaElement).value).toBe('# handwritten\nkeep;'));
    const editor=screen.getByLabelText(t('rules-raw-label')) as HTMLTextAreaElement;
    fireEvent.input(editor,{target:{value:'# handwritten\nstop;'}});
    fireEvent.click(screen.getByRole('button',{name:t('common-save')}));
    await waitFor(()=>expect(service.save).toHaveBeenCalledWith('script','Actual source','# handwritten\nstop;','version-1',false));
  });
  it('enforces raw-edit UI policy and separate active-script deactivation', async () => {
    const service=api(false);render(() => <ScriptsPanel api={service}/>);
    await waitFor(()=>expect(service.source).toHaveBeenCalled());
    expect((screen.getByRole('textbox',{name:t('rules-raw-label')}) as HTMLTextAreaElement).readOnly).toBe(true);
    expect(screen.getByRole('button',{name:t('common-save')})).toBeDisabled();
    expect(screen.getByRole('button',{name:t('common-delete')})).toBeDisabled();
    await waitFor(()=>expect(screen.getByRole('button',{name:t('rules-script-deactivate')})).not.toBeDisabled());
    fireEvent.click(screen.getByRole('button',{name:t('rules-script-deactivate')}));
    await waitFor(()=>expect(service.activate).toHaveBeenCalledWith(null,'version-1'));
  });
  it('ignores a late source response after switching accounts', async () => {
    const first=api();let finish!: (text:string)=>void;
    first.source=vi.fn(()=>new Promise<string>((resolve)=>{finish=resolve;}));
    const second=api();second.source=vi.fn(async ()=>'# second account\nkeep;');
    const [service,setService]=createSignal(first);render(()=> <ScriptsPanel api={service()}/>);
    await waitFor(()=>expect(first.source).toHaveBeenCalled());setService(second);
    await waitFor(()=>expect((screen.getByLabelText(t('rules-raw-label')) as HTMLTextAreaElement).value).toBe('# second account\nkeep;'));
    finish('# first account secret');await Promise.resolve();
    expect((screen.getByLabelText(t('rules-raw-label')) as HTMLTextAreaElement).value).toBe('# second account\nkeep;');
    expect(screen.queryByDisplayValue('# first account secret')).not.toBeInTheDocument();
  });
});
it('saves vacation fields with state and leaves a failed save visibly unsaved',async()=>{
  const value:Vacation={id:'singleton',isEnabled:false,fromDate:null,toDate:null,subject:null,textBody:'Old',htmlBody:null};
  const service:VacationApi={get:vi.fn(async()=>({state:'v1',value})),save:vi.fn(async()=>{throw new Error('forbidden');})};
  render(()=> <VacationPanel api={service}/>);
  const body=await screen.findByLabelText(t('rules-vacation-body'));
  await waitFor(()=>expect(screen.getByRole('button',{name:t('common-save')})).not.toBeDisabled());
  fireEvent.input(body,{target:{value:'Back tomorrow'}});
  fireEvent.click(screen.getByRole('button',{name:t('common-save')}));
  await waitFor(()=>expect(service.save).toHaveBeenCalledWith({...value,textBody:'Back tomorrow'},'v1'));
  expect(await screen.findByRole('alert')).toHaveTextContent('forbidden');
  expect(screen.queryByText(t('rules-vacation-saved'))).not.toBeInTheDocument();
});

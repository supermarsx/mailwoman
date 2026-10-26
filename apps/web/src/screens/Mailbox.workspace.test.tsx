import { describe, expect, it, vi } from 'vitest';
import { render, fireEvent, screen, within, waitFor } from '@solidjs/testing-library';
import { AppContext } from '../state/context.ts';
import { createAppState } from '../state/store.ts';
import { makeClient, mkEmail } from '../components/appHarness.tsx';
import { MailboxScreen } from './Mailbox.tsx';
import { RealtimeContext } from '../realtime/context.ts';
import { createRealtimeController } from '../realtime/controller.ts';
import type { EmailGetResponse, JmapRequest } from '../api/jmap-types.ts';

function mount(attachment=false) {
  const first=mkEmail('a');
  if(attachment)Object.assign(first,{attachments:[{partId:'2',blobId:'file-a',name:'report.txt',type:'text/plain',size:4}]});
  const emails=[first,mkEmail('b')];const client=makeClient({emails});
  const original=client.jmap;
  client.jmap=vi.fn(async (request:JmapRequest)=>{const response=await original(request);const ids=request.methodCalls.find(call=>call[0]==='Email/get')?.[1]['ids'];if(Array.isArray(ids)){for(const call of response.methodResponses)if(call[0]==='Email/get')(call[1] as unknown as EmailGetResponse).list=emails.filter(email=>ids.includes(email.id));}return response;});
  const app=createAppState(client);const realtime=createRealtimeController();
  const result=render(()=><AppContext.Provider value={app}><RealtimeContext.Provider value={realtime}><MailboxScreen/></RealtimeContext.Provider></AppContext.Provider>);
  return {app,realtime,...result};
}
describe('retained mailbox workspace',()=>{
  it('opens the selected message export editor in Settings and retains its reader',async()=>{
    const {app}=mount();await app.login({username:'me@example.org',password:'p'});
    fireEvent.click(await screen.findByText('Subject a',{selector:'.list__subject'}));
    fireEvent.click(await screen.findByTestId('reader-export'));
    expect(await screen.findByRole('heading',{name:'Export message'})).toBeInTheDocument();
    expect(screen.getByRole('tab',{name:'Subject a'})).toBeInTheDocument();
    expect(screen.getByRole('tab',{name:'Settings'})).toHaveAttribute('aria-selected','true');
  });
  it('opens attachments in their own reusable tab without replacing the message reader',async()=>{
    const {app,realtime}=mount(true);await app.login({username:'me@example.org',password:'p'});
    fireEvent.click(await screen.findByText('Subject a',{selector:'.list__subject'}));
    const item=await screen.findByRole('option',{name:/report.txt/});fireEvent.click(item);
    expect(screen.getByRole('tab',{name:'report.txt'})).toHaveAttribute('aria-selected','true');
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('tab',{name:'Subject a'}));
    fireEvent.click(screen.getByRole('option',{name:/report.txt/}));
    expect(realtime.subTabs.tabs().filter(tab=>tab.kind==='attachment')).toHaveLength(1);
    expect(screen.getByRole('tab',{name:'Subject a'})).toBeInTheDocument();
  });
  it('opens independent message tabs and restores each reader without another load',async()=>{
    const {app,realtime,container}=mount();await app.login({username:'me@example.org',password:'p'});
    fireEvent.click(await screen.findByText('Subject a',{selector:'.list__subject'}));
    await waitFor(()=>expect(app.openEmail()?.id).toBe('a'));
    fireEvent.click(screen.getByText('Subject b',{selector:'.list__subject'}));
    await waitFor(()=>expect(app.openEmail()?.id).toBe('b'));
    expect(container.querySelectorAll('.workspace-pane .reader__subject')).toHaveLength(2);
    const first=realtime.subTabs.tabs().find(tab=>tab.title==='Subject a')!;
    fireEvent.click(document.getElementById(`subtab-tab-${first.id}`)!);
    expect(app.openEmail()?.id).toBe('a');
    expect(screen.getByRole('tab',{name:'Subject a'})).toHaveAttribute('aria-selected','true');
  });
  it('keeps each composer mounted and opens sectioned Settings without a modal',async()=>{
    const {app,realtime}=mount();await app.login({username:'me@example.org',password:'p'});
    fireEvent.click(screen.getByRole('button',{name:/^Compose$/}));
    const first=realtime.subTabs.activeId()!;fireEvent.input(within(screen.getByRole('tabpanel',{name:'Compose'})).getByLabelText('Subject'),{target:{value:'Draft alpha'}});
    fireEvent.click(screen.getByRole('button',{name:/^Compose$/}));
    fireEvent.input(within(screen.getByRole('tabpanel',{name:'Compose'})).getByLabelText('Subject'),{target:{value:'Draft beta'}});
    fireEvent.click(document.getElementById(`subtab-tab-${first}`)!);
    expect(within(screen.getByRole('tabpanel',{name:'Compose'})).getByLabelText('Subject')).toHaveValue('Draft alpha');
    fireEvent.click(screen.getByRole('button',{name:/^Settings$/}));
    const settings=await screen.findByRole('region',{name:'Settings'});
    expect(within(settings).getByRole('navigation',{name:'Settings sections'})).toBeInTheDocument();
    expect(screen.queryByRole('dialog',{name:'Settings'})).not.toBeInTheDocument();
    fireEvent.click(document.getElementById(`subtab-tab-${first}`)!);
    expect(within(screen.getByRole('tabpanel',{name:'Compose'})).getByLabelText('Subject')).toHaveValue('Draft alpha');
  });
});

import { expect, it, vi } from 'vitest';
import { createRoot,createSignal } from 'solid-js';
import { createMailSlice } from './mail.ts';
import { makeClient, mkEmail } from '../../components/appHarness.tsx';
it('keeps automatic current-folder scope in structured JMAP filters while preserving all user-authored search operators',async()=>{
  const client=makeClient({emails:[mkEmail('a')]});
  await createRoot(async dispose=>{try{
    const mail=createMailSlice({client,showToast:vi.fn()});await mail.login({username:'me@example.org',password:'p'});
    const lastFilter=()=>vi.mocked(client.jmap).mock.calls.flatMap(([request])=>request.methodCalls.filter(([name])=>name==='Email/query').map(call=>call[1]['filter'])).at(-1)as Record<string,unknown>;
    await mail.searchMessages('hello in:Inbox',{scopeMailboxId:'inbox',baseQuery:'hello'});
    expect(mail.search()).toBe('hello in:Inbox');expect(lastFilter()).toMatchObject({inMailbox:'inbox',text:'hello'});expect(lastFilter()['text']).not.toContain('in:');
    await mail.searchMessages('from:alice is:followup in:Inbox',{scopeMailboxId:'inbox',baseQuery:'from:alice is:followup'});
    expect(lastFilter()).toMatchObject({inMailbox:'inbox',text:'from:alice is:followup'});
    await mail.searchMessages('hello in:Archive');expect(lastFilter()['inMailbox']).toBeUndefined();expect(lastFilter()['text']).toBe('hello in:archive');
  }finally{dispose();}});
});
it('uses the same structured automatic scope for reduced offline search',async()=>{
  const client=makeClient({emails:[mkEmail('a')]}),[online,setOnline]=createSignal(true),searchOffline=vi.fn(()=>[mkEmail('a')]);
  await createRoot(async dispose=>{try{
    const mail=createMailSlice({client,online,searchOffline,showToast:vi.fn()});await mail.login({username:'me@example.org',password:'p'});setOnline(false);
    await mail.searchMessages('hello in:Inbox',{scopeMailboxId:'inbox',baseQuery:'hello'});
    expect(searchOffline).toHaveBeenCalledWith(expect.objectContaining({text:'hello',inMailbox:'inbox'}));expect(mail.search()).toBe('hello in:Inbox');
  }finally{dispose();}});
});

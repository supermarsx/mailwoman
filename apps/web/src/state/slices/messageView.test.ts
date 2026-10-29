import { describe, expect, it, vi } from 'vitest';
import { createEffect, createRoot, createSignal, untrack } from 'solid-js';
import type { Email } from '../../api/jmap-types.ts';
import { createMessageView } from './messageView.ts';
const email=(id:string):Email => ({id,keywords:{},mailboxIds:{}} as Email);
describe('independent retained message views', () => {
  it('keeps separate bodies and refuses an older request overwriting a newer message', async () => {
    let finish!:(value:{email:Email;html:string})=>void;
    const load=vi.fn(async (_account:string,id:string) => id==='slow' ? await new Promise<{email:Email;html:string}>(resolve => {finish=resolve;}) : {email:email(id),html:id});
    const options={account:()=>'a',load,readDelay:()=>null,markRead:vi.fn()};
    const left=createMessageView(options);const right=createMessageView(options);
    const stale=left.openMessage('slow');await left.openMessage('new');await right.openMessage('right');
    finish({email:email('slow'),html:'stale'});await stale;
    expect(left.sanitizedHtml()).toBe('new');expect(right.sanitizedHtml()).toBe('right');
    left.dispose();right.dispose();
  });
  it('cancels hidden readers read timers and fences responses after account changes', async () => {
    vi.useFakeTimers();let dispose!:()=>void;
    let change!:(id:string)=>void;let account!:()=>string;
    createRoot(clean => {dispose=clean;const pair=createSignal('a');account=pair[0];change=pair[1];});
    const markRead=vi.fn();const view=createMessageView({account,load:async (_a,id)=>({email:email(id),html:id}),readDelay:()=>100,markRead});
    view.setActive(true);await view.openMessage('one');view.setActive(false);vi.advanceTimersByTime(100);expect(markRead).not.toHaveBeenCalled();
    view.setActive(true);vi.advanceTimersByTime(100);expect(markRead).toHaveBeenCalledWith('one');
    change('b');expect(view.openEmail()).toBeNull();view.dispose();dispose();vi.useRealTimers();
  });
  it('never publishes a response captured under an earlier account', async () => {
    let finish!:(value:{email:Email;html:string})=>void;
    const [account,change]=createSignal('old');
    const view=createMessageView({account,load:async ()=>await new Promise(resolve=>{finish=resolve;}),readDelay:()=>null,markRead:vi.fn()});
    const pending=view.openMessage('private');change('new');finish({email:email('private'),html:'old private body'});await pending;
    expect(view.openEmail()).toBeNull();expect(view.sanitizedHtml()).toBeNull();expect(view.readLoading()).toBe(false);view.dispose();
  });
  it('keeps a message opened while the view is created inside another computation (a restored tab)', async () => {
    // The workspace restore creates views and opens them from inside an effect,
    // where a new effect's first run waits for the outer one to finish.
    let view!:ReturnType<typeof createMessageView>;let pending!:Promise<void>;let dispose!:()=>void;
    createRoot(clean => {dispose=clean;createEffect(() => untrack(() => {
      view=createMessageView({account:()=>'a',load:async (_a,id)=>({email:email(id),html:`body ${id}`}),readDelay:()=>null,markRead:vi.fn()});
      pending=view.openMessage('restored');
    }));});
    await pending;
    expect(view.openEmail()?.id).toBe('restored');expect(view.sanitizedHtml()).toBe('body restored');
    view.dispose();dispose();
  });
  it('does not mark again a message the user marked unread while it stays open', async () => {
    vi.useFakeTimers();
    const markRead=vi.fn();
    const view=createMessageView({account:()=>'a',load:async (_a,id)=>({email:email(id),html:id}),readDelay:()=>100,markRead});
    view.setActive(true);await view.openMessage('one');vi.advanceTimersByTime(100);expect(markRead).toHaveBeenCalledTimes(1);
    view.patch('one',{keywords:{$seen:true}});
    view.patch('one',{keywords:{}});
    // The workspace re-asserts the active reader (setActive) on unrelated updates.
    view.setActive(false);view.setActive(true);vi.advanceTimersByTime(500);
    expect(markRead).toHaveBeenCalledTimes(1);
    // A reload of the same message (a remote-content grant) keeps it unread; another message is read as usual.
    await view.openMessage('one');vi.advanceTimersByTime(100);expect(markRead).toHaveBeenCalledTimes(1);
    await view.openMessage('two');vi.advanceTimersByTime(100);expect(markRead).toHaveBeenLastCalledWith('two');
    view.dispose();vi.useRealTimers();
  });
  it('follows changes made elsewhere: keywords silently, a move or a delete as a notice that keeps the copy', async () => {
    vi.useFakeTimers();
    const markRead=vi.fn();
    const view=createMessageView({account:()=>'a',load:async (_a,id)=>({email:{id,keywords:{},mailboxIds:{inbox:true}} as unknown as Email,html:'body'}),readDelay:()=>500,markRead});
    view.setActive(true);await view.openMessage('m');
    // Read on another device before the timer here fires: no second read write.
    view.reconcile('m',{keywords:{$seen:true},mailboxIds:{inbox:true}});
    vi.advanceTimersByTime(600);expect(markRead).not.toHaveBeenCalled();
    expect(view.openEmail()?.keywords).toEqual({$seen:true});expect(view.remoteChange()).toBeNull();
    // Unchanged state is a no-op; a local move (patch) is not reported as remote.
    view.patch('m',{mailboxIds:{archive:true}});view.reconcile('m',{keywords:{$seen:true},mailboxIds:{archive:true}});
    expect(view.remoteChange()).toBeNull();
    // Moved elsewhere: the body stays, the notice names the new folders.
    view.reconcile('m',{keywords:{$seen:true,$flagged:true},mailboxIds:{trash:true}});
    expect(view.remoteChange()).toEqual({kind:'moved',mailboxIds:{trash:true}});
    expect(view.sanitizedHtml()).toBe('body');expect(view.openEmail()?.keywords?.['$flagged']).toBe(true);
    view.dismissRemoteChange();expect(view.remoteChange()).toBeNull();
    // Gone: the copy stays open.
    view.reconcile('m',null);expect(view.remoteChange()).toEqual({kind:'deleted'});expect(view.openEmail()?.id).toBe('m');
    // Another message clears the notice; a reconcile for a message not open is ignored.
    view.reconcile('other',null);await view.openMessage('next');expect(view.remoteChange()).toBeNull();
    view.dispose();vi.useRealTimers();
  });
  it('keeps a message marked unread on another device unread here', async () => {
    vi.useFakeTimers();const markRead=vi.fn();
    const view=createMessageView({account:()=>'a',load:async (_a,id)=>({email:{id,keywords:{$seen:true},mailboxIds:{inbox:true}} as unknown as Email,html:''}),readDelay:()=>100,markRead});
    view.setActive(true);await view.openMessage('m');
    view.reconcile('m',{keywords:{},mailboxIds:{inbox:true}});
    view.setActive(false);view.setActive(true);vi.advanceTimersByTime(200);
    expect(markRead).not.toHaveBeenCalled();view.dispose();vi.useRealTimers();
  });
});

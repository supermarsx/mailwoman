import {describe,expect,it,vi} from 'vitest';
import {activeLinkedMailbox,installLinkedMailbox,linkedMailboxEpoch,onLinkedMailboxChange,type LinkedMailboxChange} from './linkedMailboxSession.ts';

const child={linkId:'11111111-1111-4111-8111-111111111111',accountId:'child',storageAuthority:'auth-child'};

describe('onLinkedMailboxChange',()=>{
  it('fires on every install with the new context, the previous one and the epoch',()=>{
    installLinkedMailbox(null);
    const seen:LinkedMailboxChange[]=[];
    const stop=onLinkedMailboxChange(change=>{
      // The new context is already active when listeners run.
      expect(activeLinkedMailbox()).toBe(change.context);
      seen.push(change);
    });
    installLinkedMailbox(child);
    installLinkedMailbox(null);
    stop();
    installLinkedMailbox(child);
    expect(seen).toHaveLength(2);
    expect(seen[0]!.context).toEqual(child);
    expect(seen[0]!.previous).toBeNull();
    expect(seen[1]!.context).toBeNull();
    expect(seen[1]!.previous).toEqual(child);
    expect(seen[1]!.epoch).toBe(seen[0]!.epoch+1);
    expect(linkedMailboxEpoch()).toBe(seen[1]!.epoch+1);
    installLinkedMailbox(null);
  });

  it('keeps notifying the other listeners when one throws',()=>{
    const error=vi.spyOn(console,'error').mockImplementation(()=>{});
    const second=vi.fn();
    const a=onLinkedMailboxChange(()=>{throw new Error('boom');});
    const b=onLinkedMailboxChange(second);
    installLinkedMailbox(child);
    a();b();
    expect(second).toHaveBeenCalledOnce();
    expect(error).toHaveBeenCalled();
    error.mockRestore();
    installLinkedMailbox(null);
  });

  it('does not hand listeners a mutable copy of the caller context',()=>{
    const input={...child};
    let got:LinkedMailboxChange|undefined;
    const stop=onLinkedMailboxChange(change=>{got=change;});
    installLinkedMailbox(input);
    stop();
    input.accountId='changed';
    expect(got!.context!.accountId).toBe('child');
    expect(Object.isFrozen(got!.context)).toBe(true);
    installLinkedMailbox(null);
  });
});

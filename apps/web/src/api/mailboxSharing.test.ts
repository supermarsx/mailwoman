import {afterEach,expect,it,vi} from 'vitest';
import {createRoot} from 'solid-js';
import {activateAccountPreferences,suspendAccountPreferences} from './accountPreferenceLifecycle.ts';
import {createSharingAccess} from './mailboxSharing.ts';
import {createAclClient,MailboxSharingError} from './acl-types.ts';
import type {JmapResponse} from './jmap-types.ts';
afterEach(suspendAccountPreferences);
const success=(accountId='a',mailboxId='box',myRights='lra'):JmapResponse=>({methodResponses:[['MailboxRights/get',{accountId,mailboxId,myRights,acl:[]},'mr']],sessionState:'s'});
it('only exposes sharing after exact account/folder administer proof',async()=>{
 activateAccountPreferences('a');let dispose!:()=>void;const jmap=vi.fn(async()=>success());const access=createRoot(cleanup=>{dispose=cleanup;return createSharingAccess(()=> 'a',{jmap});});
 expect(access.canShare('box')).toBe(false);expect(await access.probe('box')).toBe(true);expect(access.canShare('box')).toBe(true);
 jmap.mockResolvedValue(success('a','box','lr'));expect(await access.probe('box',true)).toBe(false);expect(access.canShare('box')).toBe(false);
 jmap.mockResolvedValue(success('foreign'));expect(await access.probe('box',true)).toBe(false);dispose();
});
it('ignores delayed rights from a superseded sign-in',async()=>{
 activateAccountPreferences('a');let resolve!:(response:JmapResponse)=>void,dispose!:()=>void;const access=createRoot(cleanup=>{dispose=cleanup;return createSharingAccess(()=> 'a',{jmap:async()=>await new Promise<JmapResponse>(done=>{resolve=done;})});});
 const pending=access.probe('box');suspendAccountPreferences();activateAccountPreferences('a');resolve(success());expect(await pending).toBe(false);expect(access.canShare('box')).toBe(false);dispose();
});
it('distinguishes unsupported and per-identifier mutation refusal',async()=>{
 const unsupported=createAclClient('a',async()=>({methodResponses:[['error',{type:'unsupported'},'mr']],sessionState:'s'}));await expect(unsupported.getMailboxRights('box')).rejects.toEqual(new MailboxSharingError('unsupported'));
 const refused=createAclClient('a',async()=>({methodResponses:[['MailboxRights/set',{accountId:'a',mailboxId:'box',notUpdated:{bob:{type:'forbidden'}}},'mrs']],sessionState:'s'}));await expect(refused.grant('box','bob','lr')).rejects.toEqual(new MailboxSharingError('forbidden'));
});

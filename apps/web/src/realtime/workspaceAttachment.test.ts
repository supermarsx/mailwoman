import {expect,it,vi} from 'vitest';
import type {Client} from '../api/client.ts';
import {resolveWorkspaceAttachment} from './workspaceAttachment.ts';
it('resolves fresh owned attachment metadata and refuses foreign or missing results',async()=>{
 const email={id:'message',subject:'Current title',from:[],receivedAt:'2026-01-01T00:00:00Z',attachments:[{blobId:'blob',name:'current.txt',type:'text/plain',size:4}]};
 const client={jmap:vi.fn(async()=>({methodResponses:[['Email/get',{accountId:'a',list:[email],notFound:[]},'workspace-attachment']],sessionState:'s'}))} as unknown as Pick<Client,'jmap'>;
 expect((await resolveWorkspaceAttachment(client,'a','message','blob'))?.name).toBe('current.txt');
 expect(await resolveWorkspaceAttachment(client,'other','message','blob')).toBeNull();
 expect(await resolveWorkspaceAttachment(client,'a','different','blob')).toBeNull();
 expect(await resolveWorkspaceAttachment(client,'a','message','missing')).toBeNull();
});

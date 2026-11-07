import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {installLinkedMailbox} from '../api/linkedMailboxSession.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {apiFetch} from '../api/http.ts';
import {blobUrlToDataUrl,blobUrlToText} from './sandbox.ts';
import {loadText} from './TextViewer.tsx';
import {buildDownloadUrl} from './attachments.ts';
import {exportUrl} from '../components/properties/folderExport.ts';

// A linked mailbox is active: every preview reads bytes the page already holds (blob: URLs), which must never
// reach the linked-scope check; real downloads must carry the child's selector.
const child={linkId:'11111111-1111-4111-8111-111111111111',accountId:'child-acc',storageAuthority:'msa1_child_verified'};
let send:ReturnType<typeof vi.fn>;
beforeEach(()=>{
  activateAccountPreferences(child.accountId,child.storageAuthority);installLinkedMailbox(child);
  send=vi.fn(async(url:string)=>new Response(url.startsWith('blob:')||url.startsWith('data:')?'preview-bytes':'{}'));vi.stubGlobal('fetch',send);
});
afterEach(()=>{installLinkedMailbox(null);suspendAccountPreferences();vi.unstubAllGlobals();});
const sentHeader=(i:number)=>new Headers((send.mock.calls[i]?.[1] as RequestInit|undefined)?.headers).get('x-mailwoman-link');

describe('previews while a linked mailbox is active',()=>{
  it.each([['image','image/png'],['audio','audio/mpeg'],['pdf','application/pdf'],['office','application/vnd.openxmlformats-officedocument.wordprocessingml.document'],['archive','application/zip']])('reads the %s bytes from a blob: URL without the link selector',async(_kind,mime)=>{
    const url=`blob:https://webmail.example/${_kind}`;
    const response=await apiFetch(url);
    expect(await response.text()).toBe('preview-bytes');
    expect(send).toHaveBeenCalledTimes(1);expect(sentHeader(0)).toBeNull();
    expect(mime).toBeTruthy();
  });
  it('image and audio frames get a data: URL',async()=>{
    await expect(blobUrlToDataUrl('blob:https://webmail.example/i','image/png')).resolves.toMatch(/^data:/);
  });
  it('text previews decode',async()=>{
    expect(await blobUrlToText('blob:https://webmail.example/t')).toBe('preview-bytes');
    expect(await loadText('blob:https://webmail.example/t','text/plain')).toMatchObject({kind:expect.not.stringMatching(/^error$/)});
  });
  it('the attachment download URL carries the linked selector (download, Download all, drag-out use the same builder)',async()=>{
    const url=buildDownloadUrl('/jmap/download/{accountId}/{blobId}/{name}?type={type}',{accountId:child.accountId,blobId:'b1',name:'a.png',mime:'image/png'});
    expect(url).toContain(`mwLink=${child.linkId}`);
    await apiFetch(url);
    expect(String(send.mock.calls[0]?.[0])).not.toContain('mwLink');expect(sentHeader(0)).toBe(child.linkId);
  });
  it('refuses a download naming another account instead of falling back to the primary',()=>{
    expect(()=>buildDownloadUrl('/jmap/download/{accountId}/{blobId}/{name}',{accountId:'primary-acc',blobId:'b1',name:'a',mime:'x/y'})).toThrow('Mailbox changed');
  });
  it('message .eml export and the folder ZIP run as the linked mailbox',async()=>{
    await apiFetch('/api/export/msg1?format=eml');
    await apiFetch(exportUrl('mb1',{format:'eml',subfolders:true}));
    expect(sentHeader(0)).toBe(child.linkId);expect(sentHeader(1)).toBe(child.linkId);
    expect(String(send.mock.calls[1]?.[0])).toContain('/api/export/folder/mb1.zip');
    expect(exportUrl('mb1',{format:'eml',subfolders:true})).toContain(`mwLink=${child.linkId}`);
  });
});

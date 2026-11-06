import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {stripIsolates} from '../i18n/index.ts';
import * as http from '../api/http.ts';
import * as zip from '../viewers/zipStore.ts';
import {collectAttachmentGroups,downloadAttachmentsOf} from './messageAttachmentsBulk.ts';
import {downloadZip,folderNames,type Named} from './messageAttachmentDownload.ts';
import type {AttachmentPolicy} from '../api/attachmentPolicy.ts';
import type {Email} from '../api/jmap-types.ts';
import type {MessageExtras} from './messageRelations.ts';

const email=(id:string,over:Partial<Email>={}):Email=>({id,mailboxIds:{inbox:true},from:null,to:null,subject:`Subject ${id}`,receivedAt:'2026-01-01T00:00:00Z',preview:null,hasAttachment:true,...over}) as Email;
const part=(name:string,size=10,type='application/pdf')=>({blobId:`b-${name}`,type,name,size,partId:name,cid:null,disposition:'attachment'}) as never;
const POLICY:AttachmentPolicy={previewAllowed:true,downloadAllowed:true,maxBytes:0,allowedExtensions:[],blockedExtensions:['exe'],warnExtensions:[],allowedMimeTypes:[],blockedMimeTypes:[],warnMimeTypes:[]};
const item=(name:string,size=10):Named=>({part:part(name,size),name,mime:'application/pdf',size});

describe('folderNames',()=>{
 it('makes one safe folder name per message and numbers repeats',()=>{
  expect(folderNames(['Plans','Plans','a/b:c',' ',''])).toEqual(['Plans','Plans (2)','a_b_c','(no subject)','(no subject) (2)'].map(name=>stripIsolates(name)));
 });
});

describe('collectAttachmentGroups',()=>{
 it('reads only messages that have attachments, a few at a time, and counts the ones that cannot be read',async()=>{
  let running=0,peak=0;
  const fetchExtras=vi.fn(async(_c:unknown,_a:string,message:Pick<Email,'id'>):Promise<MessageExtras>=>{
   running++;peak=Math.max(peak,running);await new Promise(r=>setTimeout(r,2));running--;
   if(message.id==='bad')throw new Error('nope');
   return {attachments:message.id==='empty'?[]:[part(`${message.id}.pdf`)],replies:[],forwards:[]};
  });
  const messages=[email('a'),email('plain',{hasAttachment:false}),email('bad'),email('empty'),...['c','d','e','f','g'].map(id=>email(id))];
  const {groups,failed}=await collectAttachmentGroups({jmap:vi.fn()} as never,'acct',messages,new Set(),fetchExtras as never);
  expect(fetchExtras).toHaveBeenCalledTimes(8);
  expect(peak).toBeLessThanOrEqual(4);
  expect(failed).toBe(1);
  expect(groups.map(group=>group.emailId)).toEqual(['a','c','d','e','f','g']);
  expect(groups[0]!.items[0]!.name).toBe('a.pdf');
 });
});

describe('downloadZip: policy summary',()=>{
 beforeEach(()=>{
  vi.spyOn(http,'apiFetch').mockImplementation(async()=>({ok:true,status:200,blob:async()=>({size:1,arrayBuffer:async()=>new Uint8Array([120]).buffer})}) as never);
  vi.stubGlobal('URL',Object.assign(URL,{createObjectURL:()=>'blob:x',revokeObjectURL:()=>undefined}));
  vi.spyOn(HTMLAnchorElement.prototype,'click').mockImplementation(()=>undefined);
 });
 afterEach(()=>{vi.restoreAllMocks();vi.unstubAllGlobals();});
 const app=()=>({accountId:()=>'acct',downloadUrl:()=>'/d/{accountId}/{blobId}/{name}?type={type}',showToast:vi.fn()});

 it('leaves out files the policy blocks, says how many, and still downloads the rest',async()=>{
  const a=app();
  await downloadZip(a,[{emailId:'m1',label:'One',items:[item('ok.pdf'),item('bad.exe')]},{emailId:'m2',label:'Two',items:[item('also.pdf'),item('worse.exe')]}],POLICY);
  expect(a.showToast).toHaveBeenCalledTimes(1);
  const [tone,message]=a.showToast.mock.calls[0]!;
  expect(tone).toBe('info');
  expect(stripIsolates(message)).toContain('Downloaded 2 attachments as a ZIP.');
  expect(stripIsolates(message)).toContain('2 files blocked by the attachment policy were left out.');
 });
 it('downloads nothing, and says so, when the policy blocks every file',async()=>{
  const a=app();
  await downloadZip(a,[{emailId:'m1',label:'One',items:[item('a.exe'),item('b.exe')]}],POLICY);
  expect(http.apiFetch).not.toHaveBeenCalled();
  expect(a.showToast).toHaveBeenCalledWith('error',expect.stringContaining('blocks all 2 files'));
 });
 it('a clean download reports success without a skipped note',async()=>{
  const a=app();
  await downloadZip(a,[{emailId:'m1',label:'One',items:[item('a.pdf')]}],POLICY);
  expect(a.showToast).toHaveBeenCalledWith('success',expect.not.stringContaining('blocked'));
 });
 it('one folder per message when several messages are zipped',async()=>{
  const a=app();
  const build=vi.spyOn(zip,'buildZip');
  await downloadZip(a,[{emailId:'m1',label:'Same',items:[item('a.pdf')]},{emailId:'m2',label:'Same',items:[item('a.pdf')]}],POLICY);
  expect(build.mock.calls[0]![0].map(entry=>entry.name)).toEqual(['Same/a.pdf','Same (2)/a.pdf']);
 });
});

describe('downloadAttachmentsOf',()=>{
 it('tells the user when the selected messages have nothing to download',async()=>{
  const showToast=vi.fn();
  await downloadAttachmentsOf({accountId:()=>'acct',downloadUrl:()=>'/d',showToast,mailboxes:()=>[]},{jmap:vi.fn()} as never,[email('a')],POLICY,async()=>({attachments:[],replies:[],forwards:[]}));
  expect(showToast).toHaveBeenLastCalledWith('error',expect.stringContaining('no attachments'));
 });
});

import { describe, expect, it } from 'vitest';
import { attachmentDecision, attachmentPolicyUrl, type AttachmentPolicy } from './attachmentPolicy.ts';
const policy:AttachmentPolicy={previewAllowed:true,downloadAllowed:true,maxBytes:10,allowedExtensions:[],blockedExtensions:['exe'],warnExtensions:['txt'],allowedMimeTypes:[],blockedMimeTypes:['application/x-msdownload'],warnMimeTypes:[]};
describe('attachment policy presentation',()=>{
  it('blocks normalized extensions and respects size and independent preview setting',()=>{
    expect(attachmentDecision(policy,{name:'safe.pdf.EXE. ',mime:'text/plain',size:1}).allowed).toBe(false);
    expect(attachmentDecision(policy,{name:'note.txt',mime:'text/plain',size:10})).toEqual({allowed:true,preview:true,warning:true});
    expect(attachmentDecision(policy,{name:'note.txt',mime:'text/plain',size:11}).allowed).toBe(false);
    expect(attachmentDecision({...policy,previewAllowed:false},{name:'a.pdf',mime:'application/pdf',size:1})).toEqual({allowed:true,preview:false,warning:false});
  });
  it('keeps the authenticated blob path and adds owned-message context explicitly',()=>{
    const url=new URL(attachmentPolicyUrl('https://mail.example/base/jmap/download/a/b/name?type=x','message/id',true,true));
    expect(url.origin).toBe('https://mail.example');expect(url.pathname).toBe('/base/jmap/download/a/b/name');
    expect(url.searchParams.get('emailId')).toBe('message/id');expect(url.searchParams.get('preview')).toBe('true');expect(url.searchParams.get('acknowledgeWarning')).toBe('true');
  });
});
